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

export function CleanVisualizerScene({ percent, phase, throughputBytesPerSec, size }: SceneProps) {
  return (
    <div style={{ width: size, height: size }} aria-hidden="true">
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
        <OrbitingParticles throughputBytesPerSec={throughputBytesPerSec} phase={phase} />
      </Canvas>
    </div>
  );
}

function ProgressRing({ percent, phase }: { percent: number; phase: string }) {
  const ringRef = useRef<THREE.Group>(null);
  const fillRef = useRef<THREE.Mesh>(null);
  const color = PHASE_COLOR[phase] ?? "#64748b";
  const { invalidate } = useThree();

  useEffect(() => {
    invalidate();
  }, [percent, phase, invalidate]);

  useFrame((_, delta) => {
    if (!ringRef.current) return;
    ringRef.current.rotation.z += delta * 0.4;
    if (phase === "cleaning" || phase === "verifying") {
      invalidate();
    }
  });

  useFrame(() => {
    if (!fillRef.current) return;
    const target = 0.4 + 0.6 * (percent / 100);
    const mat = fillRef.current.material as THREE.MeshStandardMaterial;
    mat.opacity = THREE.MathUtils.damp(mat.opacity, target, 4, 0.016);
  });

  const geo = useMemo(() => new THREE.TorusGeometry(0.7, 0.08, 24, 80), []);

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

function Halo({ color, percent }: { color: string; percent: number }) {
  const meshRef = useRef<THREE.Mesh>(null);
  const { invalidate } = useThree();

  useFrame((state) => {
    if (!meshRef.current) return;
    const t = state.clock.elapsedTime;
    const scale = 1 + 0.06 * Math.sin(t * 1.8) + (percent / 100) * 0.1;
    meshRef.current.scale.setScalar(scale);
    if (percent > 0 && percent < 100) invalidate();
  });

  return (
    <mesh ref={meshRef}>
      <sphereGeometry args={[0.92, 24, 24]} />
      <meshBasicMaterial color={color} transparent opacity={0.06} />
    </mesh>
  );
}

const PARTICLE_COUNT = 32;
const ACTIVE_PHASES = new Set(["cleaning", "verifying", "schedulingReboot"]);

function OrbitingParticles({
  throughputBytesPerSec,
  phase,
}: {
  throughputBytesPerSec: number;
  phase: string;
}) {
  const groupRef = useRef<THREE.Points>(null);
  const { invalidate } = useThree();

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

  const intensity = Math.min(1, throughputBytesPerSec / (100 * 1024 * 1024));

  useFrame((_, delta) => {
    if (!groupRef.current || !ACTIVE_PHASES.has(phase)) return;
    const pos = groupRef.current.geometry.attributes.position as THREE.BufferAttribute;
    const arr = pos.array as Float32Array;
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      arr[i * 3] += velocities[i * 3] * delta * (0.4 + intensity);
      arr[i * 3 + 1] += velocities[i * 3 + 1] * delta * (0.4 + intensity);
      const x = arr[i * 3];
      const y = arr[i * 3 + 1];
      const r = Math.hypot(x, y);
      if (r > 1.1 || r < 0.5) {
        const theta = Math.atan2(y, x);
        const newR = 0.72 + Math.random() * 0.18;
        arr[i * 3] = Math.cos(theta) * newR;
        arr[i * 3 + 1] = Math.sin(theta) * newR;
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
