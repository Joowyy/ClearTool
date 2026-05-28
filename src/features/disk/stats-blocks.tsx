// stats-blocks — tarjetas superiores con métricas globales del análisis.
//
// Cuatro tarjetas en un grid:
//   1) Uso del disco (donut + breakdown libre / usado / analizado).
//   2) Totales del escaneo (carpetas, archivos, duración, errores).
//   3) Distribución por antigüedad (barras horizontales).
//   4) Categorías dominantes (donut por ExtCategory).
import { motion } from "framer-motion";
import {
  HardDrive,
  Files,
  Clock,
  AlertTriangle,
  PieChart,
  Layers,
} from "lucide-react";
import { useMemo } from "react";
import { formatBytes } from "../../lib/utils";
import { cn } from "../../lib/utils";
import type {
  AgeDistribution,
  DiskAnalysisReport,
  DriveListing,
  ExtCategory,
  ExtensionStat,
} from "../../api/types";
import { CATEGORY_COLORS, CATEGORY_LABELS } from "./disk-helpers";

interface StatsBlocksProps {
  report: DiskAnalysisReport | null;
  drive: DriveListing | null;
  scanning: boolean;
}

export function StatsBlocks({ report, drive, scanning }: StatsBlocksProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3">
      <DiskUsageCard report={report} drive={drive} scanning={scanning} />
      <ScanTotalsCard report={report} scanning={scanning} />
      <AgeDistributionCard report={report} scanning={scanning} />
      <CategoriesCard report={report} scanning={scanning} />
    </div>
  );
}

// ── shell común ───────────────────────────────────────────────────────────

function StatsCard({
  title,
  icon: Icon,
  children,
  delay = 0,
}: {
  title: string;
  icon: typeof HardDrive;
  children: React.ReactNode;
  delay?: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay, duration: 0.2, ease: "easeOut" }}
      className={cn(
        "relative panel-raised rounded-lg p-3.5 overflow-hidden",
        "border border-edge-default/10 hover:border-signal-cyan/20",
        "transition-colors duration-150",
      )}
    >
      <div
        aria-hidden
        className="absolute inset-0 pointer-events-none opacity-60"
        style={{
          background:
            "linear-gradient(135deg, rgba(34,211,238,0.06), rgba(99,102,241,0.04) 40%, transparent 70%)",
        }}
      />
      <div className="relative">
        <div className="flex items-center gap-2 mb-3">
          <Icon className="h-3.5 w-3.5 text-signal-cyan" strokeWidth={1.75} />
          <span className="text-[10px] uppercase tracking-wider font-semibold text-ink-tertiary">
            {title}
          </span>
        </div>
        {children}
      </div>
    </motion.div>
  );
}

// ── 1) uso del disco ──────────────────────────────────────────────────────

function DiskUsageCard({
  report,
  drive,
  scanning,
}: {
  report: DiskAnalysisReport | null;
  drive: DriveListing | null;
  scanning: boolean;
}) {
  const driveTotal = report?.driveTotalBytes ?? drive?.totalBytes ?? 0;
  const driveFree = report?.freeBytes ?? drive?.freeBytes ?? 0;
  const used = Math.max(0, driveTotal - driveFree);
  const scanned = report?.totalBytes ?? 0;
  const usedPct = driveTotal === 0 ? 0 : (used / driveTotal) * 100;

  return (
    <StatsCard title="Uso del disco" icon={HardDrive} delay={0}>
      <div className="flex items-center gap-3">
        <Donut percent={usedPct} accent="#06b6d4" />
        <div className="flex-1 min-w-0 space-y-1">
          <div className="font-mono text-base font-semibold text-ink-primary tabular-nums">
            {formatBytes(used)} <span className="text-ink-tertiary text-xs">/ {formatBytes(driveTotal)}</span>
          </div>
          <div className="text-[11px] text-ink-tertiary">
            {formatBytes(driveFree)} libres
          </div>
          {scanned > 0 && (
            <div className="text-[10px] text-ink-muted">
              Analizado: <span className="font-mono">{formatBytes(scanned)}</span>
              {scanned < used && (
                <span className="text-ink-muted">
                  {" "}· {formatBytes(used - scanned)} sin acceso
                </span>
              )}
            </div>
          )}
          {scanning && (
            <div className="text-[10px] text-signal-cyan animate-pulse">
              Analizando…
            </div>
          )}
        </div>
      </div>
    </StatsCard>
  );
}

