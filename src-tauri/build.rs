// build.rs — embebe el manifest Windows y dispara `tauri-build`.
//
// Diferencia debug/release:
//   - DEBUG   → `asInvoker`. Sin esto, `cargo run` falla con error 740
//               porque la terminal de desarrollo no está elevada.
//   - RELEASE → `requireAdministrator`. La app final pide UAC al arrancar,
//               coherente con el principio de "elevación al inicio" elegido.
//
// El fichero `ClearTool.exe.manifest` que vive al lado de este `build.rs`
// queda como referencia documental — Tauri ignora `.manifest` sueltos; el
// manifest activo es el que se embebe aquí.

fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let exec_level = if profile == "release" {
        "requireAdministrator"
    } else {
        "asInvoker"
    };

    let manifest = format!(
        r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="{level}" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
      <supportedOS Id="{{1f676c76-80e1-4239-95bb-83d0f6d0da78}}"/>
    </application>
  </compatibility>
  <asmv3:application xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <asmv3:windowsSettings xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">
      <dpiAware>True/PM</dpiAware>
    </asmv3:windowsSettings>
    <asmv3:windowsSettings xmlns="http://schemas.microsoft.com/SMI/2007/WindowsSettings">
      <dpiAwareness>PerMonitorV2</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>"#,
        level = exec_level
    );

    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new().app_manifest(&manifest),
        ),
    )
    .expect("failed to run tauri-build");

    // Le decimos a cargo que vuelva a correr build.rs si cambia el perfil.
    println!("cargo:rerun-if-changed=build.rs");
}
