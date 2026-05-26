# Paso 06 — Disclaimers UI + user catalog merge

**Área**: 04-debloat-catalog
**Tiempo estimado**: 4-5 horas
**Dependencias**: Paso 01 (schema con disclaimerLevel)

## Qué hacemos

Dos cosas relacionadas:
1. UI: modales de confirmación con `disclaimerLevel` y `disclaimerText` antes de aplicar packages destructivos.
2. Backend: cargar `bloatware-catalog.user.json` opcional desde `%APPDATA%\ClearTool\catalogs\` y mergear.

## Por qué

- **Disclaimers**: el catálogo ya marca riesgo, pero el usuario power-user que entra a "Total" no lee cada entry. Los packages con consecuencias dramáticas (Edge, Store, OneDrive, AV OEM) necesitan modal explícito antes de aplicar.
- **User catalog**: la comunidad y el propio usuario quieren añadir entries propias sin compilar.

## Archivos que tocamos

- `src/features/debloat/components/disclaimer-modal.tsx` (nuevo)
- `src/features/debloat/debloat-page.tsx` (modificado)
- `src-tauri/src/domain/catalog.rs` (modificado para soportar user merge)
- `src-tauri/src/core/config.rs` (asegurar paths para user catalog)

## Cómo

### 1. Disclaimer modal

```tsx
// src/features/debloat/components/disclaimer-modal.tsx
import { AlertTriangle, AlertOctagon, Info } from "lucide-react";
import { Button } from "../../../components/ui/button";

interface DisclaimerModalProps {
  open: boolean;
  entries: Array<{
    id: string;
    displayName: string;
    disclaimerLevel: "info" | "warning" | "danger";
    disclaimerText: string;
  }>;
  onConfirm: () => void;
  onCancel: () => void;
}

export function DisclaimerModal({ open, entries, onConfirm, onCancel }: DisclaimerModalProps) {
  if (!open || entries.length === 0) return null;

  const maxLevel = entries.reduce<"info" | "warning" | "danger">(
    (acc, e) => (
      e.disclaimerLevel === "danger" ? "danger" :
      e.disclaimerLevel === "warning" && acc !== "danger" ? "warning" :
      acc
    ),
    "info"
  );

  const Icon = maxLevel === "danger" ? AlertOctagon : maxLevel === "warning" ? AlertTriangle : Info;
  const iconColor = maxLevel === "danger" ? "text-signal-red" : maxLevel === "warning" ? "text-yellow-400" : "text-signal-cyan";

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
      <div className="bg-card border border-border rounded-lg max-w-2xl w-full p-6 flex flex-col gap-4">
        <div className="flex items-center gap-3">
          <Icon className={`h-8 w-8 ${iconColor}`} />
          <div>
            <h2 className="text-xl font-bold">
              {entries.length === 1
                ? "Antes de continuar..."
                : `${entries.length} packages con advertencias`}
            </h2>
            <p className="text-sm text-muted-foreground">
              Lee con cuidado. Estas acciones tienen consecuencias específicas.
            </p>
          </div>
        </div>

        <div className="max-h-80 overflow-auto space-y-3">
          {entries.map((e) => (
            <div key={e.id} className="border border-border rounded p-3">
              <div className="font-medium">{e.displayName}</div>
              <p className="text-sm text-muted-foreground mt-1">{e.disclaimerText}</p>
            </div>
          ))}
        </div>

        <div className="flex gap-2 justify-end">
          <Button variant="outline" onClick={onCancel}>Cancelar</Button>
          <Button variant={maxLevel === "danger" ? "destructive" : "default"} onClick={onConfirm}>
            Entiendo, continuar
          </Button>
        </div>
      </div>
    </div>
  );
}
```

### 2. Integración en `debloat-page.tsx`

```tsx
const [showDisclaimer, setShowDisclaimer] = useState(false);
const [pendingApply, setPendingApply] = useState<RemoveBloatwareInput | null>(null);

const handleApply = () => {
  // Detectar entries seleccionadas con disclaimer
  const dangerous = catalog
    .filter(e => selected.has(e.id) && e.disclaimerLevel)
    .map(e => ({
      id: e.id,
      displayName: e.displayName,
      disclaimerLevel: e.disclaimerLevel!,
      disclaimerText: e.disclaimerText ?? "",
    }));

  const input: RemoveBloatwareInput = {
    entryIds: [...selected],
    dryRun,
    createRestorePoint: true,
    applyPolicies: true,
    disableServices: true,
  };

  if (dangerous.length > 0) {
    setPendingApply(input);
    setShowDisclaimer(true);
  } else {
    removeMutation.mutate(input);
  }
};

