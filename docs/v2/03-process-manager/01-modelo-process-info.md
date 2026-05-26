# Paso 01 — Modelos `ProcessInfo`, `ProcessCategory`

**Área**: 03-process-manager
**Tiempo estimado**: 1 hora
**Dependencias**: ninguna

## Qué hacemos

Definir los DTOs Rust + TS que representan un proceso y su categorización.

## Archivos que tocamos

- `src-tauri/src/models/process.rs` (nuevo)
- `src-tauri/src/models/mod.rs` (export)
- `src/api/types.ts` (tipos correspondientes)

## Cómo

### Rust

```rust
// src-tauri/src/models/process.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub display_name: Option<String>,
    pub exe_path: Option<String>,
    pub session_id: u32,
    pub start_time: Option<String>,    // RFC3339
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub thread_count: u32,
    pub handle_count: u32,
    pub command_line: Option<String>,
    pub user_sid: Option<String>,
    pub user_name: Option<String>,
    pub is_uwp: bool,
    pub uwp_package_family: Option<String>,
    pub category: ProcessCategory,
    pub is_protected: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessCategory {
    System,
    Service,
    Browser,
    Communication,
    Media,
    Development,
    Background,
    UserApp,
    Unknown,
}

impl Default for ProcessCategory {
    fn default() -> Self { ProcessCategory::Unknown }
}
```

### TS

```ts
// src/api/types.ts (añadir)

export interface ProcessInfo {
  pid: number;
  parentPid: number | null;
  name: string;
  displayName: string | null;
  exePath: string | null;
  sessionId: number;
  startTime: string | null;
  cpuPercent: number;
  memoryBytes: number;
  threadCount: number;
  handleCount: number;
  commandLine: string | null;
  userSid: string | null;
  userName: string | null;
  isUwp: boolean;
  uwpPackageFamily: string | null;
  category: ProcessCategory;
  isProtected: boolean;
}

export type ProcessCategory =
  | "system"
  | "service"
  | "browser"
  | "communication"
  | "media"
  | "development"
  | "background"
  | "user-app"
  | "unknown";
```

## Criterio de done

- [ ] `ProcessInfo` y `ProcessCategory` definidos en Rust.
- [ ] Exportados desde `models/mod.rs`.
- [ ] Tipos TypeScript correspondientes.
- [ ] `cargo check` + `tsc --noEmit` pasan.
