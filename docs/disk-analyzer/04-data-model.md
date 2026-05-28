# 04 — Modelo de datos

DTOs end-to-end. Todos los structs Rust llevan
`#[serde(rename_all = "camelCase")]` y aparecen reflejados al milímetro en
`src/api/types.ts`.

## `DriveListing`

Ver `01-disk-enumeration.md`.

## `NodeKind`

```rust
pub enum NodeKind { Dir, File, Symlink, Junction }
```

Se traslada desde `models/tree.rs` (borrado) a `models/disk.rs`.

## `TreemapNode` (ampliado)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreemapNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: NodeKind,
    pub extension: Option<String>,
    pub file_count: u64,           // archivos descendientes (recursivo)
    pub dir_count: u64,            // dirs descendientes (recursivo)
    pub last_modified: Option<String>, // ISO 8601 del más reciente
    pub percent_of_parent: f32,    // 0.0 .. 100.0 (lo calcula backend)
    pub percent_of_root: f32,      // 0.0 .. 100.0
    pub children: Vec<TreemapNode>,
    pub truncated: bool,           // true si tiene hijos no emitidos (max_depth)
    pub error: Option<String>,     // "access denied", etc.
}
```

## `BuildTreemapInput` (revisado)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTreemapInput {
    pub root: String,                       // "C:\\" típicamente
    pub max_depth_emit: u32,                // default 6
    pub min_size_mb: u64,                   // default 10
    pub follow_reparse_points: bool,        // default false
    pub include_hidden: bool,               // default true
    pub include_system: bool,               // default true
    pub scan_id: String,                    // uuid v4 desde el frontend
}
```

## `DiskAnalysisReport`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskAnalysisReport {
    pub scan_id: String,
    pub root: String,
    pub scanned_at: String,         // ISO 8601
    pub duration_ms: u64,
    pub total_bytes: u64,
    pub total_files: u64,
    pub total_dirs: u64,
    pub free_bytes: u64,            // del disco al momento de escanear
    pub drive_total_bytes: u64,     // capacidad del disco
    pub top_extensions: Vec<ExtensionStat>,
    pub largest_folders: Vec<FolderStat>,
    pub largest_files: Vec<FileStat>,
    pub age_distribution: AgeDistribution,
    pub errors: Vec<String>,        // warnings durante el escaneo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionStat {
    pub extension: String,          // "" para sin extensión
    pub category: ExtCategory,      // media/image/code/docs/archive/executable/other
    pub bytes: u64,
    pub file_count: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderStat {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub file_count: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStat {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub extension: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgeDistribution {
    pub last_7_days: AgeBucket,
    pub last_30_days: AgeBucket,
    pub last_90_days: AgeBucket,
    pub last_1_year: AgeBucket,
    pub last_5_years: AgeBucket,
    pub older: AgeBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgeBucket { pub bytes: u64, pub file_count: u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtCategory {
    Media, Image, Code, Docs, Archive, Executable, Database, Font, ThreeD, Other,
}
```

## Respuesta del comando `build_treemap_data`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskAnalysisResult {
    pub report: DiskAnalysisReport,
    pub root: TreemapNode,
}
```

## TypeScript

Crear los espejos en `src/api/types.ts`. Ejemplo final mínimo:

```ts
export type NodeKind = "Dir" | "File" | "Symlink" | "Junction";
export type ExtCategory =
  | "media" | "image" | "code" | "docs" | "archive"
  | "executable" | "database" | "font" | "threeD" | "other";
export type DriveType =
  | "fixed" | "removable" | "network" | "cdRom" | "ramDisk" | "unknown";
// ...resto idéntico camelCase.
```

## Sembrado para mocks (modo web)

En `client.ts` el `mocks["list_drives"]` devuelve `[{ letter: "C", ... }]`
con un dato realista para que la UI compile y se vea en modo web.