// En JSX:
<DisclaimerModal
  open={showDisclaimer}
  entries={dangerousEntries}
  onConfirm={() => {
    setShowDisclaimer(false);
    if (pendingApply) removeMutation.mutate(pendingApply);
  }}
  onCancel={() => setShowDisclaimer(false)}
/>
```

### 3. User catalog merge

#### Backend

```rust
// src-tauri/src/domain/catalog.rs (modificar load_bloatware_catalog)

use crate::core::config;

pub fn load_bloatware_catalog() -> AppResult<Vec<BloatwareEntry>> {
    let embedded: BloatwareCatalogFile = serde_json::from_str(EMBEDDED_BLOATWARE_JSON)
        .map_err(|e| AppError::Catalog(format!("embedded: {}", e)))?;

    let user_path = config::user_catalogs_dir().join("bloatware-catalog.user.json");
    let user: Option<BloatwareCatalogFile> = if user_path.exists() {
        match std::fs::read_to_string(&user_path) {
            Ok(s) => match serde_json::from_str(&s) {
                Ok(c) => Some(c),
                Err(e) => {
                    log::warn!("user catalog inválido: {}", e);
                    None
                }
            },
            Err(_) => None,
        }
    } else { None };

    let mut by_id: std::collections::HashMap<String, BloatwareEntry> = embedded
        .entries
        .into_iter()
        .map(|e| (e.id.clone(), e))
        .collect();

    if let Some(u) = user {
        for entry in u.entries {
            // User override O añade
            by_id.insert(entry.id.clone(), entry);
        }
    }

    Ok(by_id.into_values().collect())
}
```

#### Crear user catalog dir si no existe

```rust
// src-tauri/src/core/config.rs (añadir)

pub fn user_catalogs_dir() -> std::path::PathBuf {
    let dir = app_data_dir().join("catalogs");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}
```

#### UI: badge "Custom" en entries user

```tsx
// debloat-page.tsx en la tabla:
{entry.customSource && (
  <Badge variant="outline" className="ml-1">Custom</Badge>
)}
```

Para que el frontend sepa cuáles son user, añadir un campo `customSource: boolean` al `BloatwareEntry` en backend al hacer el merge:

```rust
struct BloatwareEntry {
    // ...
    #[serde(default)]
    pub custom_source: bool,    // ← NUEVO, populado al mergear
}
```

Y poblar en el merge:
```rust
for mut entry in u.entries {
    entry.custom_source = true;
    by_id.insert(entry.id.clone(), entry);
}
```

### 4. Ejemplo de user catalog

Documentar en README de la skill un ejemplo:

```json
// %APPDATA%\ClearTool\catalogs\bloatware-catalog.user.json
{
  "schemaVersion": 2,
  "entries": [
    {
      "id": "custom-my-corp-app",
      "displayName": "MyCorp Internal App",
      "description": "App interna de mi empresa que ya no uso.",
      "category": "third-party-bundled",
      "risk": "low",
      "consequences": ["Pierdes acceso a la app."],
      "removalMethod": "uninstaller-string",
      "displayNamePattern": "MyCorp*",
      "publisher": "MyCorp Inc.",
      "reversible": false,
      "reverseRecipe": { "kind": "noop", "reason": "Pedir reinstalación a IT." },
      "presets": ["total"],
      "verified": "2026-05-25"
    }
  ]
}
```

## Criterio de done

- [ ] `<DisclaimerModal>` muestra modal con icono según `disclaimerLevel`.
- [ ] Click "Aplicar" en debloat con entries danger → muestra modal antes de mutate.
- [ ] Cancelar el modal NO ejecuta la mutation.
- [ ] User catalog se mergea sobre el embebido.
- [ ] User overrides funcionan (mismo `id` en user → reemplaza embedded).
- [ ] Entries user tienen badge "Custom" en UI.
- [ ] Si user catalog tiene JSON inválido, se ignora con log warning (no crash).
- [ ] Settings → "Abrir carpeta de catálogos" añadido (botón que llama `tauri.opener.openPath(user_catalogs_dir)`).
