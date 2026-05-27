# 11 — Visualización 3D en la consola (R3F)

> **Severidad:** 🟢 P2 — el toque "bonito". La consola funciona sin
> esto, pero el usuario pidió "animaciones 3D y esperas".
> **Modelo:** Sonnet 4.6.
> **Bloque:** 3 de 4 del set "consola de limpieza".
> **Depende de:** [`10-clean-console-frontend.md`](10-clean-console-frontend.md)
> (existe `CleanConsoleHeader` con un slot para el visualizer).

## 1. Problema

El header de la consola necesita un foco visual que dé "vida" durante
la espera. Un anillo CSS plano cumple, pero ClearTool ya invierte en
R3F (dashboard 3D, disk-tube) — coherente meter un toque 3D aquí.

Restricciones obligatorias:

- **No volver a romper** lo del doc 05: `frameloop="demand"`,
  `powerPreference: "low-power"`, listeners de context-lost.
- **Fallback 2D** si WebGL no está disponible.
- **No dominar el header** — máx 120×120 px.
- **Reaccionar al progreso real** (`bytesFreed / totalEstimatedBytes`)
  y a la fase (cambio de color/intensidad).

## 2. Diseño visual elegido

**Anillo de progreso 3D con partículas y halo**. Visual:

```
        ╭─ halo difuso (post-processing fake con sphere translúcida)
        │
       ╭───╮       ← anillo principal (torus geom)
      ╱  %  ╲      ← número del porcentaje superpuesto en HTML
     │   42  │
      ╲     ╱
       ╰───╯
       ✨ ✨ ✨   ← partículas que orbitan el anillo
        ▲ ▲ ▲   ← micro-partículas escapando hacia abajo cuando
                  el porcentaje sube ("liberando bytes")
```

Por qué esta variante en vez de un tubo o partículas-puras:

| Variante | Pros | Contras |
|---|---|---|
| Tubo que se vacía | Metáfora literal | No transmite "trabajo en curso" cuando está parado |
| Partículas dispersas | Bonito | Difícil leer el progreso real |
| **Anillo + halo + partículas** | Combina progreso + actividad + estado | Más componentes a sincronizar |

## 3. Archivos nuevos

| Path | Tipo |
|---|---|
| `src/features/cache-cleaner/clean-visualizer.tsx` | NUEVO — `<CleanVisualizer />` |
| `src/features/cache-cleaner/clean-visualizer-scene.tsx` | NUEVO — escena R3F |
| `src/features/cache-cleaner/clean-visualizer-fallback.tsx` | NUEVO — SVG fallback |
| `src/features/cache-cleaner/clean-console-header.tsx` | MODIFICADO — meter el visualizer |

## 4. API del componente

```ts
interface CleanVisualizerProps {
  /** 0..100. */
  percent: number;
  /** Mismo enum que el backend. */
  phase:
    | "preparing"
    | "creatingRestorePoint"
    | "closingProcesses"
    | "cleaning"
    | "schedulingReboot"
    | "verifying"
    | "complete"
    | "failed";
  /** Bytes/s. Modula intensidad de partículas. */
  throughputBytesPerSec: number;
  size?: number; // default 120
}
```

## 5. Colores por fase

```ts
const PHASE_COLOR: Record<string, string> = {
  preparing: "#64748b",             // slate
  creatingRestorePoint: "#a78bfa",  // violeta — recordar al usuario que es "seguro"
  closingProcesses: "#fbbf24",      // ámbar — atención
  cleaning: "#10b981",              // emerald — verde "limpiando"
  schedulingReboot: "#f59e0b",      // amber — diferido
  verifying: "#22d3ee",             // cyan — comprobando
  complete: "#34d399",              // emerald brillante
  failed: "#ef4444",                // rose
};
```

## 6. Detección de WebGL (fallback)

`src/features/cache-cleaner/clean-visualizer.tsx`:

