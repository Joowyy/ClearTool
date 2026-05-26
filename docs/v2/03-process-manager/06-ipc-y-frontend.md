# Paso 06 — IPC + frontend (pestaña Procesos)

**Área**: 03-process-manager
**Tiempo estimado**: 8-10 horas
**Dependencias**: Pasos 01-05

## Qué hacemos

Exponer los comandos IPC y crear la pestaña Procesos con:
- Tabla agrupada por categoría.
- Búsqueda + filtros.
- Acciones (Cerrar, Kill, Suspend, Resume).
- Preset "Liberar para limpiar".

## Archivos que tocamos

- `src-tauri/src/ipc/processes.rs` (nuevo)
- `src-tauri/src/ipc/mod.rs` (export)
- `src-tauri/src/lib.rs` (registrar)
- `src/api/client.ts` (wrappers TS)
- `src/features/processes/processes-page.tsx` (nuevo)
- `src/features/processes/components/*.tsx` (componentes)
- `src/lib/routes.ts` (añadir ruta `/processes`)
- `src/components/layout/tab-nav.tsx` (añadir entry, OPCIONAL si sidebar viene después)

## Cómo

### 1. IPC

```rust
// src-tauri/src/ipc/processes.rs
use crate::core::AppResult;
use crate::models::process::ProcessInfo;
use crate::models::process_lock::LockingProcess;
use crate::platform::processes;

#[tauri::command]
pub async fn list_processes() -> AppResult<Vec<ProcessInfo>> {
    processes::list_processes_extended()
}

#[tauri::command]
pub async fn kill_process(pid: u32) -> AppResult<()> {
    processes::kill_process(pid)
}

#[tauri::command]
pub async fn kill_process_tree(pid: u32) -> AppResult<()> {
    processes::kill_process_tree(pid)
}

#[tauri::command]
pub async fn suspend_process(pid: u32) -> AppResult<()> {
    processes::suspend_process(pid)
}

#[tauri::command]
pub async fn resume_process(pid: u32) -> AppResult<()> {
    processes::resume_process(pid)
}

#[tauri::command]
pub async fn close_gracefully(pid: u32, timeout_ms: u32) -> AppResult<bool> {
    processes::close_gracefully(pid, timeout_ms).await
}

#[tauri::command]
pub async fn who_locks_path(path: String) -> AppResult<Vec<LockingProcess>> {
    crate::platform::process_lock::who_locks(std::path::Path::new(&path))
}

#[tauri::command]
pub async fn release_caches() -> AppResult<ReleaseReport> {
    // Implementación en Paso 06.2 (más abajo).
    processes::release_common_apps_for_cleanup().await
}
```

Registrar en `lib.rs`.

### 2. TS wrappers

```ts
// src/api/client.ts
export const listProcesses = () => invoke<ProcessInfo[]>("list_processes");
export const killProcess = (pid: number) => invoke<void>("kill_process", { pid });
export const killProcessTree = (pid: number) => invoke<void>("kill_process_tree", { pid });
export const suspendProcess = (pid: number) => invoke<void>("suspend_process", { pid });
export const resumeProcess = (pid: number) => invoke<void>("resume_process", { pid });
export const closeGracefully = (pid: number, timeoutMs: number) =>
  invoke<boolean>("close_gracefully", { pid, timeoutMs });
export const whoLocksPath = (path: string) =>
  invoke<LockingProcess[]>("who_locks_path", { path });
export const releaseCaches = () => invoke<ReleaseReport>("release_caches");
```

### 3. Página Procesos

