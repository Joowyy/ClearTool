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

export function CleanVisualizer({ percent, phase, throughputBytesPerSec, size = 120 }: Props) {
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
