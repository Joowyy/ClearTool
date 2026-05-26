# 11 — Auto-update (Tauri Updater + manifest server)

> **Posición:** 11/14.
> **Dependencias:** [10-DISTRIBUCION](10-DISTRIBUCION.md) en verde.
> **Output:** auto-update funcional opt-in, manifest server propio en GitHub Pages (o equivalente), canales `stable` / `beta`, firma de updates con clave Ed25519.

---

## 1. Resumen ejecutivo

Sin auto-update, cada release requiere que el usuario descargue manualmente. Para una herramienta open-source con tracción, eso degrada la base de usuarios actualizados.

ClearTool implementa **Tauri Updater** con:

1. **Opt-in** — el usuario decide si se buscan updates. Setting ya existe (`behavior.checkUpdatesOnStart`).
2. **Firma Ed25519** — Tauri exige updates firmados con clave privada. La pública vive embebida en el binario.
3. **Manifest hosted** — JSON estático servido desde `https://updates.cleartool.app/manifest.json`.
4. **Hosting:** GitHub Pages con un repo dedicado o subdominio CNAME.
5. **Canales:** `stable` (default) y `beta` (para early adopters; configurable en Ajustes → Avanzado).

---

## 2. Diagnóstico

### 2.1 Por qué Tauri Updater y no algo custom

| Opción | Pros | Contras |
|---|---|---|
| Tauri Updater (`tauri-plugin-updater`) | Oficial, firma builtin, diffing builtin, UX integrada | Solo soporta full bundle (no patches incrementales) |
| Squirrel-style custom | Patches binarios pequeños | Implementación compleja, mantenimiento |
| Velopack (sucesor moderno) | Patches, rollback | Aún ecosistema joven |

**Decisión:** Tauri Updater. Más simple, más seguro, suficiente para v1.0.

### 2.2 Hosting del manifest

| Opción | Coste | Pro/Contra |
|---|---|---|
| GitHub Pages | Gratis | Latencia OK, requiere repo dedicado |
| Cloudflare Pages | Gratis | CDN global, custom domain fácil |
| S3 + CloudFront | ~$1/mes | Más control, requiere AWS account |
| Servidor propio (VPS) | ~$5/mes | Innecesario para JSON estático |

**Decisión:** GitHub Pages en repo `cleartool/cleartool-updates`. Con CNAME `updates.cleartool.app`.

### 2.3 Tamaño del bundle de update

NSIS `.exe` ≈ 8-15 MB. Bajar uno entero por update no es ideal para usuarios con conexiones limitadas, pero es aceptable para v1.0. Patches incrementales = v1.2+.

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| Tauri Updater plugin oficial | Battle-tested + firma builtin |
| Clave Ed25519 generada localmente, NUNCA en repo | La privada se guarda en password manager + offline backup |
| Manifest JSON estático en GitHub Pages | Cero costos, control versionado |
| 2 canales (`stable`, `beta`) seleccionables | Sin alpha — beta ya es opt-in suficiente |
| Check al startup (opt-in) + check manual desde Settings | Doble entry-point |
| Update **no se aplica automáticamente** — se notifica + usuario decide | Coherencia con principio de transparencia |
| Notas de release embebidas en el manifest | UX en el modal de update |

---

## 4. Generación de claves Tauri

### 4.1 Comando

```bash
# Una sola vez, fuera del repo:
npx @tauri-apps/cli signer generate -w ~/.cleartool/updater-private.key
```

Output:
- `~/.cleartool/updater-private.key` — **NUNCA al repo**. Backup offline.
- Clave pública (base64) imprimida — copiar a `tauri.conf.json` → `plugins.updater.pubkey`.

### 4.2 Storage de la clave privada

| Donde | Comentario |
|---|---|
| `~/.cleartool/updater-private.key` | Solo en la máquina del maintainer |
| Password manager (1Password, Bitwarden) | Encriptada, sincronizada |
| Backup offline (USB + caja fuerte física) | Para disaster recovery |
| GitHub Secrets `TAURI_SIGNING_PRIVATE_KEY` | Usado por CI release workflow |

**Crítico:** si la clave se pierde, los usuarios actuales no podrán recibir updates firmados con esta clave. Mitigación: rotación de clave = release que invalida updaters viejos (documentar y notificar).

### 4.3 Variables de entorno en CI

```yaml
# .github/workflows/release.yml
env:
  TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```

---

## 5. Manifest JSON

### 5.1 Estructura

**Archivo:** `cleartool-updates/manifest.json` (en el repo dedicado de updates)

