# Paso 04 — Medición de impacto + frontend

**Área**: 05-startup-manager
**Tiempo estimado**: 4-5 horas
**Dependencias**: Pasos 02 y 03

## Qué hacemos

1. Leer Event Log de "Diagnostics-Performance" para sacar impact por proceso al logon.
2. Asignar impact a cada `StartupEntry` matcheando por exe path o nombre.
3. UI con agrupación por nivel de impacto.

## Cómo

### 1. Lectura del Event Log

```rust
// src-tauri/src/platform/startup.rs (añadir)

const BOOT_PERF_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-WinEvent -LogName 'Microsoft-Windows-Diagnostics-Performance/Operational' -MaxEvents 10 |
  Where-Object { $_.Id -eq 100 -or $_.Id -eq 200 } |
  ForEach-Object {
    $xml = [xml]$_.ToXml()
    $impacts = @{}
    foreach ($data in $xml.Event.EventData.Data) {
      if ($data.Name -like '*Application*' -or $data.Name -like '*Service*') {
        # Skip
      }
    }
    [PSCustomObject]@{
      TimeCreated = $_.TimeCreated.ToString('o')
      Id = $_.Id
      XML = $xml.OuterXml
    }
  } | ConvertTo-Json -Compress
"#;

#[derive(Debug)]
pub struct StartupImpactSnapshot {
    pub measured_at: String,
    pub per_exe_ms: std::collections::HashMap<String, u32>,
}

pub fn read_startup_impact_snapshot() -> Option<StartupImpactSnapshot> {
    // En lugar de leer XML complejo, una aproximación más simple:
    // Get-WinEvent con propertyselectors específicos.
    // Para v1.0, devolver None y dejar como "Unknown".
    // Implementar parser robusto en M5 polish.
    None
}
```

### 2. Heurística rápida sin Event Log

Si no podemos medir, asignamos por categoría:
- `Updater`, `Background` → `Low`
- `Launcher`, `Widget`, `CloudSync` → `Medium`
- `Communication`, `Media` → `High`
- `Driver`, `Security` → `High` (siempre se hacen notar al boot)

```rust
fn estimate_impact(entry: &StartupEntry) -> StartupImpact {
    match entry.category {
        StartupCategory::Updater | StartupCategory::Background => StartupImpact::Low,
        StartupCategory::Launcher | StartupCategory::Widget | StartupCategory::CloudSync => StartupImpact::Medium,
        StartupCategory::Communication | StartupCategory::Media | StartupCategory::Driver | StartupCategory::Security => StartupImpact::High,
        _ => StartupImpact::Unknown,
    }
}
```

### 3. Clasificación heurística por nombre

```rust
fn classify_by_name(name: &str, cmd: &str) -> StartupCategory {
    let n = name.to_lowercase();
    let c = cmd.to_lowercase();
    let combined = format!("{} {}", n, c);
    if combined.contains("update") { return StartupCategory::Updater; }
    if combined.contains("onedrive") || combined.contains("dropbox") || combined.contains("googledrive") {
        return StartupCategory::CloudSync;
    }
    if combined.contains("discord") || combined.contains("teams") || combined.contains("slack") {
        return StartupCategory::Communication;
    }
    if combined.contains("spotify") || combined.contains("itunes") {
        return StartupCategory::Media;
    }
    if combined.contains("nvidia") || combined.contains("amd") || combined.contains("realtek") {
        return StartupCategory::Driver;
    }
    StartupCategory::UserApp
}
```

Aplicar en `list_all`:

```rust
pub fn list_all() -> AppResult<Vec<StartupEntry>> {
    let mut entries = Vec::new();
    entries.extend(platform::list_registry_run_entries());
    entries.extend(platform::list_startup_folder_entries());
    entries.extend(platform::list_logon_scheduled_tasks());
    entries.extend(platform::list_automatic_services());

    for e in &mut entries {
        if matches!(e.category, StartupCategory::Unknown) {
            e.category = classify_by_name(&e.display_name, &e.command);
        }
        e.impact = estimate_impact(e);
    }
    Ok(entries)
}
```

### 4. IPC

