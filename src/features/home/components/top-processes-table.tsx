import { Activity } from "lucide-react";
import { formatBytes } from "../../../lib/utils";
import type { TelemetryProcessInfo } from "../../../api";

interface TopProcessesTableProps {
  processes: TelemetryProcessInfo[];
}

export function TopProcessesTable({ processes }: TopProcessesTableProps) {
  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center gap-2 text-sm text-muted-foreground mb-3">
        <Activity className="h-4 w-4" />
        Top procesos
      </div>
      <ul className="space-y-2 flex-1">
        {processes.length === 0 && (
          <li className="text-xs text-muted-foreground">Recopilando…</li>
        )}
        {processes.map((p) => (
          <li key={p.pid} className="text-sm">
            <div className="flex items-center justify-between gap-3">
              <span className="truncate font-medium" title={p.name}>
                {p.name}
              </span>
              <span className="tabular-nums text-muted-foreground text-xs flex-shrink-0">
                {p.cpuPercent.toFixed(1)}% · {formatBytes(p.memoryBytes)}
              </span>
            </div>
            <div className="mt-1 h-1 rounded-full bg-muted overflow-hidden">
              <div
                className="h-full rounded-full"
                style={{
                  width: `${Math.min(100, p.cpuPercent)}%`,
                  background: `linear-gradient(90deg, hsl(var(--accent-cyan)), hsl(var(--accent-violet)))`,
                }}
              />
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}