```json
{
  "version": "1.0.1",
  "notes": "## ClearTool 1.0.1\n\nCorrecciones menores y mejoras de estabilidad.\n\n- Fix de bug en revert de tweaks HKLM.\n- Mejora performance scan de C:\\.\n- Texto del onboarding mejorado.",
  "pub_date": "2026-06-15T10:30:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "AAAAxxxxBASE64SIGNATURE",
      "url": "https://github.com/cleartool/cleartool/releases/download/v1.0.1/ClearTool_1.0.1_x64-setup.exe"
    }
  }
}
```

### 5.2 Canales

Servidos como rutas distintas:

- `https://updates.cleartool.app/stable/manifest.json` — público, default.
- `https://updates.cleartool.app/beta/manifest.json` — para opt-in.

El switch entre canales se hace via setting; la app lee el setting y consulta el endpoint correspondiente.

### 5.3 Repo `cleartool-updates`

Estructura:

```
cleartool-updates/
  index.html              ← landing simple "ClearTool updates server"
  manifest.json           ← symlink/copia del stable más reciente
  stable/
    manifest.json
    archive/
      1.0.0.json
      1.0.1.json
  beta/
    manifest.json
    archive/
      1.0.1-beta.1.json
  CNAME                   ← contiene "updates.cleartool.app"
  README.md
```

### 5.4 Workflow de actualización del manifest

**Archivo (en el repo principal):** `.github/workflows/publish-update.yml`

```yaml
name: Publish update manifest

on:
  release:
    types: [published]

jobs:
  update-manifest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: cleartool/cleartool-updates
          token: ${{ secrets.UPDATES_REPO_PAT }}
          path: updates

      - name: Determine channel
        id: chan
        run: |
          if [[ "${{ github.ref_name }}" == *"-beta"* ]]; then
            echo "channel=beta" >> $GITHUB_OUTPUT
          else
            echo "channel=stable" >> $GITHUB_OUTPUT
          fi

      - name: Download release artifacts
        run: |
          mkdir -p artifacts
          gh release download "${{ github.ref_name }}" \
            -R cleartool/cleartool \
            -p '*-setup.exe' -p '*-setup.exe.sig' \
            -D artifacts
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate manifest
        run: |
          VERSION="${{ github.ref_name }}"
          VERSION="${VERSION#v}"  # quitar el "v" inicial
          NOTES=$(gh release view "${{ github.ref_name }}" -R cleartool/cleartool --json body -q .body)
          SIG=$(cat artifacts/*.sig)
          URL="https://github.com/cleartool/cleartool/releases/download/${{ github.ref_name }}/ClearTool_${VERSION}_x64-setup.exe"
          PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

          jq -n \
            --arg ver "$VERSION" \
            --arg notes "$NOTES" \
            --arg date "$PUB_DATE" \
            --arg sig "$SIG" \
            --arg url "$URL" \
            '{
              version: $ver,
              notes: $notes,
              pub_date: $date,
              platforms: {
                "windows-x86_64": {
                  signature: $sig,
                  url: $url
                }
              }
            }' > updates/${{ steps.chan.outputs.channel }}/manifest.json

          cp updates/${{ steps.chan.outputs.channel }}/manifest.json \
             updates/${{ steps.chan.outputs.channel }}/archive/${VERSION}.json

          # Stable también actualiza el root manifest (default)
          if [[ "${{ steps.chan.outputs.channel }}" == "stable" ]]; then
            cp updates/stable/manifest.json updates/manifest.json
          fi
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Commit and push
        run: |
          cd updates
          git config user.name "ClearTool Bot"
          git config user.email "bot@cleartool.app"
          git add .
          git commit -m "Publish ${{ github.ref_name }}"
          git push
```

---

## 6. Implementación en la app

### 6.1 Dependencia

**Archivo:** `src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-updater = "2"
```

**Archivo:** `package.json`

```json
{
  "dependencies": {
    "@tauri-apps/plugin-updater": "^2"
  }
}
```

### 6.2 Registrar plugin

**Archivo:** `src-tauri/src/lib.rs`

```rust
pub fn run() {
    crate::core::settings::init();
    crate::domain::catalog::validate_all_at_startup();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        // ... resto de plugins
        .invoke_handler(tauri::generate_handler![
            // ... handlers existentes
            ipc::updater::check_for_update,
            ipc::updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error tauri");
}
```

### 6.3 Comandos IPC

**Archivo:** `src-tauri/src/ipc/updater.rs` (nuevo)