```tsx
// src/features/processes/processes-page.tsx
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listProcesses } from "../../api";
import { ProcessGroup } from "./components/process-group";
import { ProcessFilters } from "./components/process-filters";
import { ReleaseButton } from "./components/release-button";

export function ProcessesPage() {
  const [search, setSearch] = useState("");
  const [showSystem, setShowSystem] = useState(false);
  const [onlyWithUI, setOnlyWithUI] = useState(false);

  const { data: processes = [], refetch, isFetching } = useQuery({
    queryKey: ["processes"],
    queryFn: listProcesses,
    refetchInterval: 2000,  // refresh cada 2s
    refetchIntervalInBackground: false,
  });

  // Filtrado en cliente
  const filtered = processes.filter((p) => {
    if (!showSystem && p.category === "system") return false;
    if (onlyWithUI && p.threadCount === 0) return false;
    if (search && !p.name.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  // Agrupado por categoría
  const grouped = filtered.reduce<Record<string, ProcessInfo[]>>((acc, p) => {
    if (!acc[p.category]) acc[p.category] = [];
    acc[p.category].push(p);
    return acc;
  }, {});

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Procesos</h2>
          <p className="text-muted-foreground text-sm">
            {processes.length} procesos · {Object.keys(grouped).length} categorías
          </p>
        </div>
        <ReleaseButton />
      </div>

      <ProcessFilters
        search={search} setSearch={setSearch}
        showSystem={showSystem} setShowSystem={setShowSystem}
        onlyWithUI={onlyWithUI} setOnlyWithUI={setOnlyWithUI}
        isFetching={isFetching} onRefresh={refetch}
      />

      <div className="flex-1 overflow-auto space-y-3">
        {Object.entries(grouped)
          .sort(([a], [b]) => CATEGORY_ORDER.indexOf(a) - CATEGORY_ORDER.indexOf(b))
          .map(([cat, procs]) => (
            <ProcessGroup key={cat} category={cat as ProcessCategory} processes={procs} />
          ))}
      </div>
    </div>
  );
}

const CATEGORY_ORDER: string[] = [
  "browser", "media", "communication", "development",
  "user-app", "background", "service", "system", "unknown",
];
```

### 4. Componente `ProcessGroup`

```tsx
// src/features/processes/components/process-group.tsx
import { useState } from "react";
import { ChevronRight, ChevronDown } from "lucide-react";
import { ProcessRow } from "./process-row";

const CATEGORY_LABELS: Record<string, string> = {
  browser: "Browsers",
  media: "Media",
  communication: "Comunicación",
  development: "Desarrollo",
  "user-app": "Aplicaciones",
  background: "Background / UWP",
  service: "Servicios (svchost)",
  system: "Sistema",
  unknown: "Otros",
};

export function ProcessGroup({ category, processes }: {
  category: string; processes: ProcessInfo[]
}) {
  const [expanded, setExpanded] = useState(category !== "system");
  const total = processes.length;
  const totalRam = processes.reduce((s, p) => s + p.memoryBytes, 0);

  return (
    <div className="border border-border rounded-lg">
      <button
        onClick={() => setExpanded(v => !v)}
        className="w-full flex items-center gap-2 p-3 hover:bg-accent/30"
      >
        {expanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        <span className="font-medium">{CATEGORY_LABELS[category]}</span>
        <span className="ml-auto text-sm text-muted-foreground">
          {total} · {formatBytes(totalRam)}
        </span>
      </button>
      {expanded && (
        <div className="divide-y divide-border/50">
          {processes
            .sort((a, b) => b.memoryBytes - a.memoryBytes)
            .map(p => <ProcessRow key={p.pid} process={p} />)}
        </div>
      )}
    </div>
  );
}
```

### 5. Componente `ProcessRow`

```tsx
// src/features/processes/components/process-row.tsx
import { useMutation } from "@tanstack/react-query";
import { Pause, Play, X, Skull } from "lucide-react";
import { Button } from "../../../components/ui/button";
import { killProcess, suspendProcess, resumeProcess, closeGracefully } from "../../../api";
import { toast } from "../../../lib/toast";

export function ProcessRow({ process }: { process: ProcessInfo }) {
  const close = useMutation({
    mutationFn: () => closeGracefully(process.pid, 5000),
    onSuccess: (graceful) => toast.success(
      graceful ? `${process.name} cerrada` : `${process.name} forzada (no respondía)`
    ),
    onError: (err) => toast.error(`No se pudo cerrar ${process.name}`, err),
  });

  const kill = useMutation({
    mutationFn: () => killProcess(process.pid),
    onSuccess: () => toast.success(`${process.name} terminada`),
    onError: (err) => toast.error(`No se pudo terminar ${process.name}`, err),
  });

  return (
    <div className="px-4 py-2 flex items-center gap-3 hover:bg-accent/20 text-sm">
      <div className="flex-1 min-w-0">
        <div className="font-medium truncate">
          {process.displayName ?? process.name}
        </div>
        <div className="text-xs text-muted-foreground truncate">
          PID {process.pid} · {process.exePath ?? "(sin path)"}
        </div>
      </div>
      <div className="text-xs font-mono text-muted-foreground">
        {process.cpuPercent.toFixed(1)}%
      </div>
      <div className="text-xs font-mono text-muted-foreground w-20 text-right">
        {formatBytes(process.memoryBytes)}
      </div>
      <div className="flex gap-1">
        <Button
          size="sm"
          variant="outline"
          disabled={process.isProtected || close.isPending}
          onClick={() => close.mutate()}
          title={process.isProtected ? "Proceso protegido" : "Cerrar (WM_CLOSE)"}
        >
          <X className="h-3 w-3" />
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={process.isProtected || kill.isPending}
          onClick={() => {
            if (confirm(`¿Terminar ${process.name} forzadamente? Puede perderse datos.`)) {
              kill.mutate();
            }
          }}
          title="Kill (forzar)"
        >
          <Skull className="h-3 w-3" />
        </Button>
      </div>
    </div>
  );
}
```

