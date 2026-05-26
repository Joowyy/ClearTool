import { lazy, Suspense, useMemo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { HardDrive, Trash2, Package, Settings2, RotateCcw } from "lucide-react";
import { useSystemSummary } from "../../hooks/use-system-summary";
import { useTelemetry } from "../../hooks/use-telemetry";
import { ROUTES } from "../../lib/routes";
import { TiltCard } from "./components/tilt-card";
import { CpuLineChart } from "./components/cpu-line-chart";
import { RamRing } from "./components/ram-ring";
import { TopProcessesTable } from "./components/top-processes-table";
import { GpuCard } from "./components/gpu-card";
import { DiskTube } from "./components/disk-tube";
import { AmbientParticles } from "./components/ambient-particles";

// Lazy-load del Canvas del hero — son ~450KB de three.js, sólo se cargan
// al entrar al Dashboard.
const DashboardHero3D = lazy(() =>
  import("./components/dashboard-hero-3d").then((m) => ({ default: m.DashboardHero3D })),
);

const containerVariants = {
  hidden: { opacity: 0 },
  show: { opacity: 1, transition: { staggerChildren: 0.07 } },
};
const itemVariants = {
  hidden: { opacity: 0, y: 24, scale: 0.98 },
  show: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { duration: 0.45, ease: [0.16, 1, 0.3, 1] as const },
  },
};

export function HomePage() {
  const { data: summary } = useSystemSummary();
  const { data: telemetry, cpuHistory } = useTelemetry();

  // Carga global: media ponderada de CPU + RAM (cada uno cuenta 50%).
  const loadGlobal = useMemo(() => {
    const cpu = telemetry?.cpuTotalPercent ?? 0;
    const ramPct = telemetry?.ramTotalBytes
      ? (telemetry.ramUsedBytes / telemetry.ramTotalBytes) * 100
      : 0;
    return (cpu + ramPct) / 2;
  }, [telemetry]);

  return (
    <div className="relative min-h-full bg-grid-fine">
      {/* Background sutil — solo en home, no compite con datos. */}
      <AmbientParticles />

      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="show"
        className="relative z-10 p-6 space-y-6"
      >
        <motion.div variants={itemVariants}>
          <div className="flex items-end justify-between flex-wrap gap-2">
            <div>
              <h2 className="text-2xl font-semibold tracking-tight text-ink-primary">
                Panel de control
              </h2>
              <p className="text-xs text-ink-tertiary font-mono mt-1">
                {summary?.osName ?? "Windows"} · build {summary?.buildNumber ?? "—"}
              </p>
            </div>
            <p className="text-2xs text-ink-tertiary tabular-nums font-mono">
              telemetría · refresh 1.5 s
            </p>
          </div>
        </motion.div>

      {/* Hero 3D + top procesos */}
      <div className="grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-6">
        <motion.div variants={itemVariants}>
          <TiltCard className="min-h-[260px] flex items-center justify-center">
            <Suspense
              fallback={
                <div className="h-[260px] flex items-center justify-center text-muted-foreground text-sm">
                  Cargando vista 3D…
                </div>
              }
            >
              <DashboardHero3D loadPercent={loadGlobal} />
            </Suspense>
          </TiltCard>
        </motion.div>
        <motion.div variants={itemVariants}>
          <TiltCard className="h-full">
            <TopProcessesTable processes={telemetry?.topProcesses ?? []} />
          </TiltCard>
        </motion.div>
      </div>

      {/* CPU + RAM + Discos */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <motion.div variants={itemVariants}>
          <TiltCard className="h-full min-h-[220px]">
            <CpuLineChart
              history={cpuHistory}
              totalPercent={telemetry?.cpuTotalPercent ?? 0}
            />
          </TiltCard>
        </motion.div>
        <motion.div variants={itemVariants}>
          <TiltCard className="h-full min-h-[220px]">
            <RamRing
              usedBytes={telemetry?.ramUsedBytes ?? 0}
              totalBytes={telemetry?.ramTotalBytes ?? 0}
            />
          </TiltCard>
        </motion.div>
        <motion.div variants={itemVariants}>
          <TiltCard className="h-full min-h-[220px]">
            <div className="flex items-center gap-2 text-sm text-muted-foreground mb-3">
              <HardDrive className="h-4 w-4" />
              Discos
            </div>
            <div className="space-y-3">
              {(summary?.drives ?? []).map((d) => (
                <DiskTube key={d.letter} drive={d} />
              ))}
              {(summary?.drives ?? []).length === 0 && (
                <p className="text-xs text-muted-foreground">Sin discos detectados</p>
              )}
            </div>
          </TiltCard>
        </motion.div>
      </div>

      {/* GPUs */}
      {(telemetry?.gpus ?? []).length > 0 && (
        <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {telemetry!.gpus.map((g) => (
            <TiltCard key={g.index}>
              <GpuCard gpu={g} />
            </TiltCard>
          ))}
        </motion.div>
      )}

        {/* Acciones rápidas */}
        <motion.div variants={itemVariants}>
          <h3 className="text-sm font-medium tracking-tight text-ink-secondary mb-3 uppercase">
            Acciones rápidas
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {[
              { to: ROUTES.CACHE, label: "Limpiar caché", icon: Trash2 },
              { to: ROUTES.DEBLOAT, label: "Eliminar bloatware", icon: Package },
              { to: ROUTES.SERVICES, label: "Gestionar servicios", icon: Settings2 },
              { to: ROUTES.RESTORE, label: "Puntos de restauración", icon: RotateCcw },
            ].map(({ to, label, icon: Icon }) => (
              <Link
                key={to}
                to={to}
                className="panel p-4 flex items-center gap-3 text-sm font-medium
                           text-ink-secondary hover:text-ink-primary
                           hover:border-signal-cyan/35 hover:bg-surface-2
                           transition-colors duration-160 ease-soft"
              >
                <Icon className="h-4 w-4 text-signal-cyan" strokeWidth={1.75} />
                <span>{label}</span>
              </Link>
            ))}
          </div>
        </motion.div>
      </motion.div>
    </div>
  );
}
