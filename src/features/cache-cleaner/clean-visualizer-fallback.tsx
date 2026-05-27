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
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
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