```tsx
import { useMemo } from "react";
import { CleanVisualizerScene } from "./clean-visualizer-scene";
import { CleanVisualizerFallback } from "./clean-visualizer-fallback";

function hasWebGL(): boolean {
  try {
    const canvas = document.createElement("canvas");
    return !!(canvas.getContext("webgl2") || canvas.getContext("webgl"));
  } catch {
    return false;
  }
}

interface Props {
  percent: number;
  phase: string;
  throughputBytesPerSec: number;
  size?: number;
}

export function CleanVisualizer({
  percent,
  phase,
  throughputBytesPerSec,
  size = 120,
}: Props) {
  const webglOk = useMemo(() => hasWebGL(), []);

  if (!webglOk) {
    return <CleanVisualizerFallback percent={percent} phase={phase} size={size} />;
  }

  return (
    <CleanVisualizerScene
      percent={percent}
      phase={phase}
      throughputBytesPerSec={throughputBytesPerSec}
      size={size}
    />
  );
}
```

## 7. Escena R3F

`src/features/cache-cleaner/clean-visualizer-scene.tsx`:

```tsx
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { useEffect, useMemo, useRef } from "react";
import * as THREE from "three";

const PHASE_COLOR: Record<string, string> = {
  preparing: "#64748b",
  creatingRestorePoint: "#a78bfa",
  closingProcesses: "#fbbf24",
  cleaning: "#10b981",
  schedulingReboot: "#f59e0b",
  verifying: "#22d3ee",
  complete: "#34d399",
  failed: "#ef4444",
};

interface SceneProps {
  percent: number;
  phase: string;
  throughputBytesPerSec: number;
  size: number;
}

export function CleanVisualizerScene({
  percent,
  phase,
  throughputBytesPerSec,
  size,
}: SceneProps) {
  return (
    <div style={{ width: size, height: size }}>
      <Canvas
        camera={{ position: [0, 0, 2.6], fov: 50 }}
        gl={{
          powerPreference: "low-power",
          antialias: false,
          alpha: true,
          preserveDrawingBuffer: false,
        }}
        dpr={[1, 1.5]}
        frameloop="demand"
        onCreated={({ gl, invalidate }) => {
          gl.domElement.addEventListener("webglcontextlost", (e) => {
            e.preventDefault();
            console.warn("CleanVisualizer: WebGL context lost");
          });
          gl.domElement.addEventListener("webglcontextrestored", () => {
            invalidate();
          });
        }}
      >
        <ambientLight intensity={0.6} />
        <directionalLight position={[2, 2, 3]} intensity={1.1} />
        <ProgressRing percent={percent} phase={phase} />
        <Halo color={PHASE_COLOR[phase] ?? "#64748b"} percent={percent} />
        <OrbitingParticles
          throughputBytesPerSec={throughputBytesPerSec}
          phase={phase}
        />
      </Canvas>
    </div>
  );
}

// ── Anillo principal ────────────────────────────────────────────────

function ProgressRing({ percent, phase }: { percent: number; phase: string }) {
  const ringRef = useRef<THREE.Mesh>(null);
  const color = PHASE_COLOR[phase] ?? "#64748b";

  const { invalidate } = useThree();
  useEffect(() => { invalidate(); }, [percent, phase, invalidate]);

  useFrame((_, delta) => {
    if (!ringRef.current) return;
    ringRef.current.rotation.z += delta * 0.4; // rotación suave
    // En "cleaning" y "verifying" hay actividad → seguimos invalidando.
    if (phase === "cleaning" || phase === "verifying") {
      invalidate();
    }
  });

  // Geometría: torus con segmentos suficientes para verse suave a 120px.
  const geo = useMemo(() => new THREE.TorusGeometry(0.7, 0.08, 24, 80), []);

  // Visualizar el progreso: pintar el torus parcialmente con un anillo
  // adicional que cubre `percent / 100` del perímetro (truco con
  // material.opacity sobre 2 meshes, o con un shader). Para mantener
  // simple: dos torus, uno gris de fondo y uno con dashOffset variable.
  const fillRef = useRef<THREE.Mesh>(null);
  useFrame(() => {
    if (!fillRef.current) return;
    // Mostrar progreso girando un torus parcial — truco: rotar +
    // ocultar parte con un clipping plane no es trivial; aquí
    // animamos la opacidad como pulso suave para "respirar".
    const target = 0.4 + 0.6 * (percent / 100);
    const mat = fillRef.current.material as THREE.MeshStandardMaterial;
    mat.opacity = THREE.MathUtils.damp(mat.opacity, target, 4, 0.016);
  });

  return (
    <group ref={ringRef}>
      <mesh geometry={geo}>
        <meshStandardMaterial color="#1e293b" roughness={0.6} metalness={0.2} />
      </mesh>
      <mesh ref={fillRef} geometry={geo} scale={1.02}>
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={0.4}
          roughness={0.3}
          metalness={0.5}
          transparent
          opacity={0.4}
        />
      </mesh>
    </group>
  );
}

// ── Halo difuso ────────────────────────────────────────────────────

function Halo({ color, percent }: { color: string; percent: number }) {
  const meshRef = useRef<THREE.Mesh>(null);
  const { invalidate } = useThree();

  useFrame((state) => {
    if (!meshRef.current) return;
    const t = state.clock.elapsedTime;
    const scale = 1 + 0.06 * Math.sin(t * 1.8) + (percent / 100) * 0.1;
    meshRef.current.scale.setScalar(scale);
    // En idle no animar.
    if (percent > 0 && percent < 100) invalidate();
  });

  return (
    <mesh ref={meshRef}>
      <sphereGeometry args={[0.92, 24, 24]} />
      <meshBasicMaterial color={color} transparent opacity={0.06} />
    </mesh>
  );
}

// ── Partículas orbitando + escapando ──────────────────────────────

interface ParticleProps {
  throughputBytesPerSec: number;
  phase: string;
}

const PARTICLE_COUNT = 32;

function OrbitingParticles({ throughputBytesPerSec, phase }: ParticleProps) {
  const groupRef = useRef<THREE.Points>(null);
  const { invalidate } = useThree();

  // Una vez: posiciones aleatorias en órbita y velocidades por canal.
  const { positions, velocities } = useMemo(() => {
    const pos = new Float32Array(PARTICLE_COUNT * 3);
    const vel = new Float32Array(PARTICLE_COUNT * 3);
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      const theta = Math.random() * Math.PI * 2;
      const r = 0.72 + Math.random() * 0.18;
      pos[i * 3] = Math.cos(theta) * r;
      pos[i * 3 + 1] = Math.sin(theta) * r;
      pos[i * 3 + 2] = (Math.random() - 0.5) * 0.1;
      vel[i * 3] = -Math.sin(theta) * 0.4;
      vel[i * 3 + 1] = Math.cos(theta) * 0.4;
      vel[i * 3 + 2] = 0;
    }
    return { positions: pos, velocities: vel };
  }, []);

  // Intensidad de partículas escala con throughput (0..1 MB/s → 0; 100 MB/s → 1).
  const intensity = Math.min(
    1,
    throughputBytesPerSec / (100 * 1024 * 1024)
  );

  useFrame((_, delta) => {
    if (!groupRef.current) return;
    if (phase !== "cleaning" && phase !== "verifying" && phase !== "schedulingReboot") {
      return;
    }
    const pos = groupRef.current.geometry.attributes.position as THREE.BufferAttribute;
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      pos.array[i * 3] += velocities[i * 3] * delta * (0.4 + intensity);
      pos.array[i * 3 + 1] += velocities[i * 3 + 1] * delta * (0.4 + intensity);
      // Reenganchar a la órbita si se aleja demasiado.
      const x = pos.array[i * 3];
      const y = pos.array[i * 3 + 1];
      const r = Math.hypot(x, y);
      if (r > 1.1 || r < 0.5) {
        const theta = Math.atan2(y, x);
        const newR = 0.72 + Math.random() * 0.18;
        (pos.array as Float32Array)[i * 3] = Math.cos(theta) * newR;
        (pos.array as Float32Array)[i * 3 + 1] = Math.sin(theta) * newR;
      }
    }
    pos.needsUpdate = true;
    invalidate();
  });

  return (
    <points ref={groupRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={PARTICLE_COUNT}
          array={positions}
          itemSize={3}
        />
      </bufferGeometry>
      <pointsMaterial
        color={phase === "failed" ? "#ef4444" : "#a7f3d0"}
        size={0.04}
        sizeAttenuation
        transparent
        opacity={0.85}
      />
    </points>
  );
}
```