function Donut({ percent, accent }: { percent: number; accent: string }) {
  const r = 28;
  const c = 2 * Math.PI * r;
  const pct = Math.max(0, Math.min(100, percent));
  const dash = (pct / 100) * c;

  return (
    <div className="relative w-[72px] h-[72px] flex-shrink-0">
      <svg viewBox="0 0 72 72" className="w-full h-full -rotate-90">
        <circle
          cx={36}
          cy={36}
          r={r}
          stroke="rgba(255,255,255,0.06)"
          strokeWidth={6}
          fill="none"
        />
        <motion.circle
          cx={36}
          cy={36}
          r={r}
          stroke={accent}
          strokeWidth={6}
          strokeLinecap="round"
          fill="none"
          strokeDasharray={c}
          initial={{ strokeDashoffset: c }}
          animate={{ strokeDashoffset: c - dash }}
          transition={{ duration: 0.6, ease: "easeOut" }}
          style={{ filter: `drop-shadow(0 0 6px ${accent}aa)` }}
        />
      </svg>
      <div className="absolute inset-0 flex items-center justify-center text-[11px] font-mono font-semibold text-ink-primary">
        {Math.round(pct)}%
      </div>
    </div>
  );
}

// ── 2) totales del escaneo ────────────────────────────────────────────────

function ScanTotalsCard({
  report,
  scanning,
}: {
  report: DiskAnalysisReport | null;
  scanning: boolean;
}) {
  const dirs = report?.totalDirs ?? 0;
  const files = report?.totalFiles ?? 0;
  const durationMs = report?.durationMs ?? 0;
  const errors = report?.errors?.length ?? 0;
  return (
    <StatsCard title="Contenido analizado" icon={Files} delay={0.05}>
      <div className="grid grid-cols-2 gap-2 text-xs">
        <Stat label="Carpetas" value={dirs.toLocaleString("es-ES")} mono />
        <Stat label="Archivos" value={files.toLocaleString("es-ES")} mono />
        <Stat
          label="Duración"
          value={formatDuration(durationMs)}
          icon={Clock}
          mono
        />
        <Stat
          label="Errores"
          value={errors > 0 ? `${errors}` : "0"}
          icon={AlertTriangle}
          mono
          accent={errors > 0 ? "text-amber-400" : undefined}
        />
      </div>
      {scanning && (
        <div className="mt-2 text-[10px] text-signal-cyan animate-pulse">
          Recorrido en curso…
        </div>
      )}
    </StatsCard>
  );
}

function Stat({
  label,
  value,
  icon: Icon,
  mono = false,
  accent,
}: {
  label: string;
  value: string;
  icon?: typeof Clock;
  mono?: boolean;
  accent?: string;
}) {
  return (
    <div>
      <div className="flex items-center gap-1 text-[10px] uppercase tracking-wider text-ink-muted">
        {Icon && <Icon className="h-3 w-3" strokeWidth={1.75} />}
        {label}
      </div>
      <div
        className={cn(
          "mt-0.5 text-sm font-semibold text-ink-primary tabular-nums",
          mono && "font-mono",
          accent,
        )}
      >
        {value}
      </div>
    </div>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s} s`;
  const m = Math.floor(s / 60);
  const rs = s % 60;
  return `${m} min ${rs} s`;
}

// ── 3) distribución por antigüedad ────────────────────────────────────────

const AGE_LABELS: { key: keyof AgeDistribution; label: string }[] = [
  { key: "last7Days", label: "< 7 días" },
  { key: "last30Days", label: "< 30 días" },
  { key: "last90Days", label: "< 90 días" },
  { key: "last1Year", label: "< 1 año" },
  { key: "last5Years", label: "< 5 años" },
  { key: "older", label: "> 5 años" },
];

const AGE_COLORS = ["#22d3ee", "#06b6d4", "#3b82f6", "#6366f1", "#a855f7", "#71717a"];

function AgeDistributionCard({
  report,
  scanning,
}: {
  report: DiskAnalysisReport | null;
  scanning: boolean;
}) {
  const total = useMemo(() => {
    if (!report) return 0;
    return AGE_LABELS.reduce(
      (s, b) => s + report.ageDistribution[b.key].bytes,
      0,
    );
  }, [report]);

  return (
    <StatsCard title="Antigüedad" icon={Clock} delay={0.1}>
      <div className="space-y-1.5">
        {AGE_LABELS.map((b, i) => {
          const bytes = report?.ageDistribution[b.key].bytes ?? 0;
          const pct = total > 0 ? (bytes / total) * 100 : 0;
          return (
            <div key={b.key} className="flex items-center gap-2 text-[11px]">
              <span className="w-16 text-ink-tertiary">{b.label}</span>
              <div className="flex-1 h-1.5 bg-edge-default/15 rounded-full overflow-hidden">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${pct}%` }}
                  transition={{ duration: 0.5, delay: 0.1 + i * 0.04 }}
                  className="h-full rounded-full"
                  style={{ backgroundColor: AGE_COLORS[i] }}
                />
              </div>
              <span className="w-10 text-right font-mono tabular-nums text-ink-tertiary">
                {pct.toFixed(0)}%
              </span>
            </div>
          );
        })}
      </div>
      {scanning && (
        <div className="mt-2 text-[10px] text-signal-cyan animate-pulse">
          Datos parciales mientras escanea…
        </div>
      )}
    </StatsCard>
  );
}

