# Plan — Rediseño completo del Explorador de Archivos

> Rama: `refactor/ui-cosmic-redesign`
> Generado: 2026-05-21
> Autor: revisión post-refactor "Quirófano Cyan"
> Estado actual: la sección `/explorer` está **funcionalmente rota** y necesita rediseño tanto arquitectónico como de UX.

---

## 1. Resumen ejecutivo

El explorador actual:
1. **No funciona como árbol.** Al expandir una carpeta, sus hijos NO aparecen anidados — caen al mismo nivel que las carpetas hermanas del root, vacíos.
2. **No tiene selector de disco.** Solo hay un input de texto crudo donde el usuario tiene que escribir `C:\` a mano. Sin lista de discos detectados, sin tamaños, sin aviso de qué se va a escanear.
3. **No avisa antes de escanear.** Pulsar "Escanear" lanza inmediatamente. Sin confirmación, sin estimación, sin dry-run.
4. **No usa virtualización**, aunque `@tanstack/react-virtual` ya está en `package.json`.
5. **Estado roto por construcción.** El backend emite eventos `explorer:node` sin `scan_id` ni `parentPath`, por lo que el frontend no puede asociar cada hijo con su padre. El bug de la expansión es consecuencia directa de esto.

Este documento describe el diagnóstico completo, la investigación de mercado y técnica realizada, y un plan operativo por fases para reconstruir el módulo.

---

## 2. Diagnóstico del bug crítico ("la expansión no funciona")

### 2.1 Reproducción

1. Abrir `/explorer`, escribir `C:\`, Escanear.
2. Aparece el listado plano del primer nivel: `Windows`, `Users`, `Program Files`, etc. — correcto.
3. Click en chevron de `Windows`.
4. **Esperado:** los hijos de `Windows` aparecen indentados debajo, marcando que pertenecen a `Windows`.
5. **Observado:** los hijos de `Windows` aparecen al mismo nivel que `Users`, `Program Files`, etc. — sin indentación, sin agrupación, sin distinción visual de quién es su padre.

### 2.2 Causa raíz

Hay **tres bugs encadenados**:

#### Bug A — `TreeView` no es recursivo

[src/features/explorer/tree-view.tsx](../src/features/explorer/tree-view.tsx) renderiza `nodes.map(node => <TreeRow ... />)` sin pasar children ni recursión:

```tsx
{nodes.map((node) => (
  <TreeRow key={node.path} node={node} depth={0} onExpand={onExpand} />
))}
```

`TreeRow` acepta una prop `children?` pero **nadie se la pasa nunca**. Al expandir, no hay forma de pintar los hijos anidados.

#### Bug B — `addNode` aplana todo en un único array

[src/features/explorer/use-explorer-tree.ts:52-61](../src/features/explorer/use-explorer-tree.ts#L52-L61):

```ts
const addNode = useCallback((node: TreeNode) => {
  setState((prev) => ({
    ...prev,
    nodes: [...prev.nodes, node].sort(...),  // ← echo al pozo común
  }));
}, []);
```

`state.nodes` es un `TreeNode[]` plano. **Cualquier** evento `explorer:node` que llegue se añade a esa lista sin importar a qué scan pertenece ni quién es su padre.

Cuando el usuario expande `Windows`, el código llama `scanTree({ root: "C:\\Windows" })`, que dispara eventos `explorer:node` con los hijos de `Windows`. Esos eventos los recoge `addNode` y los apila al lado de `Users`, `Program Files`, etc. Visualmente parecen "carpetas hermanas".

#### Bug C — `expandNode` reset a `children: []`

[src/features/explorer/use-explorer-tree.ts:97-102](../src/features/explorer/use-explorer-tree.ts#L97-L102):

```ts
if (childrenResult.status === "fulfilled") {
  next.set(path, {
    children: [] as TreeNode[],   // ← siempre vacío
    ...
  });
}
```

Aunque arregláramos B (asociar nodos a su parent), `expandNode` ignora el resultado de `scanTree` y siempre setea `children` a `[]`. Los hijos nunca llegan al `Map<path, NodeState>`.

#### Bug D — Backend no incluye `parentPath` ni `scanId` en eventos

[src-tauri/src/ipc/explorer.rs:22-25](../src-tauri/src/ipc/explorer.rs#L22-L25):

```rust
for node in &nodes {
    let _ = app.emit("explorer:node", node);  // ← solo el TreeNode, sin contexto
}
```

Para que el frontend pueda enrutar correctamente cada hijo a su padre, el evento necesita acarrear **al menos** `scanId` y `parentPath`. Sin eso, no hay forma fiable de saber a quién pertenece cada nodo emitido.

### 2.3 Conclusión

El diseño actual mezcla:
- Estado plano (`nodes: TreeNode[]`) + estado por path (`byPath: Map<string, NodeState>`) — duplica fuente de verdad.
- Estado local de expansión en `TreeRow` (`useState(expanded)`) + estado global en el hook — race conditions garantizadas.
- Eventos del backend sin metadatos suficientes.
- Renderizado plano sin recursión.

**No basta con parchear**. Hay que rediseñar la representación del árbol y el contrato de eventos.

---

## 3. Otros bugs y problemas detectados (revisión completa)

| # | Severidad | Archivo / línea | Problema |
|---|---|---|---|
| 1 | Crítico | tree-view + use-explorer-tree | Bug A/B/C/D descritos arriba |
| 2 | Alto | explorer-page.tsx | Sin selector de disco — solo `<Input>` de texto |
| 3 | Alto | explorer-page.tsx | Sin confirmación antes de escanear (puede dispararse sobre `C:\` accidentalmente) |
| 4 | Alto | tree-view.tsx | Sin virtualización — un escaneo de `C:\Users\…\AppData` puede generar miles de filas y congelar la UI |
| 5 | Medio | use-explorer-tree.ts:48 | `catch {}` silencioso — los errores del backend no se propagan a la UI |
| 6 | Medio | tree-row.tsx:43-52 | Estado `expanded` local conflictivo con `byPath.expanded` global |
| 7 | Medio | tree-row.tsx:76-80 | Línea de indentación (`<div className="absolute ...">`) sin `position: relative` en el padre — se posiciona respecto a la página, no respecto a la fila |
| 8 | Medio | use-explorer-tree.ts:55-58 | `[...prev.nodes, node].sort(...)` en cada evento — O(n log n) por nodo. 5000 archivos = 25M ops |
| 9 | Medio | explorer-page.tsx:33 | `onChange={(e) => onScan(e.target.value)}` — escribir un carácter lanza un escaneo |
| 10 | Medio | explorer-page.tsx:46-49 | `handleCancel` solo marca `scanning=false` en el frontend — el backend sigue corriendo |
| 11 | Medio | ipc/explorer.rs:31 | `cancel_scan` es no-op. No hay tokens de cancelación reales |
| 12 | Medio | tree-row.tsx:35-40 | Colores por tamaño hard-codeados (`text-green-400`, `text-red-400`) — no usan tokens nuevos `signal.*` |
| 13 | Bajo | tree-view.tsx | Look-and-feel pre-refactor — sigue usando `bg-card`, `text-muted-foreground`, no la paleta Quirófano Cyan |
| 14 | Bajo | tree-row.tsx | Tooltip de path completo solo via atributo `title=` HTML — UX pobre |
| 15 | Bajo | explorer-page.tsx | Búsqueda local solo filtra el primer nivel del array plano. Carpetas profundas son invisibles a la búsqueda |
| 16 | Bajo | use-explorer-tree.ts | `addChildNodes` se exporta pero **nadie lo usa** — código muerto |
| 17 | Bajo | ipc/explorer.rs | Documentación dice "está preparada para escuchar `explorer:node`" pero el comando es **síncrono** y emite todos los nodos de golpe al final — falso streaming |
| 18 | Bajo | Backend | `compute_directory_size` no soporta cancelación — un escaneo de tamaño en `C:\` puede tardar minutos sin posibilidad de abortar |

---

## 4. Investigación de mercado (referentes UX)

Buscando "cómo hacen los demás" para herramientas de exploración de disco y árboles de archivos profesionales.

### 4.1 [WizTree](https://diskanalyzer.com/)

Probablemente la referencia clave para **velocidad** y **simplicidad** en Windows:

- **Flujo:** dropdown de discos detectados → click "Scan" → en segundos lista jerárquica + treemap.
- **No te deja escribir paths a mano** — los discos los descubre del SO.
- Top pane: árbol jerárquico ordenado por tamaño.
- Bottom pane: **treemap** (rectángulos proporcionales al tamaño) — pista visual inmediata de qué pesa más.
- Lee la MFT (Master File Table) directamente en NTFS — por eso es órdenes de magnitud más rápido que iterar archivos con APIs Win32 estándar.

**Qué nos enseña:**
- El usuario empieza eligiendo **disco**, no path.
- El treemap es opcional pero valioso para "qué me come el espacio".
- Velocidad = expectativa, no extra.

### 4.2 [TreeSize](https://www.jam-software.com/treesize)

Más enterprise / profesional:

- Selector inicial: drive **o** path **o** network share.
- Vista de árbol con barra de uso (% del padre) en cada fila — visualmente lees la jerarquía sin abrir nada.
- Modos: top files, file types, age, owner.
- Reporting (CSV/HTML).

**Qué nos enseña:**
- Las **barras de uso inline** por fila (% del padre) son la mejor forma de leer "quién pesa mucho dentro de su carpeta" sin abrir.
- Modos múltiples sobre el mismo dataset = potente sin complicar.

### 4.3 [SpaceMonger](https://spacemongerapp.com/) / SpaceSniffer

- Solo treemap full-screen, sin árbol.
- Click para drill-down, click-derecho para back.
- **Pista para nuestro caso:** la combinación árbol + treemap (WizTree) es superior a solo treemap (SpaceMonger) cuando además quieres limpiar o navegar.

### 4.4 VS Code Explorer / Finder / File Explorer de Windows 11

- Árbol vertical infinito con chevron + indentación 16-20 px.
- Iconos consistentes por tipo de archivo.
- Scroll virtualizado (en el caso de VS Code).
- Sin tamaños inline — eso es lo que nos diferencia de un file manager genérico.

---

## 5. Investigación técnica (cómo construir el árbol)

### 5.1 Librerías candidatas

| Librería | Bundle | Virtualización | DnD | Editing inline | Headless |
|---|---|---|---|---|---|
| [react-arborist](https://github.com/brimdata/react-arborist) | ~25 KB | Sí | Sí | Sí | No (UI incluida) |
| [headless-tree](https://headless-tree.lukasbach.com/) | ~12 KB | Sí (via @tanstack/virtual) | Sí | Sí | Sí (sólo lógica) |
| react-complex-tree | ~40 KB | Parcial | Sí | Sí | No |
| Custom (con `@tanstack/react-virtual`) | 0 KB extra | Manual | No | No | Total |

### 5.2 Decisión recomendada: **headless-tree + nuestro propio JSX**

Justificación:
- ClearTool ya tiene `@tanstack/react-virtual` en `package.json` (ahora mismo sin usar).
- Queremos que el árbol **se vea como Quirófano Cyan**, no como el theme por defecto de una librería.
- headless-tree separa lógica (focus, expansión, multi-selección, lazy loading) del rendering — encaja con el sistema de tokens nuevo.
- Bundle pequeño + cero dependencias visuales.
- Soporta `lazy loading` con un `dataLoader` async — es el modelo natural para nuestro caso (cada expand dispara un `scanTree`).

Alternativa: custom 100% con `@tanstack/react-virtual` si headless-tree resulta excesivo. Coste extra: ~200 líneas reescribiendo focus/keyboard/expansion, pero ya tenemos el bug B/C/D resueltos.

**Recomendación:** custom 100% en la fase 1 (control total, sin dep nueva), evaluar headless-tree en fase 2 si lo necesitamos.

### 5.3 Modelo de datos correcto

Reemplazar el actual `nodes: TreeNode[] + byPath: Map<string, NodeState>` por:

```ts
type NodeId = string;  // ruta absoluta canonicalizada