```rust
// src-tauri/src/ipc/startup.rs
#[tauri::command] pub async fn list_startup() -> AppResult<Vec<StartupEntry>> {
    crate::domain::startup::list_all()
}
#[tauri::command] pub async fn disable_startup(id: String) -> AppResult<()> {
    crate::domain::startup::disable_startup(&id)
}
#[tauri::command] pub async fn enable_startup(id: String) -> AppResult<()> {
    crate::domain::startup::enable_startup(&id)
}
```

### 5. Frontend

```tsx
// src/features/startup/startup-page.tsx
import { useQuery, useMutation } from "@tanstack/react-query";
import { listStartup, disableStartup, enableStartup } from "../../api";
import { toast } from "../../lib/toast";

export function StartupPage() {
  const { data: entries = [], refetch } = useQuery({
    queryKey: ["startup"],
    queryFn: listStartup,
  });

  // Agrupado por impact
  const grouped = {
    high: entries.filter(e => e.impact === "high" && e.enabled),
    medium: entries.filter(e => e.impact === "medium" && e.enabled),
    low: entries.filter(e => e.impact === "low" && e.enabled),
    disabled: entries.filter(e => !e.enabled),
  };

  return (
    <div className="p-6 flex flex-col gap-4">
      <h2 className="text-2xl font-bold">Arranque automático</h2>
      <p className="text-muted-foreground text-sm">
        {entries.length} entradas · {grouped.high.length} alto impacto
      </p>

      <ImpactGroup title="⚠ Alto impacto" items={grouped.high} level="high" onToggle={refetch} />
      <ImpactGroup title="Medio impacto" items={grouped.medium} level="medium" onToggle={refetch} />
      <ImpactGroup title="Bajo impacto" items={grouped.low} level="low" onToggle={refetch} />
      {grouped.disabled.length > 0 && (
        <ImpactGroup title="Deshabilitadas" items={grouped.disabled} level="disabled" onToggle={refetch} />
      )}
    </div>
  );
}

function ImpactGroup({ title, items, level, onToggle }: {
  title: string; items: StartupEntry[]; level: string; onToggle: () => void;
}) {
  if (items.length === 0) return null;
  return (
    <details open={level === "high"} className="border border-border rounded-lg">
      <summary className="cursor-pointer p-3 font-medium">{title} ({items.length})</summary>
      <div className="divide-y divide-border/50">
        {items.map(e => <StartupRow key={e.id} entry={e} onToggle={onToggle} />)}
      </div>
    </details>
  );
}

function StartupRow({ entry, onToggle }: { entry: StartupEntry; onToggle: () => void }) {
  const toggleMut = useMutation({
    mutationFn: () => entry.enabled ? disableStartup(entry.id) : enableStartup(entry.id),
    onSuccess: () => {
      toast.success(`${entry.displayName} ${entry.enabled ? "deshabilitada" : "habilitada"}`);
      onToggle();
    },
    onError: (err) => toast.error("Error", err),
  });

  return (
    <div className="px-4 py-2 flex items-center gap-3 text-sm">
      <div className="flex-1">
        <div className="font-medium">{entry.displayName}</div>
        <div className="text-xs text-muted-foreground truncate">{entry.command}</div>
        <div className="text-xs text-muted-foreground">
          Origen: {originLabel(entry.origin)} · Categoría: {entry.category}
        </div>
      </div>
      <Button
        size="sm"
        variant={entry.enabled ? "outline" : "default"}
        onClick={() => toggleMut.mutate()}
        disabled={toggleMut.isPending}
      >
        {entry.enabled ? "Deshabilitar" : "Habilitar"}
      </Button>
    </div>
  );
}

function originLabel(o: StartupOrigin): string {
  switch (o.kind) {
    case "registry": return `Registry (${o.hive})`;
    case "startupFolder": return "Carpeta Startup";
    case "scheduledTask": return "Tarea programada";
    case "service": return "Servicio";
    case "uwpAutoStart": return "UWP";
  }
}
```

## Criterio de done

- [ ] Página `/startup` lista entries agrupadas por impact.
- [ ] Toggle disable/enable funciona para los 4 orígenes.
- [ ] Toast confirma cada acción.
- [ ] Refetch automático tras toggle.
- [ ] Test manual: deshabilitar OneDrive autostart → reinicio → OneDrive no arranca → reactivar → arranca.