```rust
use crate::core::AppResult;
use crate::core::settings;
use serde::Serialize;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
}

#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> AppResult<UpdateInfo> {
    let endpoint = endpoint_for_channel(&settings::get().advanced.update_channel);
    let updater = app.updater_builder()
        .endpoints(vec![endpoint.parse().unwrap()])
        .build()
        .map_err(|e| crate::core::AppError::Updater(format!("build: {}", e)))?;

    match updater.check().await {
        Ok(Some(update)) => Ok(UpdateInfo {
            available: true,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: Some(update.version.clone()),
            notes: update.body.clone(),
            pub_date: update.date.map(|d| d.to_string()),
        }),
        Ok(None) => Ok(UpdateInfo {
            available: false,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: None,
            notes: None,
            pub_date: None,
        }),
        Err(e) => Err(crate::core::AppError::Updater(format!("check: {}", e))),
    }
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> AppResult<()> {
    let endpoint = endpoint_for_channel(&settings::get().advanced.update_channel);
    let updater = app.updater_builder()
        .endpoints(vec![endpoint.parse().unwrap()])
        .build()
        .map_err(|e| crate::core::AppError::Updater(format!("build: {}", e)))?;

    if let Some(update) = updater.check().await.ok().flatten() {
        update.download_and_install(|_chunk, _total| {}, || {})
            .await
            .map_err(|e| crate::core::AppError::Updater(format!("install: {}", e)))?;
    }
    Ok(())
}

fn endpoint_for_channel(channel: &str) -> String {
    match channel {
        "beta" => "https://updates.cleartool.app/beta/manifest.json".to_string(),
        _ => "https://updates.cleartool.app/stable/manifest.json".to_string(),
    }
}
```

### 6.4 Añadir `update_channel` a Settings

**Archivo:** `src-tauri/src/models/settings.rs`

```rust
pub struct AdvancedSettings {
    pub log_level: String,
    pub audit_log_max_mb: u32,
    pub diagnostic_mode: bool,
    pub update_channel: String,   // "stable" | "beta"
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            log_level: "info".into(),
            audit_log_max_mb: 10,
            diagnostic_mode: false,
            update_channel: "stable".into(),
        }
    }
}
```

### 6.5 Frontend: hook + modal

**Archivo:** `src/features/updates/use-updates.ts` (nuevo)

```ts
import { useQuery, useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion: string | null;
  notes: string | null;
  pubDate: string | null;
}

export function useCheckUpdate(enabled: boolean) {
  return useQuery({
    queryKey: ["updates", "check"],
    queryFn: () => invoke<UpdateInfo>("check_for_update"),
    enabled,
    staleTime: 30 * 60 * 1000, // 30 min
  });
}

export function useInstallUpdate() {
  return useMutation({
    mutationFn: () => invoke<void>("install_update"),
  });
}
```

**Archivo:** `src/features/updates/update-notification.tsx` (nuevo)

```tsx
import { useState } from "react";
import { useCheckUpdate, useInstallUpdate } from "./use-updates";
import { useSettings } from "../settings/use-settings";

export function UpdateNotification() {
  const settings = useSettings().data;
  const enabled = !!settings?.behavior.checkUpdatesOnStart;
  const { data: info } = useCheckUpdate(enabled);
  const install = useInstallUpdate();
  const [dismissed, setDismissed] = useState(false);

  if (!info?.available || dismissed) return null;

  return (
    <div className="fixed bottom-4 right-4 max-w-sm bg-card border border-primary/40 rounded-lg p-4 shadow-lg z-40">
      <div className="flex items-start justify-between mb-2">
        <h4 className="font-bold">Actualización disponible</h4>
        <button onClick={() => setDismissed(true)} className="text-muted-foreground">×</button>
      </div>
      <p className="text-sm text-muted-foreground mb-2">
        ClearTool {info.latestVersion} (tenés {info.currentVersion}).
      </p>
      {info.notes && (
        <div className="text-xs bg-background/40 rounded p-2 mb-3 max-h-32 overflow-auto whitespace-pre-wrap">
          {info.notes}
        </div>
      )}
      <div className="flex gap-2">
        <button
          className="px-3 py-1 bg-primary text-primary-foreground rounded text-sm disabled:opacity-50"
          onClick={() => install.mutate()}
          disabled={install.isPending}
        >
          {install.isPending ? "Instalando..." : "Instalar y reiniciar"}
        </button>
        <button
          className="px-3 py-1 border border-border rounded text-sm"
          onClick={() => setDismissed(true)}
        >
          Más tarde
        </button>
      </div>
    </div>
  );
}
```

Montar en `app-shell.tsx`.

### 6.6 Settings — UI para canal

