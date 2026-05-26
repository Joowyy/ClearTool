# Plan — Dashboard rediseñado con 3D real + monitorización en vivo

> Plan operativo Parte 2. Convierte la HomePage actual (5 cards estáticas) en un dashboard de control con render 3D real y métricas live.
> Generado: 2026-05-20.

## 1. Visión

```
┌──────────────────────────────────────────────────────────────────────┐
│  ╭─────────────────────────────────────╮  ┌──────────────────────┐   │
│  │                                      │  │ Top procesos          │   │
│  │       ANILLO 3D PULSANTE             │  │ ───────────────────── │   │
│  │       (carga global)                 │  │ chrome.exe   12.4 %   │   │
│  │                                      │  │ Code.exe     8.1  %   │   │
│  │       45 %                           │  │ tauri.exe    6.3  %   │   │
│  │                                      │  │ explorer.exe 3.0  %   │   │
│  ╰─────────────────────────────────────╯  │ dwm.exe      2.7  %   │   │
│                                            └──────────────────────┘   │
│  ┌────────────────┐ ┌────────────────┐ ┌────────────────────────┐    │
│  │ CPU últimos 60s│ │ RAM 22 / 32 GB │ │  Discos (tubes 3D)     │    │
│  │   ╱╲    ╱╲    │ │     ◯ 68%       │ │   ▮▮▮▮ C:  ▮▮ X:        │    │
│  │  ╱  ╲  ╱  ╲   │ │                 │ │   ▮▮▮  Y:  ▮  Z:         │    │
│  │ ╱    ╲╱    ╲  │ │                 │ │                          │    │
│  └────────────────┘ └────────────────┘ └────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  GPU 0  NVIDIA RTX 4070   |  uso 34 %  |  6.2 / 12 GB  |  62 °C │ │
│  │  GPU 1  Intel Iris Xe     |  uso 12 %  |  shared       |  --    │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Acciones rápidas (mantengo)                                    │ │
│  │  [Limpiar caché] [Eliminar bloatware] [Servicios] [Restore]     │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

La barra superior (topbar) absorbe el indicador "Modo limitado" + el username pequeño en la esquina (donde antes había la card "Usuario").

## 2. Paleta y look

Variables CSS nuevas en [src/styles/globals.css](../src/styles/globals.css):

```css
:root {
  /* Fondo: gradiente radial muy oscuro con halo violeta-azul */
  --bg-gradient: radial-gradient(
    ellipse at top,
    hsl(240 60% 12%) 0%,
    hsl(225 55% 6%) 45%,
    hsl(222 84% 3%) 100%
  );
  /* Glass card */
  --glass-bg: hsla(240 30% 12% / 0.55);
  --glass-border: hsla(220 80% 70% / 0.18);
  --glass-shadow:
    0 1px 0 0 hsla(220 80% 80% / 0.12) inset,
    0 30px 60px -20px hsla(240 80% 5% / 0.7),
    0 8px 24px -8px hsla(220 80% 50% / 0.18);
  /* Acentos */
  --accent-cyan: hsl(190 95% 55%);
  --accent-violet: hsl(265 90% 65%);
  --accent-amber: hsl(38 95% 60%);
  --accent-red: hsl(0 85% 60%);
}
```

Clases utility:

```css
.glass {
  background: var(--glass-bg);
  backdrop-filter: blur(18px) saturate(140%);
  border: 1px solid var(--glass-border);
  box-shadow: var(--glass-shadow);
  border-radius: 1rem;
}
.glass-hover {
  transition: transform .25s ease-out, box-shadow .25s ease-out;
}
.glass-hover:hover { transform: translateY(-2px) scale(1.005); }
```

Mapping de color por carga (helper en TS):

| Uso     | Color                |
|---------|----------------------|
| 0–60 %  | `var(--accent-cyan)` |
| 60–85 % | `var(--accent-amber)`|
| 85–100 %| `var(--accent-red)`  |

## 3. Componentes 3D (R3F)

### 3.1. `dashboard-hero-3d.tsx`

Anillo 3D pulsante con `MeshDistortMaterial` (drei). Color HSL animado en función del % global. La escena:

```tsx
<Canvas camera={{ position: [0, 0, 4], fov: 50 }} dpr={[1, 2]}>
  <ambientLight intensity={0.4} />
  <pointLight position={[5, 5, 5]} intensity={1.2} color={accentColor} />
  <pointLight position={[-5, -5, 5]} intensity={0.6} color="#5b9bff" />
  <Float speed={1.2} rotationIntensity={0.4} floatIntensity={1.2}>
    <mesh rotation={[Math.PI / 2.4, 0, 0]}>
      <torusGeometry args={[1, 0.35, 64, 128]} />
      <MeshDistortMaterial
        distort={0.25 + load * 0.25}
        speed={1.5}
        color={accentColor}
        roughness={0.15}
        metalness={0.6}
      />
    </mesh>
  </Float>
  <EffectComposer><Bloom intensity={0.6} luminanceThreshold={0.2} /></EffectComposer>
