import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { FileText, RefreshCw, AlertTriangle, RotateCcw, CheckCircle, XCircle } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { listAuditLog, revertAuditEntry, type AuditEntry, type ReverseRecipe } from "../../api";
import { formatDate } from "../../lib/utils";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";
import { formatError } from "../../lib/errors";

function moduleVariant(module: string): "default" | "secondary" | "warning" | "destructive" {
  const m = module.toLowerCase();
  if (m.includes("debloat")) return "destructive";
  if (m.includes("registry")) return "warning";
  if (m.includes("service")) return "secondary";
  return "default";
}

function renderRecipe(recipe: ReverseRecipe): string {
  switch (recipe.kind) {
    case "Registry":
      return `Registry (${recipe.operations.length} ops)`;
    case "Service":
      return `Service: ${recipe.serviceName} → ${recipe.previousStartType}`;
    case "AppxReinstall":
      return `Reinstall: ${recipe.packageFamilyName}`;
    case "Noop":
      return `No reversible: ${recipe.reason}`;
  }
}

export function AuditPage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const qc = useQueryClient();
  const [expandedRunId, setExpandedRunId] = useState<string | null>(null);

  const {
    data: entries = [],
    isLoading,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ["audit-log"],
    queryFn: listAuditLog,
  });

  const revertMutation = useMutation({
    mutationFn: (runId: string) => revertAuditEntry(runId),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["audit-log"] });
    },
  });

  const groupedByRun = entries.reduce<Record<string, AuditEntry[]>>((acc, entry) => {
    if (!acc[entry.runId]) acc[entry.runId] = [];
    acc[entry.runId].push(entry);
    return acc;
  }, {});

  const runs = Object.entries(groupedByRun).sort(
    ([, a], [, b]) => new Date(b[0].timestamp).getTime() - new Date(a[0].timestamp).getTime()
  );

  if (isLoading) {
    return <div className="p-6 text-muted-foreground">Cargando registro de auditoría...</div>;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Registro de Auditoría</h2>
          <p className="text-muted-foreground text-sm">
            {entries.length} operaciones registradas · {runs.length} ejecuciones
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => void refetch()} disabled={isFetching}>
          <RefreshCw className={`h-4 w-4 mr-1 ${isFetching ? "animate-spin" : ""}`} />
          Actualizar
        </Button>
      </div>

      {!isElevated && (
        <div className="flex items-center gap-2 p-3 bg-yellow-900/30 border border-yellow-700/50 rounded-lg text-yellow-400 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          Se requieren permisos de administrador para revertir operaciones.
        </div>
      )}

      {runs.length === 0 ? (
        <EmptyState
          icon={FileText}
          title="Sin registros"
          description="Aún no se han realizado operaciones destructivas"
        />
      ) : (
        <div className="flex-1 overflow-auto space-y-3">
          {runs.map(([runId, runEntries]) => {
            const first = runEntries[0];
            const isExpanded = expandedRunId === runId;
            const allSuccess = runEntries.every((e) => e.status === "success");
            const hasReversible = runEntries.some((e) => e.reverseRecipe.kind !== "Noop");

            return (
              <div
                key={runId}
                className="border border-border rounded-lg overflow-hidden"
              >
                <div
                  className="flex items-center justify-between p-4 cursor-pointer hover:bg-accent/30"
                  onClick={() => setExpandedRunId(isExpanded ? null : runId)}
                >
                  <div className="flex items-center gap-3">
                    {allSuccess ? (
                      <CheckCircle className="h-5 w-5 text-signal-emerald" />
                    ) : (
                      <XCircle className="h-5 w-5 text-signal-red" />
                    )}
                    <div>
                      <div className="font-medium text-sm">
                        {first.module} — {first.operation}
                        {first.dryRun && <span className="ml-2 text-xs text-muted-foreground">(dry-run)</span>}
                      </div>
                      <div className="text-xs text-muted-foreground font-mono">
                        {runId}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="text-right text-xs text-muted-foreground">
                      <div>{formatDate(first.timestamp)}</div>
                      <div>{runEntries.length} entrada{runEntries.length > 1 ? "s" : ""} · {first.itemsAffected.length} items</div>
                    </div>
                    {first.restorePointSeq !== null && (
                      <Badge variant="secondary">RP #{first.restorePointSeq}</Badge>
                    )}
                  </div>
                </div>

                {isExpanded && (
                  <div className="border-t border-border bg-surface-1/50">
                    <table className="w-full text-xs">
                      <thead className="bg-card border-b border-border">
                        <tr>
                          <th className="text-left p-2 font-medium text-muted-foreground">Operación</th>
                          <th className="text-left p-2 font-medium text-muted-foreground">Items</th>
                          <th className="text-left p-2 font-medium text-muted-foreground">Reversa</th>
                          <th className="text-left p-2 font-medium text-muted-foreground">Estado</th>
                          <th className="text-right p-2 font-medium text-muted-foreground">Acción</th>
                        </tr>
                      </thead>
                      <tbody>
                        {runEntries.map((entry, idx) => (
                          <tr key={idx} className="border-b border-border/50">
                            <td className="p-2">
                              <Badge variant={moduleVariant(entry.module)}>{entry.module}</Badge>
                              <span className="ml-2">{entry.operation}</span>
                            </td>
                            <td className="p-2 text-muted-foreground">{entry.itemsAffected.length}</td>
                            <td className="p-2 font-mono text-muted-foreground">
                              {renderRecipe(entry.reverseRecipe)}
                            </td>
                            <td className="p-2">
                              {entry.status === "success" ? (
                                <span className="text-signal-emerald">OK</span>
                              ) : (
                                <span className="text-signal-red" title={entry.error ?? undefined}>
                                  Error
                                </span>
                              )}
                            </td>
                            <td className="p-2 text-right">
                              {hasReversible && entry.reverseRecipe.kind !== "Noop" && (
                                <Button
                                  variant="outline"
                                  size="sm"
                                  disabled={!isElevated || revertMutation.isPending}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    revertMutation.mutate(runId);
                                  }}
                                  title={!isElevated ? "Requiere administrador" : undefined}
                                >
                                  <RotateCcw className="h-3 w-3 mr-1" />
                                  Revertir
                                </Button>
                              )}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {revertMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Operación revertida correctamente.
        </div>
      )}

      {revertMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error al revertir: {formatError(revertMutation.error)}
        </div>
      )}
    </div>
  );
}