interface TreeNodeState {
  node: TreeNode;
  parentId: NodeId | null;       // null = raíz
  childrenIds: NodeId[] | null;   // null = no cargado, [] = cargado sin hijos
  expanded: boolean;
  loading: boolean;               // hay scan pendiente sobre este nodo
  sizeComputed: number | null;    // null = no calculado todavía
  computingSize: boolean;
  error: string | null;
  depth: number;                  // pre-calculado para virtual scroll
}

interface TreeStore {
  rootId: NodeId | null;
  byId: Map<NodeId, TreeNodeState>;
  visibleIds: NodeId[];           // derivado: traversal pre-order de visibles
  activeScanId: string | null;    // último scan emitido; ignorar eventos viejos
}
```

`visibleIds` se recalcula cuando cambia expanded/childrenIds/parentId. Es lo que `<VirtualList>` itera para rendering.

### 5.4 Contrato de eventos nuevo (backend)

Modificar `ipc/explorer.rs` para que `explorer:node` lleve metadatos:

```rust
#[derive(serde::Serialize, ts_rs::TS, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeEvent {
    pub scan_id: String,
    pub parent_path: String,
    pub node: TreeNode,
}

for node in &nodes {
    let _ = app.emit("explorer:node", NodeEvent {
        scan_id: scan_id.clone(),
        parent_path: input.root.clone(),
        node: node.clone(),
    });
}
```

Permite al frontend:
- Descartar eventos de scans cancelados (`scanId != activeScanId`).
- Insertar el nodo bajo su `parentPath` correcto en `byId`.

Además: convertir `scan_tree` a `async + tokio::spawn` para que emita streaming real (en vez de bloquear hasta tener todos los nodos).

---

## 6. Plan de UX nuevo

### 6.1 Pantalla inicial (sin escaneo activo)

```
┌─────────────────────────────────────────────────────────┐
│  Explorador                                              │
│  Selecciona qué quieres analizar.                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│   Discos detectados                                      │
│   ┌────────────────────────────────────────────────────┐ │
│   │ C:\  Sistema       512 GB · 187 libres   [Analizar]│ │  ← card
│   │ ▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░  63 % usado                     │ │
│   ├────────────────────────────────────────────────────┤ │
│   │ D:\  Data         1.0 TB · 720 libres    [Analizar]│ │
│   │ ▓▓▓▓▓▓░░░░░░░░░░░░░  28 % usado                     │ │
│   ├────────────────────────────────────────────────────┤ │
│   │ X:\  USB Backup    32 GB · 2 libres      [Analizar]│ │
│   │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░  94 % usado · ámbar              │ │
│   └────────────────────────────────────────────────────┘ │
│                                                          │
│   ─── o usa una ruta concreta ────────────────────────  │
│   [_____________________________]  [Analizar ruta]      │
│   Ej: C:\Users\jowy\Downloads                           │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