</Canvas>
```

Centro: número grande con el % global (CPU·RAM·disco ponderado).

### 3.2. `disk-tube.tsx`

Por cada disco, un **cilindro 3D vertical** con relleno animado proporcional a `used / total`. Color del relleno cambia si quedan <10% libres.

```tsx
<Canvas ...>
  <mesh>
    <cylinderGeometry args={[0.6, 0.6, 2, 32, 1, true]} />
    <meshStandardMaterial transparent opacity={0.15} color="#88a" />
  </mesh>
  <mesh position={[0, -1 + fillRatio, 0]} scale={[1, fillRatio, 1]}>
    <cylinderGeometry args={[0.58, 0.58, 2, 32]} />
    <meshStandardMaterial color={accentColor} emissive={accentColor} emissiveIntensity={0.4}/>
  </mesh>
</Canvas>
```

### 3.3. Animaciones de entrada

Framer Motion staggered, `delayChildren: 0.1` por card:

```tsx
const containerVariants = {
  hidden: { opacity: 0 },
  show: { opacity: 1, transition: { staggerChildren: 0.1 } },
};
const itemVariants = {
  hidden: { opacity: 0, y: 24, scale: 0.97 },
  show: { opacity: 1, y: 0, scale: 1, transition: { duration: 0.45, ease: [0.16, 1, 0.3, 1] } },
};
```

### 3.4. Hover tilt 3D (CSS + Framer)

```tsx
const x = useMotionValue(0);
const y = useMotionValue(0);
const rotateX = useTransform(y, [-50, 50], [8, -8]);
const rotateY = useTransform(x, [-50, 50], [-8, 8]);

<motion.div
  style={{ rotateX, rotateY, transformStyle: "preserve-3d", perspective: 1000 }}
  onMouseMove={(e) => {
    const r = e.currentTarget.getBoundingClientRect();
    x.set(e.clientX - r.left - r.width / 2);
    y.set(e.clientY - r.top - r.height / 2);
  }}
  onMouseLeave={() => { x.set(0); y.set(0); }}
>
  ...content...
</motion.div>
```

## 4. Backend — pipeline de telemetría

### 4.1. `platform/sysmon.rs`

```rust
use sysinfo::{System, ProcessRefreshKind, CpuRefreshKind, MemoryRefreshKind};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

