import { useState, useCallback } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Trash2,
  ScanLine,
  AlertTriangle,
  Shield,
  SkipForward,
  CheckCircle2,
  Loader2,
  ArrowLeft,
  RefreshCw,
} from "lucide-react";
import { Button } from "../../components/ui/button";
import { Checkbox } from "../../components/ui/checkbox";
import { Badge } from "../../components/ui/badge";
import { formatBytes } from "../../lib/utils";
import {
  listCacheLocations,
  analyzeCacheLocations,
  executeCleanPlan,
  verifyClean,
  type CacheLocation,
  type CleanPlan,
  type CleanReportV2,
  type VerifyReport,
  type BlockedLocation,
  type PermissionLocation,
  type SkippedLocation,
  type ReadyLocation,
} from "../../api";
import { EmptyState } from "../../components/empty-state";
import { formatError } from "../../lib/errors";
import { toast } from "../../lib/toast";

function riskVariant(risk?: string | null): "success" | "warning" | "destructive" {
  const r = (risk ?? "").toLowerCase();
  if (r === "high" || r === "dangerous") return "destructive";
  if (r === "medium" || r === "moderate") return "warning";
  return "success";
}

function strategyLabel(strategy: string): string {
  const labels: Record<string, string> = {
    "direct-delete": "Borrado directo",
    "uwp-app-aware": "UWP consciente",
    "browser-aware": "Consciente del navegador",
    "process-locked": "Proceso bloqueando",
    "system-restart-required": "Reinicio requerido",
    "take-ownership-and-delete": "Tomar propiedad",
  };
  return labels[strategy] ?? strategy;
}

// ── SelectionView ──────────────────────────────────────────────────────

interface SelectionViewProps {
  catalog: CacheLocation[];
  selectedIds: Set<string>;
  onChange: (ids: Set<string>) => void;
  onAnalyze: () => void;
  isAnalyzing: boolean;
}