En el tab "Avanzado" de Settings (creado en [07-SETTINGS](07-SETTINGS.md)):

```tsx
<div>
  <label className="text-sm font-medium">Canal de actualizaciones</label>
  <div className="flex gap-3 mt-1">
    <label className="flex items-center gap-1">
      <input
        type="radio"
        checked={settings.advanced.updateChannel === "stable"}
        onChange={() => onPatch({ advanced: { ...settings.advanced, updateChannel: "stable" } })}
      />
      Estable
    </label>
    <label className="flex items-center gap-1">
      <input
        type="radio"
        checked={settings.advanced.updateChannel === "beta"}
        onChange={() => onPatch({ advanced: { ...settings.advanced, updateChannel: "beta" } })}
      />
      Beta (acceso temprano)
    </label>
  </div>
  <p className="text-xs text-muted-foreground mt-1">
    El canal beta recibe versiones pre-release con features nuevas pero menos probadas.
  </p>
</div>
```

---

## 7. Permisos Tauri

**Archivo:** `src-tauri/capabilities/default.json`

Añadir:

```json
{
  "permissions": [
    // ...
    "updater:default",
    "updater:allow-check",
    "updater:allow-download",
    "updater:allow-install"
  ]
}
```

---

## 8. CSP

**Archivo:** `src-tauri/tauri.conf.json`

Ya incluye en `connect-src`:

```
https://updates.cleartool.app
```

Si los `url` del manifest apuntan a `github.com/.../releases/download/`, añadir:

```
https://github.com https://*.githubusercontent.com
```

---

## 9. Tests

### 9.1 Test del endpoint resolver

```rust
#[test]
fn endpoint_para_canal_stable() {
    let e = cleartool::ipc::updater::endpoint_for_channel("stable");
    assert!(e.contains("stable/manifest.json"));
}

#[test]
fn endpoint_para_canal_beta() {
    let e = cleartool::ipc::updater::endpoint_for_channel("beta");
    assert!(e.contains("beta/manifest.json"));
}

#[test]
fn endpoint_desconocido_cae_en_stable() {
    let e = cleartool::ipc::updater::endpoint_for_channel("alpha");
    assert!(e.contains("stable/manifest.json"));
}
```

### 9.2 Test manual de update flow

1. Build local `v1.0.0` con keys generadas.
2. Instalar en VM.
3. Generar `v1.0.1` local + crear manifest manual con signature.
4. Servir manifest local con `python -m http.server` en `localhost:8080`.
5. Override endpoint temporal (env var de debug) apuntando a localhost.
6. Lanzar app → debe detectar update → mostrar notification.
7. Click "Instalar y reiniciar" → descarga + verifica firma + relanza con nueva versión.
8. Confirmar en `/settings` → "Acerca de" → versión = 1.0.1.

---

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Clave privada se pierde | Backup offline obligatorio; documentar en runbook |
| Manifest server down | Tauri falla graciosamente — usuario sigue con versión actual |
| Update firma corrupta | Tauri rechaza install, usuario ve error claro |
| Update interrumpido (red caída) | Tauri lo abandona, usuario puede reintentar |
| Update sobre instalación per-machine vs per-user | Mantener per-user consistente entre versiones |
| MITM en download de update | HTTPS + firma Ed25519 (defensa en profundidad) |
| Usuario en canal beta no quiere downgrade | Documentar: cambiar a stable + reinstalar manualmente |

---

## 11. Definition of Done

- [ ] Clave Ed25519 generada, privada en password manager, pública en `tauri.conf.json`.
- [ ] `cleartool-updates` repo creado con estructura completa + CNAME.
- [ ] DNS `updates.cleartool.app` apuntando a GitHub Pages.
- [ ] `.github/workflows/publish-update.yml` en repo principal.
- [ ] `tauri-plugin-updater` añadido como dependencia.
- [ ] `core::settings::AdvancedSettings::update_channel` añadido.
- [ ] `ipc::updater::{check_for_update, install_update}` implementados.
- [ ] Frontend: `UpdateNotification` + selector de canal en Settings.
- [ ] Permisos `updater:*` en `capabilities/default.json`.
- [ ] Test manual end-to-end con manifest local pasa.
- [ ] Documentación de rotación de claves en `docs/MAINTAINER-RUNBOOK.md`.
- [ ] Commit `feat(updates): auto-update Tauri Updater + canales + manifest server`.

---

## 12. Próximo archivo

→ [12-I18N-ONBOARDING.md](12-I18N-ONBOARDING.md) — internacionalización ES/EN, primer arranque (onboarding wizard), accesibilidad básica.
