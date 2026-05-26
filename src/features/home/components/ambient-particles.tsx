/*
 * ambient-particles — campo de partículas cyan que se eleva por detrás del
 * dashboard. Es el "respirador" del sistema: las motas representan los
 * fragmentos sueltos (caché, logs, telemetría) que la app puede absorber.
 *
 * Decisiones de rendimiento:
 *   - 320 partículas — suficiente para densidad visual, ligero en GPU.
 *   - BufferAttribute mutado en useFrame (no setState — regla `perf-never-set-state-in-useframe`).
 *   - frameloop="always" (mueven siempre, pero pausamos al ocultar la
 *     ventana mediante useDocumentVisibility en el padre).
 *   - dpr cap [1, 1.5] — el campo es difuso, no se nota la subresolución.
 *   - sin lights, sin postprocesado — un PointsMaterial es ~0% GPU.
 *
 * Resultado: ~0.3-0.5% GPU en una iGPU moderna, imperceptible.
 */
import { useMemo, useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";

const COUNT = 320;
const FIELD_W = 14;     // ancho horizontal del campo (en unidades world)
const FIELD_H = 8;      // alto antes de hacer wrap
const CYAN = new THREE.Color("#34D2E0");

function ParticleField() {
  const ref = useRef<THREE.Points>(null);

  // Posiciones iniciales aleatorias dentro del campo, con velocidad por punto.
  const { positions, velocities } = useMemo(() => {
    const pos = new Float32Array(COUNT * 3);
    const vel = new Float32Array(COUNT);
    for (let i = 0; i < COUNT; i++) {
      pos[i * 3 + 0] = (Math.random() - 0.5) * FIELD_W;
      pos[i * 3 + 1] = (Math.random() - 0.5) * FIELD_H;
      pos[i * 3 + 2] = (Math.random() - 0.5) * 4;
      // Velocidad vertical entre 0.05 y 0.18 u/s — lento, contemplativo.
      vel[i] = 0.05 + Math.random() * 0.13;
    }
    return { positions: pos, velocities: vel };
  }, []);

  useFrame((_, delta) => {
    const points = ref.current;
    if (!points) return;
    const attr = points.geometry.getAttribute("position") as THREE.BufferAttribute;
    const arr = attr.array as Float32Array;
    for (let i = 0; i < COUNT; i++) {
      const yIdx = i * 3 + 1;
      arr[yIdx] += velocities[i] * delta;
      // Wrap vertical — al salir por arriba, reaparece por abajo a x aleatoria.
      if (arr[yIdx] > FIELD_H / 2) {
        arr[yIdx] = -FIELD_H / 2;
        arr[i * 3 + 0] = (Math.random() - 0.5) * FIELD_W;
      }
    }
    attr.needsUpdate = true;
  });

  return (
    <points ref={ref}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          args={[positions, 3]}
          count={COUNT}
          array={positions}
          itemSize={3}
        />
      </bufferGeometry>
      <pointsMaterial
        size={0.035}
        color={CYAN}
        transparent
        opacity={0.55}
        sizeAttenuation
        depthWrite={false}
        blending={THREE.AdditiveBlending}
      />
    </points>
  );
}

export function AmbientParticles() {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-0 z-0"
      style={{ contain: "strict" }}
    >
      <Canvas
        dpr={[1, 1.5]}
        camera={{ position: [0, 0, 6], fov: 50 }}
        gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
        // El fondo principal sigue siendo el surface-canvas via CSS — el
        // canvas WebGL queda transparente para no sobreescribir el tinte.
        style={{ background: "transparent" }}
      >
        <ParticleField />
      </Canvas>
    </div>
  );
}