// ── 4) categorías dominantes ──────────────────────────────────────────────

function CategoriesCard({
  report,
  scanning,
}: {
  report: DiskAnalysisReport | null;
  scanning: boolean;
}) {
  const grouped = useMemo(() => {
    if (!report) return { entries: [] as Array<[ExtCategory, number]>, total: 0 };
    const acc = new Map<ExtCategory, number>();
    let total = 0;
    for (const e of report.topExtensions as ExtensionStat[]) {
      acc.set(e.category, (acc.get(e.category) ?? 0) + e.bytes);
      total += e.bytes;
    }
    const entries = Array.from(acc.entries()).sort((a, b) => b[1] - a[1]);
    return { entries, total };
  }, [report]);

  return (
    <StatsCard title="Categorías dominantes" icon={Layers} delay={0.15}>
      {grouped.entries.length === 0 ? (
        <div className="text-xs text-ink-tertiary">
          {scanning ? "Analizando categorías…" : "Sin datos todavía"}
        </div>
      ) : (
        <div className="flex items-center gap-3">
          <StackedBar entries={grouped.entries} total={grouped.total} />
          <div className="flex-1 space-y-1 text-[11px]">
            {grouped.entries.slice(0, 4).map(([cat, bytes]) => {
              const pct = grouped.total > 0 ? (bytes / grouped.total) * 100 : 0;
              return (
                <div key={cat} className="flex items-center gap-1.5">
                  <span
                    className="w-2 h-2 rounded-sm flex-shrink-0"
                    style={{ backgroundColor: CATEGORY_COLORS[cat] }}
                  />
                  <span className="text-ink-tertiary flex-1 truncate">
                    {CATEGORY_LABELS[cat]}
                  </span>
                  <span className="font-mono tabular-nums text-ink-secondary">
                    {pct.toFixed(0)}%
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </StatsCard>
  );
}

function StackedBar({
  entries,
  total,
}: {
  entries: Array<[ExtCategory, number]>;
  total: number;
}) {
  return (
    <div className="relative w-[64px] h-[64px] flex-shrink-0 flex items-center justify-center">
      <svg viewBox="0 0 64 64" className="w-full h-full -rotate-90">
        {entries.reduce<{ offset: number; nodes: React.ReactNode[] }>(
          (acc, [cat, bytes], idx) => {
            const pct = total > 0 ? bytes / total : 0;
            const r = 24;
            const c = 2 * Math.PI * r;
            const dash = pct * c;
            const offset = acc.offset;
            acc.nodes.push(
              <circle
                key={cat + idx}
                cx={32}
                cy={32}
                r={r}
                stroke={CATEGORY_COLORS[cat]}
                strokeWidth={8}
                strokeDasharray={`${dash} ${c - dash}`}
                strokeDashoffset={-offset}
                fill="none"
              />,
            );
            return { offset: acc.offset + dash, nodes: acc.nodes };
          },
          { offset: 0, nodes: [] },
        ).nodes}
      </svg>
      <PieChart
        className="absolute inset-0 m-auto h-4 w-4 text-ink-muted/50"
        strokeWidth={1.5}
      />
    </div>
  );
}