- Discos: listados desde `SystemSummary.drives` (ya existe).
- Cada card: barra de uso + chip ámbar/rojo si <10 % libre.
- Click "Analizar" → confirmación → escaneo.
- Ruta arbitraria sigue disponible pero **secundaria**.

### 6.2 Modal de confirmación antes de escanear

Obligatorio para todo escaneo:

```
┌──────────────────────────────────────────────────┐
│  Analizar C:\                                     │
├──────────────────────────────────────────────────┤
│  El escaneo es de **solo lectura**. No modifica  │
│  nada en disco.                                  │
│                                                  │
│  Profundidad: 1 nivel (los hijos se cargan       │
│  bajo demanda al expandir).                      │
│  Tiempo estimado: 5–30 s según el disco.         │
│  Cancelable en cualquier momento.                │
│                                                  │
│  ☐ Incluir archivos ocultos                      │
│  ☐ Seguir junctions / symlinks (riesgo de loop)  │
│                                                  │
│       [Cancelar]    [Analizar →]                  │
└──────────────────────────────────────────────────┘
```

Coherente con el principio "Transparencia total" del CLAUDE.md.

### 6.3 Pantalla durante el escaneo / con resultados

```
┌─────────────────────────────────────────────────────────────────┐
│  Explorador · C:\        [↻ rescanear]  [✕ cancelar]            │
│  ─────────────────────────────────────────────────────────────  │
│  ►  Windows                     38.4 GB  ▓▓▓▓▓▓▓▓▓▓▓░░  29%   │
│  ▼  Users                       64.2 GB  ▓▓▓▓▓▓▓▓▓▓▓▓░  49%   │
│        ▼ jowy                   58.1 GB  ▓▓▓▓▓▓▓▓▓▓▓▓▓  44%   │
│             ► AppData           41.3 GB  ▓▓▓▓▓▓▓▓░░░░░  31%   │
│             ► Documents          3.2 GB  ▓░░░░░░░░░░░░   2%   │
│             ► Downloads         12.0 GB  ▓▓▓░░░░░░░░░░   9%   │
│        ► Public                  6.1 GB  ▓▓░░░░░░░░░░░   4%   │
│  ►  Program Files               18.7 GB  ▓▓▓▓░░░░░░░░░  14%   │
│  ►  ProgramData                  8.1 GB  ▓░░░░░░░░░░░░   6%   │
│  ─────────────────────────────────────────────────────────────  │
│  187 carpetas · 12,453 archivos · 132.8 GB analizados            │
└─────────────────────────────────────────────────────────────────┘
```