## 8. Fallback SVG (sin WebGL)

`src/features/cache-cleaner/clean-visualizer-fallback.tsx`:

```tsx
interface Props {
  percent: number;
  phase: string;
  size: number;
}

const PHASE_COLOR: Record<string, string> = {
  preparing: "#64748b",
  creatingRestorePoint: "#a78bfa",
  closingProcesses: "#fbbf24",
  cleaning: "#10b981",
  schedulingReboot: "#f59e0b",
  verifying: "#22d3ee",
  complete: "#34d399",
  failed: "#ef4444",
};

export function CleanVisualizerFallback({ percent, phase, size }: Props) {
  const radius = size / 2 - 8;
  const circ = 2 * Math.PI * radius;
  const offset = circ * (1 - percent / 100);
  const color = PHASE_COLOR[phase] ?? "#64748b";

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke="#1e293b"
        strokeWidth={6}
      />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke={color}
        strokeWidth={6}
        strokeDasharray={circ}
        strokeDashoffset={offset}
        strokeLinecap="round"
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
        style={{ transition: "stroke-dashoffset 200ms ease-out, stroke 300ms" }}
      />
    </svg>
  );
}
```

## 9. Integración en `CleanConsoleHeader`

Sustituir el placeholder del doc 10 por el visualizer real:

