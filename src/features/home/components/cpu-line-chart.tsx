import {
  Area,
  AreaChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { Cpu } from "lucide-react";

interface CpuLineChartProps {
  history: number[];
  totalPercent: number;
}

/// Línea de CPU% últimos N ticks. El historial viene precalculado del hook
/// `useTelemetry`. Se renderiza con un gradiente sutil bajo la curva.
export function CpuLineChart({ history, totalPercent }: CpuLineChartProps) {
  const data = history.map((value, idx) => ({ idx, value }));

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Cpu className="h-4 w-4" />
          CPU
        </div>
        <span className="text-2xl font-bold tabular-nums">
          {totalPercent.toFixed(1)}%
        </span>
      </div>
      <div className="flex-1 min-h-[140px]">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
            <defs>
              <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="hsl(var(--accent-cyan))" stopOpacity={0.6} />
                <stop offset="100%" stopColor="hsl(var(--accent-cyan))" stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis dataKey="idx" hide />
            <YAxis domain={[0, 100]} hide />
            <Tooltip
              contentStyle={{
                background: "hsla(240, 30%, 8%, 0.95)",
                border: "1px solid hsla(220, 80%, 70%, 0.25)",
                borderRadius: 8,
                fontSize: 12,
              }}
              labelFormatter={() => ""}
              formatter={(v: number) => [`${v.toFixed(1)}%`, "CPU"]}
            />
            <Area
              type="monotone"
              dataKey="value"
              stroke="hsl(var(--accent-cyan))"
              strokeWidth={2}
              fill="url(#cpuGradient)"
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
