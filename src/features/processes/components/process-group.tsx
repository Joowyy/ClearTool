import { useState } from "react";
import { ChevronRight, ChevronDown } from "lucide-react";
import { ProcessRow } from "./process-row";
import type { ProcessInfo, ProcessCategory } from "../../../api";

const CATEGORY_LABELS: Record<string, string> = {
  browser: "Browsers",
  media: "Media",
  communication: "Comunicaci\u00f3n",
  development: "Desarrollo",
  "user-app": "Aplicaciones",
  background: "Background / UWP",
  service: "Servicios (svchost)",
  system: "Sistema",
  unknown: "Otros",
};

const CATEGORY_ICONS: Record<string, string> = {
  browser: "\ud83c\udf10",
  media: "\ud83c\udfb5",
  communication: "\ud83d\udcac",
  development: "\ud83d\udee0\ufe0f",
  "user-app": "\ud83d\udcbc",
  background: "\u2699\ufe0f",
  service: "\u2699\ufe0f",
  system: "\ud83d\udd12",
  unknown: "\u2753",
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export function ProcessGroup({ category, processes }: {
  category: ProcessCategory; processes: ProcessInfo[]
}) {
  const [expanded, setExpanded] = useState(category !== "system");
  const total = processes.length;
  const totalRam = processes.reduce((s, p) => s + p.memoryBytes, 0);
  const catKey = category as string;

  return (
    <div className="border border-border rounded-lg">
      <button
        onClick={() => setExpanded(v => !v)}
        className="w-full flex items-center gap-2 p-3 hover:bg-accent/30"
      >
        {expanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        <span className="text-lg">{CATEGORY_ICONS[catKey] ?? "\u2753"}</span>
        <span className="font-medium">{CATEGORY_LABELS[catKey] ?? catKey}</span>
        <span className="ml-auto text-sm text-muted-foreground">
          {total} &middot; {formatBytes(totalRam)}
        </span>
      </button>
      {expanded && (
        <div className="divide-y divide-border/50">
          {processes
            .sort((a, b) => b.memoryBytes - a.memoryBytes)
            .map(p => <ProcessRow key={p.pid} process={p} />)}
        </div>
      )}
    </div>
  );
}
