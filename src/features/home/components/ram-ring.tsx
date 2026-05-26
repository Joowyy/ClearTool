import {
  PolarAngleAxis,
  RadialBar,
  RadialBarChart,
  ResponsiveContainer,
} from "recharts";
import { MemoryStick } from "lucide-react";
import { formatBytes } from "../../../lib/utils";

interface RamRingProps {
  usedBytes: number;
  totalBytes: number;
}

export function RamRing({ usedBytes, totalBytes }: RamRingProps) {
  const percent = totalBytes > 0 ? (usedBytes / totalBytes) * 100 : 0;
  const data = [{ name: "ram", value: percent, fill: ringColor(percent) }];

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <MemoryStick className="h-4 w-4" />
          Memoria
        </div>
        <span className="text-xs text-muted-foreground tabular-nums">
          {formatBytes(usedBytes)} / {formatBytes(totalBytes)}
        </span>
      </div>
      <div className="flex-1 min-h-[140px] relative">
        <ResponsiveContainer width="100%" height="100%">
          <RadialBarChart
            innerRadius="72%"
            outerRadius="100%"
            data={data}
            startAngle={90}
            endAngle={-270}
          >
            <PolarAngleAxis type="number" domain={[0, 100]} tick={false} />
            <RadialBar
              background={{ fill: "hsla(220, 30%, 25%, 0.3)" }}
              dataKey="value"
              cornerRadius={8}
              isAnimationActive={false}
            />
          </RadialBarChart>
        </ResponsiveContainer>
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
          <span className="text-3xl font-bold tabular-nums">{percent.toFixed(0)}%</span>
          <span className="text-xs text-muted-foreground">usada</span>
        </div>
      </div>
    </div>
  );
}

function ringColor(p: number): string {
  if (p >= 85) return "hsl(var(--accent-red))";
  if (p >= 60) return "hsl(var(--accent-amber))";
  return "hsl(var(--accent-cyan))";
}
