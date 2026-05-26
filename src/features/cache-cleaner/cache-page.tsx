import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Trash2, ScanLine } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Checkbox } from "../../components/ui/checkbox";
import { Badge } from "../../components/ui/badge";
import { formatBytes } from "../../lib/utils";
import {
  listCacheLocations,
  scanCacheLocations,
  cleanCacheLocations,
  type CacheLocation,
  type CacheScanReport,
} from "../../api";
import { EmptyState } from "../../components/empty-state";
import { formatError } from "../../lib/errors";

function riskVariant(risk?: string | null): "success" | "warning" | "destructive" {
  const r = (risk ?? "").toLowerCase();
  if (r === "high") return "destructive";
  if (r === "medium") return "warning";
  return "success";
}

export function CachePage() {
  const qc = useQueryClient();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [scanReports, setScanReports] = useState<Map<string, CacheScanReport>>(new Map());

  const {
    data: locations = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ["cache-locations"],
    queryFn: listCacheLocations,
  });

  const scanMutation = useMutation({
    mutationFn: (ids: string[]) => scanCacheLocations(ids),
    onSuccess: (reports) => {
      const map = new Map<string, CacheScanReport>();
      reports.forEach((r) => map.set(r.id, r));
      setScanReports(map);
    },
  });

  const cleanMutation = useMutation({
    mutationFn: () =>
      cleanCacheLocations({
        ids: Array.from(selected),
        dryRun: false,
        createRestorePoint: true,
        forceCloseProcesses: false,
      }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["cache-locations"] });
      setScanReports(new Map());
      setSelected(new Set());
    },
  });

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  const selectAll = () => setSelected(new Set(locations.map((l) => l.id)));
  const clearAll = () => setSelected(new Set());

  const totalBytes = Array.from(scanReports.values())
    .filter((r) => selected.has(r.id))
    .reduce((s, r) => s + r.bytesAfterFilters, 0);

  if (isLoading) return <div className="p-6 text-muted-foreground">Cargando...</div>;

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
      <div className="flex items-center justify-between flex-wrap gap-2">
        <div>
          <h2 className="text-2xl font-bold">Limpieza de caché</h2>
          <p className="text-muted-foreground text-sm">
            {selected.size} de {locations.length} ubicaciones seleccionadas
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
            variant="outline"
            size="sm"
            disabled={selected.size === 0 || scanMutation.isPending}
            onClick={() => scanMutation.mutate(Array.from(selected))}
          >
            <ScanLine className="h-4 w-4 mr-1" />
            Escanear
          </Button>
          <Button
            variant="destructive"
            size="sm"
            disabled={selected.size === 0 || cleanMutation.isPending}
            onClick={() => cleanMutation.mutate()}
          >
            <Trash2 className="h-4 w-4 mr-1" />
            {cleanMutation.isPending
              ? "Limpiando..."
              : `Limpiar${totalBytes > 0 ? ` (${formatBytes(totalBytes)})` : ""}`}
          </Button>
        </div>
      </div>

      {locations.length === 0 ? (
        <EmptyState icon={Trash2} title="Sin ubicaciones de caché" />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="p-3 w-10">
                  <Checkbox
                    checked={selected.size === locations.length && locations.length > 0}
                    onCheckedChange={(v) => (v ? selectAll() : clearAll())}
                  />
                </th>
                <th className="text-left p-3 font-medium text-muted-foreground">Ubicación</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Categoría</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Riesgo</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Estimado</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Detectado</th>
              </tr>
            </thead>
            <tbody>
              {locations.map((loc: CacheLocation) => {
                const report = scanReports.get(loc.id);
                return (
                  <tr
                    key={loc.id}
                    className="border-b border-border/50 hover:bg-accent/30 cursor-pointer"
                    onClick={() => toggle(loc.id)}
                  >
                    <td className="p-3" onClick={(e) => e.stopPropagation()}>
                      <Checkbox
                        checked={selected.has(loc.id)}
                        onCheckedChange={() => toggle(loc.id)}
                      />
                    </td>
                    <td className="p-3">
                      <div className="font-medium">{loc.displayName}</div>
                      <div className="text-xs text-muted-foreground font-mono">
                        {loc.path}
                      </div>
                    </td>
                    <td className="p-3">
                      {loc.category ? (
                        <Badge variant="secondary">{loc.category}</Badge>
                      ) : (
                        <span className="text-xs text-muted-foreground">—</span>
                      )}
                    </td>
                    <td className="p-3">
                      <Badge variant={riskVariant(loc.risk)}>{loc.risk ?? "low"}</Badge>
                    </td>
                    <td className="p-3 text-right text-muted-foreground text-xs">
                      {loc.averageSize ?? "—"}
                    </td>
                    <td className="p-3 text-right font-mono text-xs">
                      {report ? formatBytes(report.bytes) : "—"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {cleanMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Limpieza completada:{" "}
          {formatBytes(cleanMutation.data?.totalBytesFreed ?? 0)} liberados,{" "}
          {cleanMutation.data?.totalFilesDeleted ?? 0} archivos eliminados.
        </div>
      )}
    </div>
  );
}