- **Indentación 18 px por nivel** + chevron + icono.
- **Barra horizontal % del padre** — pista visual estilo TreeSize.
- Tamaño formateado en `JetBrains Mono` (tabular).
- Skeleton/spinner inline mientras el hijo carga.
- Virtualización vertical para listas grandes.
- Sticky header con totales abajo.

### 6.4 Estados a cubrir

| Estado | Comportamiento |
|---|---|
| Sin escaneo | Pantalla 6.1 |
| Confirmación | Modal 6.2 |
| Escaneando inicial | Skeleton de filas + spinner en barra superior |
| Resultados | Pantalla 6.3 |
| Nodo expandiendo | Chevron rota, spinner inline en columna tamaño |
| Nodo con error | Chip rojo "error" + tooltip con detalle |
| Nodo protegido (sin permisos) | Icono candado + tooltip "Requiere admin" |
| Búsqueda activa | Highlight de matches + filtra ramas |
| Cancelado mid-scan | Toast + se conservan los resultados parciales |

---

## 7. Plan de implementación por fases

**Cada fase es ejecutable en una sesión y deja la app funcionando.**

### Fase 1 — Backend: contrato correcto + cancelación real

#### Paso 1.1 — Añadir `NodeEvent` y emitir con `parentPath` + `scanId`