function SelectionView({
  catalog,
  selectedIds,
  onChange,
  onAnalyze,
  isAnalyzing,
}: SelectionViewProps) {
  const toggle = (id: string) => {
    const next = new Set(selectedIds);
    next.has(id) ? next.delete(id) : next.add(id);
    onChange(next);
  };

  const selectAll = () => onChange(new Set(catalog.map((l) => l.id)));
  const clearAll = () => onChange(new Set());

  return (
    <div className="flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between flex-wrap gap-2">
        <div>
          <h2 className="text-2xl font-bold">Limpieza de caché</h2>
          <p className="text-muted-foreground text-sm">
            {selectedIds.size} de {catalog.length} ubicaciones seleccionadas
          </p>
        </div>
        <div className="flex gap-2 flex-wrap">
          <Button variant="outline" size="sm" onClick={selectAll}>
            Seleccionar todo
          </Button>
          <Button variant="outline" size="sm" onClick={clearAll}>
            Limpiar selección
          </Button>
          <Button
            variant="default"
            size="sm"
            disabled={selectedIds.size === 0 || isAnalyzing}
            onClick={onAnalyze}
          >
            {isAnalyzing ? (
              <Loader2 className="h-4 w-4 mr-1 animate-spin" />
            ) : (
              <ScanLine className="h-4 w-4 mr-1" />
            )}
            Analizar
          </Button>
        </div>
      </div>

      {catalog.length === 0 ? (
        <EmptyState icon={Trash2} title="Sin ubicaciones de caché" />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="p-3 w-10">
                  <Checkbox
                    checked={selectedIds.size === catalog.length && catalog.length > 0}
                    onCheckedChange={(v) => (v ? selectAll() : clearAll())}
                  />
                </th>
                <th className="text-left p-3 font-medium text-muted-foreground">Ubicación</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Categoría</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Estrategia</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Riesgo</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Estimado</th>
              </tr>
            </thead>
            <tbody>
              {catalog.map((loc) => (
                <tr
                  key={loc.id}
                  className="border-b border-border/50 hover:bg-accent/30 cursor-pointer"
                  onClick={() => toggle(loc.id)}
                >
                  <td className="p-3" onClick={(e) => e.stopPropagation()}>
                    <Checkbox
                      checked={selectedIds.has(loc.id)}
                      onCheckedChange={() => toggle(loc.id)}
                    />
                  </td>
                  <td className="p-3">
                    <div className="font-medium">{loc.displayName}</div>
                    <div className="text-xs text-muted-foreground font-mono">{loc.path}</div>
                  </td>
                  <td className="p-3">
                    {loc.category ? (
                      <Badge variant="secondary">{loc.category}</Badge>
                    ) : (
                      <span className="text-xs text-muted-foreground">—</span>
                    )}
                  </td>
                  <td className="p-3 text-xs text-muted-foreground">
                    {loc.strategy ? strategyLabel(loc.strategy) : "—"}
                  </td>
                  <td className="p-3">
                    <Badge variant={riskVariant(loc.risk)}>{loc.risk ?? "safe"}</Badge>
                  </td>
                  <td className="p-3 text-right text-muted-foreground text-xs">
                    {loc.averageSize ?? "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// ── Section wrappers ───────────────────────────────────────────────────

function SectionCard({
  icon: Icon,
  title,
  count,
  totalBytes,
  color,
  children,
}: {
  icon: React.ElementType;
  title: string;
  count: number;
  totalBytes: number;
  color: string;
  children: React.ReactNode;
}) {
  const colorMap: Record<string, string> = {
    green: "border-green-700/50 bg-green-900/10",
    yellow: "border-yellow-700/50 bg-yellow-900/10",
    red: "border-red-700/50 bg-red-900/10",
    gray: "border-gray-700/50 bg-gray-900/10",
  };
  const iconColorMap: Record<string, string> = {
    green: "text-green-400",
    yellow: "text-yellow-400",
    red: "text-red-400",
    gray: "text-gray-400",
  };

  return (
    <details open className={`border ${colorMap[color]} rounded-lg`}>
      <summary className="p-3 cursor-pointer flex items-center gap-2">
        <Icon className={`h-5 w-5 ${iconColorMap[color]}`} />
        <span className="font-medium">
          {title} ({count})
        </span>
        <span className="ml-auto text-sm text-muted-foreground">
          {formatBytes(totalBytes)}
        </span>
      </summary>
      <div className="divide-y divide-border/50 px-2 pb-2">{children}</div>
    </details>
  );
}

function LocationRow({
  displayName,
  path,
  bytes,
  extra,
}: {
  displayName: string;
  path: string;
  bytes: number;
  extra?: React.ReactNode;
}) {
  return (
    <div className="p-3 flex items-center gap-3">
      <div className="flex-1 min-w-0">
        <div className="font-medium">{displayName}</div>
        <div className="text-xs text-muted-foreground font-mono truncate">{path}</div>
        {extra}
      </div>
      <div className="text-sm font-mono text-muted-foreground">{formatBytes(bytes)}</div>
    </div>
  );
}

// ── PlanView ───────────────────────────────────────────────────────────

interface PlanViewProps {
  plan: CleanPlan;
  onExecute: (opts: {
    autoCloseBlocking: boolean;
    scheduleBlockedForReboot: boolean;
    dryRun: boolean;
  }) => void;
  onCancel: () => void;
  onReAnalyze: () => void;
  isExecuting: boolean;
  report: CleanReportV2 | null;
  verify: VerifyReport | null;
}

function PlanView({
  plan,
  onExecute,
  onCancel,
  onReAnalyze,
  isExecuting,
  report,
  verify,
}: PlanViewProps) {
  const [autoClose, setAutoClose] = useState(false);
  const [scheduleReboot, setScheduleReboot] = useState(true);
  const [dryRun, setDryRun] = useState(false);

  return (
    <div className="flex flex-col gap-4 h-full">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="sm" onClick={onCancel} disabled={isExecuting}>
          <ArrowLeft className="h-4 w-4 mr-1" />
          Volver
        </Button>
        <div className="flex-1">
          <h2 className="text-2xl font-bold">Plan de limpieza</h2>
          <p className="text-muted-foreground text-sm">
            {plan.ready.length} listas · {plan.blocked.length} bloqueadas ·{" "}
            {plan.permissionIssues.length} permisos · {plan.skipped.length} omitidas
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={onReAnalyze} disabled={isExecuting}>
          <RefreshCw className="h-4 w-4 mr-1" />
          Re-analizar
        </Button>
      </div>

      <div className="flex gap-4 text-sm">
        <div className="px-3 py-1.5 rounded-lg bg-green-900/20 border border-green-800/40">
          <span className="text-green-400 font-medium">Listas: </span>
          <span className="text-muted-foreground">{formatBytes(plan.totalEstimatedBytes)}</span>
        </div>
        <div className="px-3 py-1.5 rounded-lg bg-yellow-900/20 border border-yellow-800/40">
          <span className="text-yellow-400 font-medium">Bloqueadas: </span>
          <span className="text-muted-foreground">{formatBytes(plan.totalBlockedBytes)}</span>
        </div>
      </div>

      <div className="flex-1 overflow-auto flex flex-col gap-3">
        {plan.ready.length > 0 && (
          <SectionCard
            icon={CheckCircle2}
            title="Listas para limpiar"
            count={plan.ready.length}
            totalBytes={plan.totalEstimatedBytes}
            color="green"
          >
            {plan.ready.map((item: ReadyLocation) => (
              <LocationRow
                key={item.id}
                displayName={item.displayName}
                path={item.resolvedPath}
                bytes={item.bytes}
                extra={
                  <div className="text-xs text-muted-foreground mt-0.5">
                    Estrategia: {strategyLabel(item.strategy)}
                    {item.ageOldestFile && ` · Archivo más antiguo: ${item.ageOldestFile}`}
                  </div>
                }
              />
            ))}
          </SectionCard>
        )}

        {plan.blocked.length > 0 && (
          <SectionCard
            icon={AlertTriangle}
            title="Bloqueadas por procesos"
            count={plan.blocked.length}
            totalBytes={plan.totalBlockedBytes}
            color="yellow"
          >
            {plan.blocked.map((item: BlockedLocation) => (
              <LocationRow
                key={item.id}
                displayName={item.displayName}
                path={item.resolvedPath}
                bytes={item.bytes}
                extra={
                  <div className="text-xs text-yellow-400/80 mt-0.5">
                    Bloqueado por:{" "}
                    {item.lockedBy.map((p) => `${p.name} (PID ${p.pid})`).join(", ")}
                  </div>
                }
              />
            ))}
          </SectionCard>
        )}

        {plan.permissionIssues.length > 0 && (
          <SectionCard
            icon={Shield}
            title="Problemas de permisos"
            count={plan.permissionIssues.length}
            totalBytes={plan.permissionIssues.reduce((s, i) => s + i.bytes, 0)}
            color="red"
          >
            {plan.permissionIssues.map((item: PermissionLocation) => (
              <LocationRow
                key={item.id}
                displayName={item.displayName}
                path={item.resolvedPath}
                bytes={item.bytes}
                extra={
                  <div className="text-xs text-red-400/80 mt-0.5 truncate">{item.reason}</div>
                }
              />
            ))}
          </SectionCard>
        )}

        {plan.skipped.length > 0 && (
          <SectionCard
            icon={SkipForward}
            title="Omitidas"
            count={plan.skipped.length}
            totalBytes={0}
            color="gray"
          >
            {plan.skipped.map((item: SkippedLocation) => (
              <LocationRow
                key={item.id}
                displayName={item.displayName}
                path=""
                bytes={0}
                extra={
                  <div className="text-xs text-muted-foreground mt-0.5">
                    {typeof item.reason === "string"
                      ? item.reason
                      : JSON.stringify(item.reason)}
                  </div>
                }
              />
            ))}
          </SectionCard>
        )}
      </div>

      {report && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Limpieza completada: {formatBytes(report.totalBytesFreed)} liberados,{" "}
          {report.perLocation.reduce((s, l) => s + l.filesDeleted, 0)} archivos eliminados.
          {report.totalBytesScheduledReboot > 0 &&
            ` ${formatBytes(report.totalBytesScheduledReboot)} programados para reboot.`}
        </div>
      )}

      {verify && verify.totalStillPresent > 0 && (
        <div className="p-3 bg-yellow-900/30 border border-yellow-700/50 rounded-lg text-yellow-400 text-sm">
          Verificación: {formatBytes(verify.totalStillPresent)} aún presentes tras la limpieza.
        </div>
      )}

      <div className="border-t border-border pt-3 flex flex-col gap-2">
        <div className="flex gap-4 text-sm flex-wrap">
          <label className="flex items-center gap-2 cursor-pointer">
            <Checkbox checked={autoClose} onCheckedChange={(v) => setAutoClose(!!v)} />
            <span>Cerrar procesos bloqueantes automáticamente</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <Checkbox checked={scheduleReboot} onCheckedChange={(v) => setScheduleReboot(!!v)} />
            <span>Programar bloqueados para reboot</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <Checkbox checked={dryRun} onCheckedChange={(v) => setDryRun(!!v)} />
            <span>Dry-run (simular)</span>
          </label>
        </div>
        <div className="flex gap-2 mt-1">
          <Button
            variant="default"
            onClick={() =>
              onExecute({ autoCloseBlocking: autoClose, scheduleBlockedForReboot: scheduleReboot, dryRun })
            }
            disabled={isExecuting || plan.ready.length === 0}
          >
            {isExecuting ? (
              <Loader2 className="h-4 w-4 mr-1 animate-spin" />
            ) : (
              <Trash2 className="h-4 w-4 mr-1" />
            )}
            {isExecuting
              ? "Ejecutando..."
              : dryRun
                ? `Simular (${formatBytes(plan.totalEstimatedBytes)})`
                : `Limpiar (${formatBytes(plan.totalEstimatedBytes)})`}
          </Button>
        </div>
      </div>
    </div>
  );
}

// ── CachePage ──────────────────────────────────────────────────────────

export function CachePage() {
  const qc = useQueryClient();
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [plan, setPlan] = useState<CleanPlan | null>(null);
  const [report, setReport] = useState<CleanReportV2 | null>(null);
  const [verify, setVerify] = useState<VerifyReport | null>(null);

  const {
    data: catalog = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ["cache-locations"],
    queryFn: listCacheLocations,
  });

  const analyzeMutation = useMutation({
    mutationFn: () => analyzeCacheLocations([...selectedIds]),
    onSuccess: (p) => {
      setPlan(p);
      setReport(null);
      setVerify(null);
    },
    onError: (err) => toast.error("Error analizando ubicaciones", { description: formatError(err) }),
  });

  const executeMutation = useMutation({
    mutationFn: async (opts: {
      autoCloseBlocking: boolean;
      scheduleBlockedForReboot: boolean;
      dryRun: boolean;
    }) => {
      if (!plan) throw new Error("No hay plan");
      const r = await executeCleanPlan(plan, {
        planId: plan.planId,
        autoCloseBlocking: opts.autoCloseBlocking,
        scheduleBlockedForReboot: opts.scheduleBlockedForReboot,
        dryRun: opts.dryRun,
        createRestorePoint: !opts.dryRun,
        timeoutPerLocationSecs: 60,
      });
      const v = await verifyClean(plan, r);
      return { report: r, verify: v, dryRun: opts.dryRun };
    },
    onSuccess: ({ report: r, verify: v, dryRun }) => {
      setReport(r);
      setVerify(v);
      if (!dryRun) {
        void qc.invalidateQueries({ queryKey: ["cache-locations"] });
      }
      toast.success(
        dryRun ? "Simulación completada" : `Liberados ${formatBytes(r.totalBytesFreed)}`,
        {
          description:
            v.totalStillPresent > 0
              ? `${formatBytes(v.totalStillPresent)} no se pudieron eliminar.`
              : undefined,
        }
      );
    },
    onError: (err) => toast.error("Error ejecutando limpieza", { description: formatError(err) }),
  });

  const handleAnalyze = useCallback(() => {
    analyzeMutation.mutate();
  }, [analyzeMutation]);

  const handleExecute = useCallback(
    (opts: { autoCloseBlocking: boolean; scheduleBlockedForReboot: boolean; dryRun: boolean }) => {
      executeMutation.mutate(opts);
    },
    [executeMutation]
  );

  if (isLoading) return <div className="p-6 text-muted-foreground">Cargando catálogo...</div>;

  if (error) {
    return (
      <div className="p-6">
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-300 text-sm whitespace-pre-wrap">
          Error al cargar el catálogo: {formatError(error)}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      {!plan ? (
        <SelectionView
          catalog={catalog}
          selectedIds={selectedIds}
          onChange={setSelectedIds}
          onAnalyze={handleAnalyze}
          isAnalyzing={analyzeMutation.isPending}
        />
      ) : (
        <PlanView
          plan={plan}
          onExecute={handleExecute}
          onCancel={() => setPlan(null)}
          onReAnalyze={handleAnalyze}
          isExecuting={executeMutation.isPending}
          report={report}
          verify={verify}
        />
      )}
    </div>
  );
}
