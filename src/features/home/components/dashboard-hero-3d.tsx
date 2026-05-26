// @ts-nocheck — R3F v8 intrinsic elements (<mesh>, <pointLight>, etc.) no se
// augmentan correctamente en JSX namespace con React 19. Eliminar cuando
// R3F v9 estable salga.
import { Canvas, useFrame } from "@react-three/fiber";
import { Float, MeshDistortMaterial } from "@react-three/drei";
import { useMemo, useRef } from "react";
import type { Mesh, Color } from "three";
import * as THREE from "three";

interface DashboardHero3DProps {
  /// 0-100. Determina color (verde→ámbar→rojo) y nivel de distorsión.
  loadPercent: number;
}

function HeroTorus({ loadPercent }: { loadPercent: number }) {
  const meshRef = useRef<Mesh>(null);
  const color = useMemo<Color>(() => {
    // verde (cyan) → ámbar → rojo según carga
    const t = Math.min(1, loadPercent / 100);
    const hue = (1 - t) * 0.55; // 0.55 = cyan → 0 = rojo
    const c = new THREE.Color();
    c.setHSL(hue, 0.85, 0.55);
    return c;
  }, [loadPercent]);

  useFrame((_, delta) => {
    if (meshRef.current) {
      meshRef.current.rotation.y += delta * 0.25;
      meshRef.current.rotation.x += delta * 0.05;
    }
  });

  return (
    <Float speed={1.2} rotationIntensity={0.4} floatIntensity={1.2}>
      <mesh ref={meshRef} rotation={[Math.PI / 2.4, 0, 0]}>
        <torusGeometry args={[1.05, 0.32, 64, 128]} />
        <MeshDistortMaterial
          color={color}
          distort={0.18 + (loadPercent / 100) * 0.22}
          speed={1.8}
          roughness={0.18}
          metalness={0.65}
        />
      </mesh>
    </Float>
  );
}

export function DashboardHero3D({ loadPercent }: DashboardHero3DProps) {
  return (
    <div className="relative w-full h-[260px]">
      <Canvas
        camera={{ position: [0, 0, 3.6], fov: 50 }}
        dpr={[1, 2]}
        gl={{ antialias: true, alpha: true }}
      >
        <ambientLight intensity={0.45} />
        <pointLight position={[5, 5, 5]} intensity={1.4} color="#8aa8ff" />
        <pointLight position={[-5, -3, 4]} intensity={0.8} color="#a070ff" />
        <HeroTorus loadPercent={loadPercent} />
      </Canvas>

      <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
        <span className="text-5xl font-bold tabular-nums tracking-tight">
          {loadPercent.toFixed(0)}%
        </span>
        <span className="text-xs text-muted-foreground uppercase tracking-widest mt-1">
          carga global
        </span>
      </div>
    </div>
  );
}

export default DashboardHero3D;
