// @ts-nocheck — R3F v8 intrinsic elements no se augmentan en React 19.
import { Canvas } from "@react-three/fiber";
import { useMemo } from "react";
import * as THREE from "three";
import type { DriveInfo } from "../../../api";
import { formatBytes } from "../../../lib/utils";

interface DiskTubeProps {
  drive: DriveInfo;
}

function Tube({ fill }: { fill: number }) {
  // fill: 0..1
  const fillColor = useMemo<THREE.Color>(() => {
    const c = new THREE.Color();
    const hue = fill > 0.9 ? 0 : fill > 0.7 ? 0.08 : 0.55;
    c.setHSL(hue, 0.85, 0.55);
    return c;
  }, [fill]);

  const fillHeight = Math.max(0.001, Math.min(1, fill)) * 2.0;
  const fillY = -1.0 + fillHeight / 2;

  return (
    <group>
      {/* contorno transparente */}
      <mesh>
        <cylinderGeometry args={[0.55, 0.55, 2.0, 32, 1, true]} />
        <meshStandardMaterial
          color="#445"
          transparent
          opacity={0.18}
          side={THREE.DoubleSide}
          roughness={0.4}
          metalness={0.2}
        />
      </mesh>
      {/* relleno */}
      <mesh position={[0, fillY, 0]}>
        <cylinderGeometry args={[0.5, 0.5, fillHeight, 32]} />
        <meshStandardMaterial
          color={fillColor}
          emissive={fillColor}
          emissiveIntensity={0.35}
          roughness={0.2}
          metalness={0.4}
        />
      </mesh>
      {/* tapa inferior */}
      <mesh position={[0, -1.02, 0]} rotation={[Math.PI / 2, 0, 0]}>
        <ringGeometry args={[0.5, 0.56, 32]} />
        <meshBasicMaterial color={fillColor} />
      </mesh>
    </group>
  );
}

export function DiskTube({ drive }: DiskTubeProps) {
  const used = Math.max(0, drive.totalBytes - drive.freeBytes);
  const fill = drive.totalBytes > 0 ? used / drive.totalBytes : 0;
  const pct = (fill * 100).toFixed(0);

  return (
    <div className="flex items-center gap-3">
      <div className="w-16 h-24 flex-shrink-0">
        <Canvas
          camera={{ position: [0, 0.3, 3.4], fov: 30 }}
          dpr={[1, 2]}
          gl={{ antialias: true, alpha: true }}
          frameloop="demand"
        >
          <ambientLight intensity={0.6} />
          <pointLight position={[3, 3, 3]} intensity={1.0} />
          <Tube fill={fill} />
        </Canvas>
      </div>
      <div className="min-w-0 flex-1">
        <div className="font-medium text-sm">Disco {drive.letter}</div>
        <div className="text-xs text-muted-foreground tabular-nums truncate">
          {formatBytes(used)} / {formatBytes(drive.totalBytes)}
        </div>
        <div className="text-xs text-muted-foreground tabular-nums">{pct}% usado</div>
      </div>
    </div>
  );
}