**Archivo:** [src-tauri/src/models/tree.rs](../src-tauri/src/models/tree.rs) (o donde vivan los models del explorer)

```rust
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeEvent {
    pub scan_id: String,
    pub parent_path: String,
    pub node: TreeNode,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanDoneEvent {
    pub scan_id: String,
    pub parent_path: String,
    pub total_emitted: usize,
}
```

#### Paso 1.2 — Reescribir `scan_tree` con streaming async + cancellation token

**Archivo:** [src-tauri/src/ipc/explorer.rs](../src-tauri/src/ipc/explorer.rs)

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static ACTIVE_SCANS: Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
pub async fn scan_tree(app: tauri::AppHandle, input: ScanTreeInput)
    -> AppResult<ScanTreeHandle>
{
    let scan_id = Uuid::new_v4().to_string();
    let cancel = Arc::new(AtomicBool::new(false));
    ACTIVE_SCANS.lock().await.insert(scan_id.clone(), cancel.clone());

    let scan_id_clone = scan_id.clone();
    let parent = input.root.clone();
    tauri::async_runtime::spawn(async move {
        let mut emitted = 0;
        match domain::explorer::list_top_level(&parent, input.follow_reparse_points) {
            Ok(nodes) => {
                for node in nodes {
                    if cancel.load(Ordering::SeqCst) { break; }
                    let _ = app.emit("explorer:node", NodeEvent {
                        scan_id: scan_id_clone.clone(),
                        parent_path: parent.clone(),
                        node,
                    });
                    emitted += 1;
                }
            }
            Err(e) => {
                let _ = app.emit("explorer:error", ScanErrorEvent {
                    scan_id: scan_id_clone.clone(),
                    parent_path: parent.clone(),
                    message: format!("{e}"),
                });
            }
        }
        let _ = app.emit("explorer:done", ScanDoneEvent {
            scan_id: scan_id_clone.clone(),
            parent_path: parent,
            total_emitted: emitted,
        });
        ACTIVE_SCANS.lock().await.remove(&scan_id_clone);
    });

    Ok(ScanTreeHandle { scan_id })
}

