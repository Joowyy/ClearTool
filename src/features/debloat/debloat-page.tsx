import { useState, useEffect } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Package, Search, Scan, AlertTriangle } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Checkbox } from "../../components/ui/checkbox";
import { Badge } from "../../components/ui/badge";
import { Input } from "../../components/ui/input";
import {
  listBloatwareCatalog,
  detectInstalledBloatware,
  removeBloatware,
  type BloatwareEntry,
  type DetectedPackage,
} from "../../api";
import { EmptyState } from "../../components/empty-state";
import { formatError } from "../../lib/errors";
import { useAppStore } from "../../lib/store";

function riskVariant(risk: string): "success" | "warning" | "destructive" {
  const r = risk.toLowerCase();
  if (r === "high") return "destructive";
  if (r === "medium") return "warning";
  return "success";
}

export function DebloatPage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState("");
  const [showOnlyInstalled, setShowOnlyInstalled] = useState(false);
  const [installedPackages, setInstalledPackages] = useState<DetectedPackage[]>([]);
  const [dryRun, setDryRun] = useState(false);

  const { data: catalog = [], isLoading } = useQuery({
    queryKey: ["bloatware-catalog"],
    queryFn: listBloatwareCatalog,
  });

  const detectMutation = useMutation({
    mutationFn: detectInstalledBloatware,
    onSuccess: (data) => {
      setInstalledPackages(data);
      const installedIds = new Set(data.map((p) => p.id));
      setSelected(installedIds);
    },
  });

  const removeMutation = useMutation({
    mutationFn: () =>
      removeBloatware({
        entryIds: Array.from(selected),
        dryRun,
        createRestorePoint: true,
        applyPolicies: true,
        disableServices: true,
      }),
  });

  useEffect(() => {
    if (catalog.length > 0 && selected.size === 0 && installedPackages.length === 0) {
      setSelected(new Set(catalog.map((e) => e.id)));
    }
  }, [catalog]);

  const installedIds = new Set(installedPackages.map((p) => p.id));

  const filtered = catalog.filter((e) => {
    const f = filter.toLowerCase();
    const matchesFilter =
      e.displayName.toLowerCase().includes(f) ||
      (e.category ?? "").toLowerCase().includes(f);
    const matchesInstalled = !showOnlyInstalled || installedIds.has(e.id);
    return matchesFilter && matchesInstalled;
  });

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  const applyPreset = (preset: "minimal" | "recommended" | "total") => {
    if (preset === "total") {
      setSelected(new Set(catalog.map((e) => e.id)));
    } else if (preset === "recommended") {
      setSelected(
        new Set(
          catalog
            .filter((e) =>
              ["consumer-app", "consumerapp", "ai", "telemetry", "ms-consumer", "msconsumer"].includes(
                (e.category ?? "").toLowerCase()
              )
            )
            .map((e) => e.id)
        )
      );
    } else {
      setSelected(
        new Set(
          catalog
            .filter(
              (e) =>
                ["consumer-app", "consumerapp", "ai"].includes(
                  (e.category ?? "").toLowerCase()
                ) && (e.risk ?? "").toLowerCase() === "low"
            )
            .map((e) => e.id)
        )
      );
    }
  };

  if (isLoading) return <div className="p-6 text-muted-foreground">Cargando catálogo...</div>;

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between flex-wrap gap-2">
        <div>
          <h2 className="text-2xl font-bold">Debloat</h2>
          <p className="text-muted-foreground text-sm">
            {selected.size} de {catalog.length} entradas seleccionadas
            {installedPackages.length > 0 && (
              <span className="ml-2">· {installedPackages.length} instaladas</span>
            )}
          </p>
        </div>
        <div className="flex gap-2 flex-wrap">
          <Button
            variant="outline"
            size="sm"
            disabled={detectMutation.isPending}
            onClick={() => detectMutation.mutate()}
          >
            <Scan className="h-4 w-4 mr-1" />
            {detectMutation.isPending ? "Escaneando..." : "Detectar instalados"}
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("minimal")}>
            Mínimo
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("recommended")}>
            Recomendado
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("total")}>
            Total
          </Button>
        </div>
      </div>

      {!isElevated && (
        <div className="flex items-center gap-2 p-3 bg-yellow-900/30 border border-yellow-700/50 rounded-lg text-yellow-400 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          Requiere permisos de administrador para eliminar paquetes.
        </div>
      )}

      <div className="flex items-center gap-4 flex-wrap">
        <div className="flex items-center gap-2">
          <Search className="h-4 w-4 text-muted-foreground flex-shrink-0" />
          <Input
            placeholder="Filtrar por nombre o categoría..."
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="max-w-sm"
          />
        </div>
        <div className="flex items-center gap-2">
          <Checkbox
            checked={showOnlyInstalled}
            onCheckedChange={(v) => setShowOnlyInstalled(!!v)}
          />
          <span className="text-sm text-muted-foreground">Solo instalados</span>
        </div>
        <div className="flex items-center gap-2">
          <Checkbox
            checked={dryRun}
            onCheckedChange={(v) => setDryRun(!!v)}
          />
          <span className="text-sm text-muted-foreground">Dry-run (simulación)</span>
        </div>
      </div>

      {filtered.length === 0 ? (
        <EmptyState icon={Package} title="Sin resultados" description="Prueba con otro filtro" />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="p-3 w-10">
                  <Checkbox
                    checked={filtered.every((e) => selected.has(e.id)) && filtered.length > 0}
                    onCheckedChange={(v) =>
                      v
                        ? setSelected((prev) => {
                            const next = new Set(prev);
                            filtered.forEach((e) => next.add(e.id));
                            return next;
                          })
                        : setSelected((prev) => {
                            const next = new Set(prev);
                            filtered.forEach((e) => next.delete(e.id));
                            return next;
                          })
                    }
                  />
                </th>
                <th className="text-left p-3 font-medium text-muted-foreground">Aplicación</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Categoría</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Estrategia</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Riesgo</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Estado</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Reversa</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((entry: BloatwareEntry) => {
                const isInstalled = installedIds.has(entry.id);
                const detected = installedPackages.find((p) => p.id === entry.id);

                return (
                  <tr
                    key={entry.id}
                    className={`border-b border-border/50 hover:bg-accent/30 cursor-pointer ${
                      selected.has(entry.id) ? "bg-cyan-900/10" : ""
                    }`}
                    onClick={() => toggle(entry.id)}
                  >
                    <td className="p-3" onClick={(e) => e.stopPropagation()}>
                      <Checkbox
                        checked={selected.has(entry.id)}
                        onCheckedChange={() => toggle(entry.id)}
                      />
                    </td>
                    <td className="p-3">
                      <div className="font-medium">{entry.displayName}</div>
                      {entry.consequences?.[0] && (
                        <div className="text-xs text-muted-foreground">{entry.consequences[0]}</div>
                      )}
                      {detected?.installLocation && (
                        <div className="text-xs text-muted-foreground font-mono mt-1">
                          {detected.installLocation}
                        </div>
                      )}
                    </td>
                    <td className="p-3">
                      <Badge variant="secondary">{entry.category ?? "—"}</Badge>
                    </td>
                    <td className="p-3 text-xs text-muted-foreground">{entry.removalStrategy ?? "—"}</td>
                    <td className="p-3">
                      <Badge variant={riskVariant(entry.risk ?? "low")}>{entry.risk ?? "low"}</Badge>
                    </td>
                    <td className="p-3">
                      {isInstalled ? (
                        <Badge variant="destructive">Instalado</Badge>
                      ) : (
                        <span className="text-xs text-muted-foreground">No detectado</span>
                      )}
                    </td>
                    <td className="p-3 text-xs text-muted-foreground">{entry.reversalMethod ?? "—"}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {removeMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          {dryRun ? (
            <>Dry-run completado: {removeMutation.data?.removed} paquetes serían removidos, {removeMutation.data?.failed} fallarían.</>
          ) : (
            <>Eliminación completada: {removeMutation.data?.removed} paquetes removidos, {removeMutation.data?.failed} fallidos.</>
          )}
        </div>
      )}

      {removeMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error: {formatError(removeMutation.error)}
        </div>
      )}

      {detectMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error al detectar paquetes: {formatError(detectMutation.error)}
        </div>
      )}
    </div>
  );
}