pub fn snapshot() -> CpuMemSnapshot {
    let mut sys = SYSTEM.lock().unwrap();
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());
    sys.refresh_processes_specifics(ProcessRefreshKind::new()
        .with_cpu()
        .with_memory());
    // CPU% necesita 2 refreshes con MINIMUM_CPU_UPDATE_INTERVAL entre medias
    // — el polling cada 1.5s ya lo cubre, pero el primer arranque hace warm-up.
    CpuMemSnapshot { ... }
}
```

### 4.2. `platform/gpu.rs`

GPU usage usa **PDH** (Performance Data Helper) leyendo los contadores `\GPU Engine(*)\Utilization Percentage`. Es lo que el Task Manager hace internamente y funciona con NVIDIA, AMD e Intel sin requerir SDK del vendor.

- Memoria total y nombre por GPU → WMI `Win32_VideoController` (AdapterRAM, Name).
- Temperatura → best-effort vía WMI `MSAcpi_ThermalZoneTemperature` (root\WMI). Si la consulta falla o no devuelve nada, dejamos `temp_celsius: None`.

### 4.3. `domain/telemetry.rs`

Función pública única `snapshot() -> TelemetrySnapshot`:

```rust
pub struct TelemetrySnapshot {
    pub cpu_total_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub top_processes: Vec<ProcessInfo>,
    pub gpus: Vec<GpuInfo>,
    pub timestamp: String,
}
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}
pub struct GpuInfo {
    pub index: u32,
    pub name: String,
    pub usage_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temp_celsius: Option<f32>,
}
```

### 4.4. `ipc/telemetry.rs`

```rust
#[tauri::command]
pub async fn get_telemetry_snapshot() -> AppResult<TelemetrySnapshot> {
    tokio::task::spawn_blocking(domain::telemetry::snapshot).await
        .map_err(|e| AppError::External(format!("join: {e}")))?
}
```

(spawn_blocking porque `sysinfo` y PDH son síncronos y pueden tardar 50-200ms; no bloquear el reactor de Tauri.)

## 5. Frontend — hook + componentes

### 5.1. `src/hooks/use-telemetry.ts`

```ts
export function useTelemetry() {
  const isVisible = useDocumentVisibility(); // pause cuando ventana oculta
  return useQuery({
    queryKey: ["telemetry"],
    queryFn: getTelemetrySnapshot,
    refetchInterval: isVisible ? 1500 : false,
    staleTime: 0,
  });
}
```

### 5.2. Componentes nuevos en `src/features/home/components/`

| Componente               | Función                                                                |
|--------------------------|------------------------------------------------------------------------|
| `dashboard-hero-3d.tsx`  | Anillo 3D pulsante con % global                                        |
| `disk-tube.tsx`          | Cilindro 3D por disco                                                  |
| `cpu-line-chart.tsx`     | LineChart Recharts, buffer rotatorio 60 ticks                          |
| `ram-ring.tsx`           | RadialBarChart con % + números (X / Y GB)                              |
| `top-processes-table.tsx`| Tabla 5 filas con barras de progreso CPU/RAM por proceso               |
| `gpu-card.tsx`           | Card por GPU (uso, memoria, temp)                                      |
| `tilt-card.tsx`          | HOC con hover tilt 3D reutilizable                                     |

## 6. Layout final (`home-page.tsx`)

```tsx
<motion.div className="p-6 space-y-6 min-h-full bg-[image:var(--bg-gradient)]"
            variants={containerVariants} initial="hidden" animate="show">
  <div className="grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-6">
    <motion.div variants={itemVariants}>
      <DashboardHero3D loadPercent={loadGlobal} />
    </motion.div>
    <motion.div variants={itemVariants}>
      <TopProcessesTable processes={top5} />
    </motion.div>
  </div>
  <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
    <motion.div variants={itemVariants}><CpuLineChart history={cpu60s} /></motion.div>
    <motion.div variants={itemVariants}><RamRing used={ramUsed} total={ramTotal} /></motion.div>
    <motion.div variants={itemVariants}>
      <DiskTubes drives={summary.drives} />
    </motion.div>
  </div>
  <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 gap-6">
    {gpus.map(g => <GpuCard key={g.index} gpu={g} />)}
  </motion.div>
  <motion.div variants={itemVariants}><QuickActions /></motion.div>
</motion.div>
```

La tarjeta "Usuario" desaparece. Username + badge de elevación van al topbar del shell (`app-shell.tsx`).

## 7. Rendimiento

- **Lazy load del Canvas 3D**: `const DashboardHero3D = lazy(() => import("./components/dashboard-hero-3d"))` con `<Suspense fallback={<Skeleton />}>`. Solo se descarga al entrar a `/`.
- **dpr cap**: `<Canvas dpr={[1, 2]}>` evita renderizar a 4x en displays HiDPI.
- **frameloop="demand"** en canvas pequeños (disk-tube) — solo redibujan al cambiar props.
- **Polling pausado** si la ventana no está visible (`document.hidden`).

## 8. Verificación

```bash
npm install                # instala three, R3F, drei, framer-motion, recharts
cd src-tauri && cargo check
cd .. && npx tsc --noEmit
npm run tauri dev
```

Checklist en la app:
- [ ] Animación staggered al entrar al Dashboard.
- [ ] Anillo 3D rota suavemente, color cambia con la carga.
- [ ] Hover sobre cards muestra tilt 3D.
- [ ] CPU% actualiza cada 1.5s.
- [ ] RAM ring coincide con Task Manager (±1%).
- [ ] Top 5 procesos coinciden razonablemente con Task Manager.
- [ ] GPU(s) detectadas. Si no hay temp disponible, aparece "—" sin romper.
- [ ] Username pequeño en topbar a la derecha + badge "Administrador" o "Modo limitado".
- [ ] La pestaña de "Caché" (Parte 1) sigue funcionando — no toqué nada suyo.
- [ ] Resize de la ventana mantiene layout sin scroll horizontal.

## 9. Fuera de scope (próxima iteración)

- Light theme (la paleta está pensada para dark).
- Histórico persistente (al cerrar la app se pierde la línea de CPU).
- Alertas/notificaciones cuando una métrica supera umbral.
- Configurar refresh interval desde Settings.
- Modo "performance" (apaga 3D y deja la versión 2D) para máquinas viejas.