#[tauri::command]
pub async fn cancel_scan(handle: ScanTreeHandle) -> AppResult<()> {
    if let Some(flag) = ACTIVE_SCANS.lock().await.get(&handle.scan_id) {
        flag.store(true, Ordering::SeqCst);
    }
    Ok(())
}
```

#### Paso 1.3 — Hacer `compute_directory_size` cancelable

Aceptar `scan_id` opcional y registrar su flag en `ACTIVE_SCANS`. El recorrido en `filesystem::directory_stats` debe consultar el flag periódicamente (ej. cada 256 entradas).

#### Paso 1.4 — Tests

Añadir tests integration en `src-tauri/tests/explorer.rs` que:
- Lanzan un scan y verifican que los eventos llegan con `scan_id` y `parent_path` correctos.
- Cancelan mid-scan y verifican que el flag se honra.

### Fase 2 — Frontend: nuevo modelo de datos

#### Paso 2.1 — Crear `tree-store.ts`

**Archivo:** `src/features/explorer/tree-store.ts` (nuevo)

Store basado en Zustand (ya está en deps) con el `TreeStore` descrito en 5.3. Acciones:
- `startScan(rootPath, opts)` — limpia y arranca scan, devuelve scanId.
- `cancelActiveScan()`.
- `onNodeEvent(event)` — inserta nodo bajo su parent, ignora si scanId no es el activo.
- `onDoneEvent(event)` — marca el parent como `loading=false`.
- `expandNode(id)` — toggle expanded, si no tiene children pide scan.
- `collapseNode(id)`.
- `computeSize(id)` — dispara `compute_directory_size` y actualiza `sizeComputed`.
- `recomputeVisible()` — refresca `visibleIds` con traversal pre-order de visibles.

#### Paso 2.2 — Borrar `use-explorer-tree.ts`

Sustituido por el store. Mantener compat: `useExplorerTree = () => useTreeStore(...)` durante migración.

### Fase 3 — Frontend: componentes con la paleta nueva

#### Paso 3.1 — `drive-picker.tsx`

**Archivo:** `src/features/explorer/components/drive-picker.tsx` (nuevo)

Lista de discos como cards horizontales (no grid). Cada card:
- Letra grande + label.
- Bytes totales/libres en JetBrains Mono.
- Barra horizontal de uso con color cyan (<70%) / ámbar (70-90%) / rojo (>90%).
- Botón `[Analizar]` a la derecha.

Datos desde `useSystemSummary().drives`.

#### Paso 3.2 — `scan-confirm-modal.tsx`

**Archivo:** `src/features/explorer/components/scan-confirm-modal.tsx` (nuevo)

Modal Radix Dialog (ya en deps). Pantalla 6.2. Opciones:
- Switch "Incluir ocultos" (default off).
- Switch "Seguir junctions" (default off, con warning).
- Tiempo estimado: cálculo simple basado en `totalBytes / 50e9 * 30s`.

Devuelve `{ confirmed: boolean, options: ScanTreeInput }`.

#### Paso 3.3 — `tree-view-v2.tsx` con virtualización

**Archivo:** `src/features/explorer/components/tree-view-v2.tsx` (nuevo)

```tsx
import { useVirtualizer } from "@tanstack/react-virtual";

