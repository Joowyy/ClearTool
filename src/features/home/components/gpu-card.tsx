import { Microchip } from "lucide-react";
import { formatBytes } from "../../../lib/utils";
import type { GpuInfo } from "../../../api";

interface GpuCardProps {
  gpu: GpuInfo;
}

export function GpuCard({ gpu }: GpuCardProps) {
  const usage = gpu.usagePercent ?? 0;
  const memPercent =
    gpu.memoryTotalBytes && gpu.memoryUsedBytes
      ? (gpu.memoryUsedBytes / gpu.memoryTotalBytes) * 100
      : 0;
  const memTotalLabel = gpu.memoryTotalBytes
    ? formatBytes(gpu.memoryTotalBytes)
    : "—";

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-2 min-w-0">
          <Microchip className="h-4 w-4 text-muted-foreground flex-shrink-0" />
          <div className="min-w-0">
            <div className="font-medium truncate text-sm" title={gpu.name}>
              {gpu.name}
            </div>
            <div className="text-xs text-muted-foreground">
              {gpu.vendor ?? "GPU"} #{gpu.index}
            </div>
          </div>
        </div>
        <div className="text-right flex-shrink-0">
          <div className="text-xl font-bold tabular-nums">
            {gpu.usagePercent != null ? `${usage.toFixed(0)}%` : "—"}
          </div>
          <div className="text-xs text-muted-foreground">uso</div>
        </div>
      </div>

      <div>
        <div className="flex justify-between text-xs text-muted-foreground mb-1">
          <span>Memoria</span>
          <span className="tabular-nums">
            {gpu.memoryUsedBytes ? formatBytes(gpu.memoryUsedBytes) : "—"} / {memTotalLabel}
          </span>
        </div>
        <div className="h-1.5 rounded-full bg-muted overflow-hidden">
          <div
            className="h-full rounded-full transition-all"
            style={{
              width: `${Math.min(100, memPercent)}%`,
              background: `linear-gradient(90deg, hsl(var(--accent-cyan)), hsl(var(--accent-violet)))`,
            }}
          />
        </div>
      </div>

      <div className="flex justify-between text-xs">
        <span className="text-muted-foreground">Temperatura</span>
        <span className="tabular-nums">
          {gpu.tempCelsius != null ? `${gpu.tempCelsius.toFixed(0)} °C` : "—"}
        </span>
      </div>
    </div>
  );
}
