# Paso 04 — Auto-updater + endpoint

**Área**: 12-distribution
**Tiempo estimado**: 5-6 horas
**Dependencias**: Paso 02 (releases firmadas en GitHub)

## Qué hacemos

Configurar `tauri-plugin-updater` para que la app consulte un endpoint, descargue update firmado y se reinstale.

## Archivos

- `src-tauri/Cargo.toml` (añadir plugin)
- `src-tauri/src/lib.rs` (init plugin)
- `src-tauri/tauri.conf.json` (config + pubkey)
- `src-tauri/capabilities/default.json` (permisos)
- `scripts/generate-latest-json.mjs` (genera latest.json post-build)
- `web/public/updates/latest.json` (sirve la web)
- `src/features/settings/components/check-updates-button.tsx` (UI)

## Cómo

### 1. Instalar plugin

```bash
npm i @tauri-apps/plugin-updater
```

```toml
# src-tauri/Cargo.toml
tauri-plugin-updater = "2"
```

### 2. Generar keypair para firma de updates

Esto es DISTINTO del code-signing cert. El updater verifica una firma adicional propia de Tauri.

```bash
npm run tauri signer generate -- -w cleartool-updater.key
```

Outputs:
- `cleartool-updater.key` — clave privada (**guardar en GitHub Secrets**, no comitear).
- `cleartool-updater.key.pub` — clave pública (**comitear**).

Añadir a `.gitignore`:
```
cleartool-updater.key
```

Guardar la privada en GitHub Secrets como `TAURI_SIGNING_PRIVATE_KEY` (contenido del .key) y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` si pusiste contraseña.

### 3. Configurar tauri.conf.json

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://cleartool.app/updates/{{target}}/{{current_version}}/{{arch}}/latest.json",
        "https://github.com/joowy/cleartool/releases/latest/download/latest.json"
      ],
      "dialog": false,
      "pubkey": "<contenido de cleartool-updater.key.pub aquí>"
    }
  }
}
```

`dialog: false` porque queremos controlar el flujo nosotros desde React (mejor UX).

### 4. Inicializar el plugin

```rust
// src-tauri/src/lib.rs
.plugin(tauri_plugin_updater::Builder::new().build())
```

### 5. Permisos en capabilities

```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    // ... existentes
    "updater:default"
  ]
}
```

### 6. Workflow CI: generar latest.json + signature

Editar `.github/workflows/release.yml` añadiendo paso de firma updater + generación de latest.json:

```yaml
      - name: Sign for updater
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        run: |
          # tauri ya firma como parte de tauri build cuando los env vars están presentes.
          # Genera los .sig en target/release/bundle/nsis/

      - name: Generate latest.json
        run: node scripts/generate-latest-json.mjs ${{ github.ref_name }}

      - name: Add latest.json to release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          files: |
            latest.json
```

Script:

```js
// scripts/generate-latest-json.mjs
import fs from "node:fs";
import path from "node:path";

const tag = process.argv[2];
const version = tag.replace(/^v/, "");

const bundleDir = "src-tauri/target/release/bundle/nsis";
const installer = fs.readdirSync(bundleDir).find(f => f.endsWith("-setup.exe"));
const sigFile = `${installer}.sig`;

const sig = fs.readFileSync(path.join(bundleDir, sigFile), "utf8").trim();

const latest = {
  version,
  notes: "Ver changelog en GitHub Releases.",
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: sig,
      url: `https://github.com/joowy/cleartool/releases/download/${tag}/${installer}`,
    },
  },
};

fs.writeFileSync("latest.json", JSON.stringify(latest, null, 2));
console.log("latest.json generado:", latest);
```

### 7. UI para check + install

```tsx
// src/features/settings/components/check-updates-button.tsx
import { useState } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Button } from "../../../components/ui/button";
import { toast } from "../../../lib/toast";
import { Download } from "lucide-react";

export function CheckUpdatesButton() {
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);

  const handleCheck = async () => {
    setChecking(true);
    try {
      const update = await check();
      if (!update) {
        toast.info("Ya tienes la última versión.");
        return;
      }
      const wantInstall = confirm(
        `Nueva versión v${update.version} disponible.\n\nNotas:\n${update.body}\n\n¿Descargar e instalar?`
      );
      if (!wantInstall) return;

      setInstalling(true);
      let downloaded = 0;
      let total = 0;

      await update.downloadAndInstall((evt) => {
        switch (evt.event) {
          case "Started":
            total = evt.data.contentLength ?? 0;
            toast.info(`Descargando ${(total / 1e6).toFixed(1)} MB...`);
            break;
          case "Progress":
            downloaded += evt.data.chunkLength;
            // Opcional: actualizar progress en UI
            break;
          case "Finished":
            toast.success("Descarga completada. Relanzando...");
            break;
        }
      });
      await relaunch();
    } catch (err) {
      toast.error("No se pudo comprobar updates", err);
    } finally {
      setChecking(false);
      setInstalling(false);
    }
  };

  return (
    <Button
      variant="outline"
      onClick={handleCheck}
      disabled={checking || installing}
    >
      <Download className="h-4 w-4 mr-2" />
      {checking ? "Comprobando..." : installing ? "Instalando..." : "Comprobar actualizaciones"}
    </Button>
  );
}
```

Integrar en Settings → General.

### 8. Auto-check al arrancar

```tsx
// src/components/layout/app-shell.tsx
useEffect(() => {
  const settings = useAppStore.getState().settings;
  if (settings?.behavior.checkUpdatesOnStart) {
    void (async () => {
      try {
        const update = await check();
        if (update) {
          toast.info(`Actualización disponible: v${update.version}`, {
            action: {
              label: "Instalar",
              onClick: () => /* ... */,
            },
          });
        }
      } catch { /* silent fail */ }
    })();
  }
}, []);
```

### 9. Endpoint web (paso 06 lo expande)

Para que el primer endpoint funcione, la web `cleartool.app` debe servir `/updates/{target}/{current_version}/{arch}/latest.json` apuntando al mismo `latest.json` del último release. Plan simple: redirect 302 a GitHub.

```nginx
# Vercel / Cloudflare Pages redirect
location /updates/ {
  return 302 https://github.com/joowy/cleartool/releases/latest/download/latest.json;
}
```

## Criterio de done

- [ ] Plugin updater inicializado.
- [ ] Keypair generado, pubkey commiteada, privkey en GitHub Secrets.
- [ ] `latest.json` generado en CI con signature válida.
- [ ] Endpoint primario o GitHub fallback responde 200 OK.
- [ ] Botón "Comprobar actualizaciones" funciona.
- [ ] Auto-check al arrancar (configurable en settings).
- [ ] Test E2E: instalar v0.9-rc1 → tag v1.0.0 → app detecta update → descarga + instala + relanza.
