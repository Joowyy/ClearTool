# Qwen 3.6 Plus — Features mecánicas (M1 → M4 completion)

> **Tareas formularias** que cierran items del Done de M1/M3/M4 sin diseño.
> Cada bloque es independiente — puedes commitear uno a uno.
>
> Si una tarea te lleva > 2h, **para** y pregunta — significa que se ha
> escalado en complejidad y posiblemente la debería hacer Sonnet.

---

## 1. i18n provider (M4 — "i18n base ES+EN")

Hoy hay `src/locales/{es,en}/common.json` pero **nada los consume** — no está
instalado `i18next`, no hay provider, no se usa `useTranslation`. Las cadenas
están hardcoded en español por todas las páginas.

### Setup mínimo

**`package.json`:**
```bash
npm install i18next react-i18next i18next-browser-languagedetector
```

**`src/lib/i18n.ts` (nuevo):**
```ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import esCommon from "../locales/es/common.json";
import enCommon from "../locales/en/common.json";

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      es: { common: esCommon },
      en: { common: enCommon },
    },
    fallbackLng: "es",
    defaultNS: "common",
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "cleartool.lang",
      caches: ["localStorage"],
    },
  });

export default i18n;
```

**`src/main.tsx`:** importar `./lib/i18n` antes del `createRoot`.

### Migrar Sidebar como ejemplo

[src/components/layout/sidebar.tsx] — cambiar las cadenas hardcoded por:
```tsx
import { useTranslation } from "react-i18next";
const { t } = useTranslation();
// {t("nav.home")} en vez de "Inicio"
```

**Importante:** **no migres todas las páginas en un único PR**. Migra solo el
Sidebar y dos páginas (Home, Cache) en este commit. El resto queda como
follow-up task en `docs/HISTORIAL-SESIONES.md`.

### Selector de idioma en Settings

[src/features/settings/settings-page.tsx] — añadir bloque dentro del Card
"Apariencia":

```tsx
<div>
  <label className="text-sm font-medium mb-2 block">{t("settings.language")}</label>
  <div className="flex gap-2">
    {[
      { code: "es", label: "Español" },
      { code: "en", label: "English" },
    ].map(({ code, label }) => (
      <Button
        key={code}
        size="sm"
        variant={i18n.language.startsWith(code) ? "default" : "outline"}
        onClick={() => i18n.changeLanguage(code)}
      >
        {label}
      </Button>
    ))}
  </div>
</div>
```

**Añadir cadenas faltantes** en ambos `common.json`:
```json
"settings": {
  "language": "Idioma" / "Language",
  ...
}
```

**Acceptance.** Cambiar idioma en Settings → el Sidebar y Home se traducen
inmediatamente. Recargar la app: el idioma persiste.

---

## 2. Modal Ctrl+/ con la lista de atajos (M4)