### 6. ReleaseButton

```tsx
// src/features/processes/components/release-button.tsx
import { useMutation } from "@tanstack/react-query";
import { releaseCaches } from "../../../api";
import { useNavigate } from "react-router-dom";
import { Sparkles } from "lucide-react";
import { Button } from "../../../components/ui/button";
import { toast } from "../../../lib/toast";

export function ReleaseButton() {
  const navigate = useNavigate();
  const release = useMutation({
    mutationFn: releaseCaches,
    onSuccess: (report) => {
      toast.success(`${report.closedCount} apps cerradas`, {
        description: "Redirigiendo a Caché...",
      });
      setTimeout(() => navigate("/cache"), 1500);
    },
    onError: (err) => toast.error("Error cerrando apps", err),
  });

  return (
    <Button
      variant="outline"
      onClick={() => {
        if (confirm(
          "Esto cerrará grácilmente Chrome, Edge, Spotify, Discord, Claude y similares. ¿Continuar?"
        )) release.mutate();
      }}
      disabled={release.isPending}
    >
      <Sparkles className="h-4 w-4 mr-2" />
      {release.isPending ? "Cerrando..." : "Liberar para limpiar"}
    </Button>
  );
}
```

### 7. Backend del `release_caches`

```rust
// src-tauri/src/platform/processes.rs

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseReport {
    pub closed_count: u32,
    pub failed_count: u32,
    pub closed_processes: Vec<String>,
}

const COMMON_CACHE_APPS: &[&str] = &[
    "spotify.exe", "discord.exe", "claude.exe", "obsidian.exe",
    "chrome.exe", "msedge.exe", "brave.exe", "firefox.exe",
    "slack.exe", "teams.exe", "ms-teams.exe", "telegram.exe", "whatsapp.exe",
];

pub async fn release_common_apps_for_cleanup() -> AppResult<ReleaseReport> {
    let processes = list_processes_extended()?;
    let mut closed = 0u32;
    let mut failed = 0u32;
    let mut closed_names = Vec::new();

    for p in processes {
        let name_lower = p.name.to_lowercase();
        if !COMMON_CACHE_APPS.contains(&name_lower.as_str()) { continue; }
        if p.is_protected { continue; }

        match close_gracefully(p.pid, 5000).await {
            Ok(_) => {
                closed += 1;
                closed_names.push(p.name);
            }
            Err(_) => failed += 1,
        }
    }

    Ok(ReleaseReport {
        closed_count: closed,
        failed_count: failed,
        closed_processes: closed_names,
    })
}
```

### 8. Ruta

```ts
// src/lib/routes.ts
export const ROUTES = {
  // ... existentes
  processes: "/processes",
};
```

Registrar en el router con `<ProcessesPage />`.

## Criterio de done

- [ ] Página `/processes` accesible desde la nav.
- [ ] Lista actualiza cada 2s.
- [ ] Filtros funcionan (search + showSystem + onlyWithUI).
- [ ] Agrupación por categoría con totales por grupo.
- [ ] Botón Cerrar (X) llama `closeGracefully` con toast.
- [ ] Botón Kill (skull) requiere confirm + toast.
- [ ] Procesos protegidos tienen botones grises + tooltip.
- [ ] "Liberar para limpiar" cierra apps comunes con confirm + redirige a Caché.