```tsx
import { CleanVisualizer } from "./clean-visualizer";

export function CleanConsoleHeader({ progress, phaseLabel }: Props) {
  const pct =
    progress.totalEstimatedBytes > 0
      ? Math.min(100, (progress.bytesFreed / progress.totalEstimatedBytes) * 100)
      : 0;

  return (
    <div className="flex gap-5">
      <CleanVisualizer
        percent={pct}
        phase={progress.phase}
        throughputBytesPerSec={progress.throughputBytesPerSec}
        size={120}
      />
      <div className="flex-1 space-y-3">
        {/* ... mismo bloque que en doc 10: títulos, ETA, barra ... */}
      </div>
    </div>
  );
}
```

Superponer el porcentaje en HTML (no en `<Text>` de drei) — más
accesible y simple:

```tsx
<div className="relative" style={{ width: 120, height: 120 }}>
  <CleanVisualizer percent={pct} phase={progress.phase} throughputBytesPerSec={progress.throughputBytesPerSec} size={120} />
  <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
    <span className="text-lg font-semibold tabular-nums text-ink-primary">
      {pct.toFixed(0)} %
    </span>
  </div>
</div>
```

## 10. Criterio de done

- [ ] `<CleanVisualizer>` renderiza 120×120 px sin tirar warnings de
      Three.js.
- [ ] Con WebGL OK, se ve anillo, halo y partículas orbitando.
- [ ] El número de porcentaje superpuesto se actualiza en tiempo real.
- [ ] El color cambia al cambiar de fase (verificar manualmente con
      cada fase del enum).
- [ ] Las partículas se intensifican con throughput alto y se
      ralentizan/paran cuando throughput=0 o `phase` no es de actividad.
- [ ] Sin WebGL, el fallback SVG muestra el anillo con animación CSS.
- [ ] No se ve `THREE.WebGLRenderer: Context Lost` repetido en consola
      durante una limpieza completa.
- [ ] El canvas pausa el render (`frameloop="demand"`) cuando no hay
      actividad — verificable en el frame-rate de devtools.

## 11. Riesgos / efectos secundarios

- **GPU compartida con el dashboard del Home**: si el usuario tiene
  Home renderizando un Canvas y ahora se abre la consola con otro
  Canvas, son 2 contextos WebGL activos. R3F y Three.js lo soportan,
  pero hardware viejo puede sufrir. Mitigación ya en doc 05: Home se
  desmonta al navegar fuera por React Router.
- **Bundle size**: Three.js + R3F ya están en el bundle (dashboard,
  disk-tube). No añade kilobytes nuevos significativos.
- **Animaciones costosas**: 32 partículas + 3 meshes es trivial. Si en
  futuras versiones aumenta, considerar `InstancedMesh` para las
  partículas.
- **Linter**: `pos.array` necesita el cast a `Float32Array` para
  poder mutar, ya está en el snippet. Si TypeScript se queja, usar
  `(pos.array as Float32Array)[i] = ...`.
- **Accesibilidad**: el visualizer es decorativo; el porcentaje
  numérico encima es la fuente de verdad para lectores de pantalla.
  Añadir `aria-hidden="true"` al `<div>` que envuelve el Canvas.