**Archivo:** [src/hooks/use-keyboard-shortcuts.ts:48-53](../../src/hooks/use-keyboard-shortcuts.ts#L48-L53)

Hoy hay un `/* TODO */` en el handler de Ctrl+/. Implementar el modal.

### Patrón

Estado global de "is shortcuts modal open" en `useAppStore` (`src/lib/store.ts`).
El handler de Ctrl+/ hace `toggleShortcutsModal()`. Un componente nuevo
`<ShortcutsModal />` se monta en `app-shell.tsx` y se renderiza condicionalmente.

**`src/components/shortcuts-modal.tsx` (nuevo):**

Lista hardcoded por ahora (los atajos no cambian dinámicamente):

```tsx
const SHORTCUTS = [
  { keys: ["Ctrl", "K"], description: "Abrir paleta de comandos" },
  { keys: ["Ctrl", ","], description: "Ir a Ajustes" },
  { keys: ["Ctrl", "E"], description: "Ir a Explorador" },
  { keys: ["Ctrl", "B"], description: "Ir a Debloat" },
  { keys: ["Ctrl", "L"], description: "Ir a Caché" },
  { keys: ["Ctrl", "/"], description: "Mostrar esta ayuda" },
  { keys: ["Esc"], description: "Cerrar diálogos / paleta" },
];
```

UI: usar `@radix-ui/react-dialog` (ya está en deps), estilizado al panel-raised
de la app, con `<kbd>` para cada tecla.

**Acceptance.** Ctrl+/ abre modal, Esc lo cierra. Ctrl+/ otra vez también lo cierra.

---

## 3. Sección "Herramientas de red" en Settings (M4)

**Archivo:** [src/features/settings/settings-page.tsx](../../src/features/settings/settings-page.tsx)

Los 6 comandos de network están expuestos en `src/api/client.ts:280-285` pero
ninguna UI los usa. Spec M4: "6 botones en Settings → Tools".

### Card nueva al final de Settings

Insertar tras el Card "Avanzado":

```tsx
<Card>
  <CardHeader>
    <CardTitle>Herramientas de red</CardTitle>
    <p className="text-xs text-muted-foreground">
      Operaciones de reparación de red. Crean restore point antes de ejecutar.
    </p>
  </CardHeader>
  <CardContent>
    <div className="grid grid-cols-2 gap-2">
      <NetworkToolButton label="Limpiar caché DNS" command={flushDns} />
      <NetworkToolButton label="Renovar IP" command={renewIp} />
      <NetworkToolButton label="Reset Winsock" command={resetWinsock} />
      <NetworkToolButton label="Reset TCP/IP" command={resetTcpip} />
      <NetworkToolButton label="Reset proxy" command={resetProxy} />
      <NetworkToolButton label="Restaurar hosts" command={restoreHostsFile} destructive />
    </div>
  </CardContent>
</Card>
```

**`NetworkToolButton` component** (mismo archivo):
- Recibe `command: (dryRun: boolean) => Promise<void>` y `destructive?: boolean`.
- Si `destructive`, usa `ConfirmDialog` (`src/components/ui/confirm-dialog.tsx`) antes
  de ejecutar.
- Estado: idle / running / done / error. Muestra toast (`src/lib/toast.ts`) con resultado.
- Honra el setting `active.safety.dryRunGlobal` — si está activo, pasa `dryRun: true` y
  muestra "Simulación completada".

**Importar:** `flushDns, renewIp, resetWinsock, resetTcpip, resetProxy, restoreHostsFile`
desde `../../api/client`.

**Acceptance.** Click en "Limpiar caché DNS" → toast "DNS cache vacío" o error. Click en
"Restaurar hosts" → confirmación, luego ejecución.

---

## 4. Botón "Exportar reporte de diagnóstico" (M1)

**Archivo:** [src/features/settings/settings-page.tsx](../../src/features/settings/settings-page.tsx) — Card "Avanzado", al lado del toggle "Modo diagnóstico".

**Backend nuevo.** `src-tauri/src/ipc/diagnostics.rs`:

```rust
#[tauri::command]
pub async fn export_diagnostic_zip(app: tauri::AppHandle) -> AppResult<String> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let dir = app.path().download_dir()
        .map_err(|e| AppError::Validation(format!("{}", e)))?;
    let zip_path = dir.join(format!("cleartool-diag-{}.zip", chrono::Utc::now().format("%Y%m%d-%H%M%S")));

    let file = std::fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. settings.json
    if let Ok(s) = crate::domain::system_info::system_summary() {
        zip.start_file("system_summary.json", opts)?;
        zip.write_all(serde_json::to_string_pretty(&s)?.as_bytes())?;
    }
    // 2. settings actuales
    let settings = crate::core::settings::current();
    zip.start_file("settings.json", opts)?;
    zip.write_all(serde_json::to_string_pretty(&settings)?.as_bytes())?;
    // 3. audit log (últimas 200 entries)
    let audit = crate::domain::audit::list_log().unwrap_or_default();
    let recent: Vec<_> = audit.iter().rev().take(200).collect();
    zip.start_file("audit-recent.json", opts)?;
    zip.write_all(serde_json::to_string_pretty(&recent)?.as_bytes())?;
    // 4. catálogos cargados
    if let Ok(c) = crate::domain::catalog::load_cache_locations() {
        zip.start_file("cache-catalog.json", opts)?;
        zip.write_all(serde_json::to_string_pretty(&c)?.as_bytes())?;
    }
    // 5. info de versión
    zip.start_file("version.txt", opts)?;
    zip.write_all(format!("ClearTool v{}\nBuilt: {}\n", env!("CARGO_PKG_VERSION"), env!("BUILD_DATE")).as_bytes())?;

    zip.finish().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    Ok(zip_path.display().to_string())
}
```

`Cargo.toml`:
```toml
zip = { version = "2", default-features = false, features = ["deflate"] }
```

Registrar comando en `lib.rs`.

**Frontend:**
```tsx
<Button variant="outline" size="sm" onClick={async () => {
  try {
    const path = await invoke<string>("export_diagnostic_zip");
    toast.success("Reporte generado", { description: path });
  } catch (e) {
    toast.error("Error generando reporte", { description: formatError(e) });
  }
}}>
  <FileArchive className="h-4 w-4 mr-1" />
  Exportar reporte de diagnóstico
</Button>
```

**Acceptance.** Click → aparece el ZIP en Downloads con los 5 archivos.

---

## 5. Catálogo bloatware: expandir de 11 a 60+ entradas (M3)

**Archivo:** `.claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json`

Spec M3 dice 120+; objetivo realista para Qwen en esta primera pasada: **60+**.
Las restantes 60 quedan como follow-up.

### Categorías y mínimo por categoría

| Categoría | Mínimo | Ejemplos de PFN |
|---|---|---|
| `microsoft-bundled` | 20 | `Microsoft.BingNews`, `Microsoft.GetHelp`, `Microsoft.Getstarted`, `Microsoft.MicrosoftSolitaireCollection`, `Microsoft.WindowsFeedbackHub`, `Microsoft.MicrosoftStickyNotes`, `Microsoft.MixedReality.Portal`, `Microsoft.People`, `Microsoft.WindowsAlarms`, `Microsoft.WindowsCamera`, `Microsoft.windowscommunicationsapps` (Mail+Calendar), `Microsoft.WindowsMaps`, `Microsoft.YourPhone`, `Microsoft.ZuneMusic`, `Microsoft.ZuneVideo`, `Microsoft.Office.OneNote`, `Microsoft.OneConnect`, `Microsoft.SkypeApp`, `Microsoft.Wallet`, `Microsoft.Print3D` |
| `oem-crapware` | 10 | Patrones `*McAfee*`, `*Norton*`, `*Lenovo.*`, `*Dell.*Update*`, `*HP.*Wolf*` (usar `name_pattern` con regex) |
| `xbox-gaming` | 6 | `Microsoft.XboxApp`, `Microsoft.XboxGamingOverlay`, `Microsoft.XboxIdentityProvider`, `Microsoft.XboxSpeechToTextOverlay`, `Microsoft.GamingApp`, `Microsoft.GamingServices` |
| `tiktok-social` | 4 | `Microsoft.Todos`, `Clipchamp.Clipchamp`, `5319275A.WhatsAppDesktop`, `BytedancePte.Ltd.TikTok` |
| `copilot-ai` | 4 | `Microsoft.Copilot`, `Microsoft.WindowsCopilot`, `Microsoft.PowerAutomateDesktop`, `Microsoft.MicrosoftEdge.Stable` (con disclaimer) |
| `developer-bloat` | 6 | `Microsoft.NET.Native.Framework.*` (frameworks: solo marcar, no eliminar por defecto), `Microsoft.VCLibs.*`, `Microsoft.UI.Xaml.*` |
| `optional-features` | 10 | Características Windows opcionales: `Microsoft.Windows.HolographicFirstRun`, `Microsoft.MicrosoftEdgeDevToolsClient`, etc. |

### Formato por entrada (schema actual)

```json
{
  "id": "ms-bing-news",
  "displayName": "Microsoft News",
  "category": "microsoft-bundled",
  "packageFamilyName": "Microsoft.BingNews_8wekyb3d8bbwe",
  "uninstallStrings": [],
  "risk": "low",
  "requiresDisclaimer": false,
  "consequences": ["Sin widget de noticias en taskbar"],
  "minWindowsBuild": 22000,
  "reverse": {
    "kind": "storeReinstall",
    "msStoreId": "9WZDNCRFHVFW"
  }
}
```

Si encuentras campos en el catálogo actual que no documenté (mira las 11
existentes), respeta el schema actual exactamente — no inventes campos
nuevos. Si crees que falta uno, déjalo como follow-up.

**Validar.** El JSON tiene `bloatware-catalog.schema.json` al lado. Si el JSON
no valida, `domain::catalog::validate_all()` aborta al arranque (línea 56 de
[lib.rs](../../src-tauri/src/lib.rs#L55-L56)). Tras añadir entries: `cargo run` y
verifica que la app arranca.

**Acceptance.** Catálogo con 60+ entries, la app arranca, página Debloat muestra
todas en la lista de catálogo.

---

## 6. Theme "follow system" (M4)

**Archivos:**
- [src/lib/store.ts](../../src/lib/store.ts) — añadir `"system"` al type Theme
- `src/lib/theme.ts` (nuevo) — hook con `matchMedia`
- [src/features/settings/settings-page.tsx:131-168](../../src/features/settings/settings-page.tsx#L131-L168) — añadir botón

### Cambios

**`store.ts`:** `Theme = "dark-cyan" | "dark-amber" | "light" | "system"`.

**`src/lib/theme.ts`:**
```ts
import { useEffect } from "react";
import { useAppStore } from "./store";

export function useThemeEffect() {
  const theme = useAppStore(s => s.theme);

  useEffect(() => {
    const root = document.documentElement;
    const effective = theme === "system"
      ? (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark-cyan" : "light")
      : theme;
    root.dataset.theme = effective;

    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = (e: MediaQueryListEvent) => {
        root.dataset.theme = e.matches ? "dark-cyan" : "light";
      };
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    }
  }, [theme]);
}
```

**`app-shell.tsx`:** `useThemeEffect()` cerca del top del componente.

**Settings page:** añadir cuarto botón "Sistema" con `icon={Monitor}` en el group de themes.

**Acceptance.** Elegir "Sistema" → la app respeta el modo OS. Cambiar el OS dark/light en Settings de Windows → la app cambia sin reload.

---

## 7. Limpieza de mocks `web mode` (M2 nueva ruta)

**Archivo:** [src/api/client.ts:82-83](../../src/api/client.ts#L82-L83)

Los mocks de `analyze_cache_locations` y `execute_clean_plan` se ven horribles
porque están en una sola línea. No es funcionalmente incorrecto, pero al menos:

- Formatear el objeto multi-línea, una key por línea.
- Verificar que cualquier comando nuevo añadido por el equipo (especialmente
  `app_version` de la tarea 10 del archivo 01 y `export_diagnostic_zip` de la
  tarea 4 de este archivo) **también está mockeado** aquí. Si falta uno, la
  app rota en modo web (`vite dev` sin Tauri).

**Acceptance.** El archivo client.ts pasa Prettier sin cambios y todos los
comandos registrados en `lib.rs::invoke_handler` tienen mock correspondiente.