function TreeViewV2() {
  const visibleIds = useTreeStore((s) => s.visibleIds);
  const byId = useTreeStore((s) => s.byId);
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: visibleIds.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 12,
  });

  return (
    <div ref={parentRef} className="flex-1 overflow-auto inset">
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualizer.getVirtualItems().map((vRow) => {
          const id = visibleIds[vRow.index];
          const state = byId.get(id)!;
          return (
            <TreeRowV2
              key={id}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${vRow.start}px)`,
              }}
              state={state}
            />
          );
        })}
      </div>
    </div>
  );
}
```

#### Paso 3.4 — `tree-row-v2.tsx`

Una fila plana con padding-left = `depth * 18`. Sin estado local de expansión: lee `state.expanded` del store. Estructura:

```
[chevron][icono] nombre · [barra %parent] · [tamaño mono]
```

Acciones:
- Click en chevron → `toggleExpand(id)`.
- Click en fila → seleccionar (highlight cyan tenue, multi-select con Cmd/Ctrl).
- Right-click → context menu (futuro: abrir en Explorer, copiar path, calcular tamaño).

#### Paso 3.5 — `explorer-page-v2.tsx` orquestador

Lógica:
1. Si `rootId === null` → renderizar `<DrivePicker>` + input ruta.
2. Si confirmando → `<ScanConfirmModal>`.
3. Si escaneando o con resultados → `<TreeViewV2>` + toolbar.

Cuando el flujo nuevo funciona, sustituir `<ExplorerPage>` original.

### Fase 4 — Pulido visual y QA

#### Paso 4.1 — Aplicar tokens Quirófano Cyan

Cambiar todos los `text-muted-foreground`, `bg-card`, `border-border` por tokens `ink-*`, `surface-*`, `edge-*` y semánticos `signal-*`.

#### Paso 4.2 — Estados vacíos / error / loading

- Sin discos: chip rojo "no se detectaron discos".
- Sin permisos en una rama: icono candado en la fila.
- Error de scan: panel inline con CTA "reintentar".

#### Paso 4.3 — Búsqueda funcional

Reemplazar el `state.nodes.filter(...)` actual por una búsqueda que:
- Filtra `byId` por nombre/path (case-insensitive).
- Expande automáticamente las ramas que contienen matches.
- Highlight de la subcadena coincidente con `<mark className="bg-signal-cyan/20 text-signal-cyan">`.

#### Paso 4.4 — Atajos de teclado

- `Enter` en una fila Dir → toggle expand.
- `→` → expandir.
- `←` → colapsar (si ya colapsado, ir al padre).
- `↑/↓` → navegar.
- `Ctrl+F` → focus búsqueda.

---

## 8. Verificación / aceptación

Al final de la fase 3:

- [ ] Abrir `/explorer` muestra cards de discos detectados, no input vacío.
- [ ] Click "Analizar" en un disco → modal de confirmación.
- [ ] Confirmar → scan arranca, eventos llegan, árbol se popula incrementalmente.
- [ ] Click chevron en una carpeta → sus hijos aparecen **anidados, indentados, debajo de ella**. NO al mismo nivel que las hermanas. NO vacíos.
- [ ] Expandir 10 niveles seguidos no congela la UI.
- [ ] Listas con 5000+ entradas hacen scroll a 60 fps.
- [ ] Botón "Cancelar" durante un scan → eventos paran, app sigue responsiva.
- [ ] Recargar `/explorer` mantiene el estado del store hasta navegación / reset manual.
- [ ] Búsqueda destaca matches y expande ramas relevantes.

Al final de la fase 4:

- [ ] La paleta coincide con el resto del shell (Quirófano Cyan).
- [ ] Todos los tamaños en JetBrains Mono con `tabular-nums`.
- [ ] Carpetas protegidas muestran candado en lugar de error genérico.
- [ ] `prefers-reduced-motion` desactiva la rotación animada del chevron.

---

## 9. Fuera de scope (futuras iteraciones)

- **Treemap visual** estilo WizTree/SpaceMonger — interesante pero requiere su propia spec (D3 / observable plot / canvas custom).
- **Lectura directa de MFT** para velocidad WizTree-like — requiere acceso raw a NTFS (privilegios SYSTEM) y librería específica.
- **Modos de visualización** (por extensión, por edad, por owner) — útiles, pero el árbol básico es prioridad.
- **Acciones sobre archivos** (borrar, mover, abrir en Explorer.exe) — el módulo es explorador, no file manager. Si se añade, requiere su propio audit log.
- **Comparar dos snapshots** (qué creció entre el lunes y hoy) — feature de TreeSize, agrega valor pero suma scope.
- **Watch en tiempo real** (notify-rs) — interesante para "ver crecer una carpeta", pero alto coste y baja demanda.

---

## 10. Sources / referencias

### Mercado
- [WizTree — Disk Space Analyzer](https://diskanalyzer.com/)
- [TreeSize Professional](https://www.jam-software.com/treesize)
- [SpaceSniffer / SpaceMonger](https://spacemongerapp.com/)

### Técnico
- [React Arborist (jameskerr/brimdata)](https://github.com/brimdata/react-arborist)
- [Headless Tree (lukasbach)](https://headless-tree.lukasbach.com/) — [post sobre arquitectura](https://medium.com/@lukasbach/headless-tree-and-the-future-of-react-complex-tree-fc920700e82a)
- [exploration — primitives by jaredLunde](https://github.com/jaredLunde/exploration)
- [Building a Recursive Component in React (folder explorer)](https://medium.com/@jaswanth_270602/building-a-recursive-component-in-react-folder-explorer-react-series-part-12-0e952893af7c)
- [File Explorer in React — GreatFrontEnd interview](https://www.greatfrontend.com/questions/user-interface/file-explorer)
- [@tanstack/react-virtual docs](https://tanstack.com/virtual/latest)
