import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, RefreshCw, AlertTriangle } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { listServices, setServiceState, applyServicePreset, type Service } from "../../api";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";
import { formatError } from "../../lib/errors";

function stateVariant(state: string): "success" | "secondary" | "warning" {
  if (state === "Running") return "success";
  if (state === "Stopped") return "secondary";
  return "warning";
}

function startTypeVariant(startType: string): "default" | "secondary" | "warning" | "destructive" {
  const s = startType.toLowerCase();
  if (s === "automatic") return "default";
  if (s === "manual") return "secondary";
  if (s === "disabled") return "destructive";
  return "warning";
}

export function ServicesPage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const qc = useQueryClient();
  const [filter, setFilter] = useState("");

  const { data: services = [], isLoading, refetch, isFetching } = useQuery({
    queryKey: ["services"],
    queryFn: listServices,
  });

  const setStateMutation = useMutation({
    mutationFn: ({ name, type }: { name: string; type: string }) =>
      setServiceState(name, type, false),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["services"] }),
  });

  const presetMutation = useMutation({
    mutationFn: (preset: string) => applyServicePreset(preset, false),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["services"] }),
  });

  const filtered = services.filter((s) => {
    const f = filter.toLowerCase();
    return (
      s.displayName.toLowerCase().includes(f) ||
      s.name.toLowerCase().includes(f) ||
      (s.description ?? "").toLowerCase().includes(f)
    );
  });

  if (isLoading) {
    return <div className="p-6 text-muted-foreground">Cargando servicios...</div>;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Servicios de Windows</h2>
          <p className="text-muted-foreground text-sm">
            {services.length} servicios encontrados
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void refetch()} disabled={isFetching}>
            <RefreshCw className={`h-4 w-4 mr-1 ${isFetching ? "animate-spin" : ""}`} />
            Actualizar
          </Button>
        </div>
      </div>

      {!isElevated && (
        <div className="flex items-center gap-2 p-3 bg-yellow-900/30 border border-yellow-700/50 rounded-lg text-yellow-400 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          Requiere permisos de administrador para modificar servicios.
        </div>
      )}

      <div className="flex gap-2 flex-wrap">
        <Button
          variant="outline"
          size="sm"
          disabled={!isElevated || presetMutation.isPending}
          onClick={() => presetMutation.mutate("minimal")}
        >
          Preset: Mínimo
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={!isElevated || presetMutation.isPending}
          onClick={() => presetMutation.mutate("recommended")}
        >
          Preset: Recomendado
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={!isElevated || presetMutation.isPending}
          onClick={() => presetMutation.mutate("aggressive")}
        >
          Preset: Agresivo
        </Button>
      </div>

      <input
        type="text"
        placeholder="Filtrar por nombre, descripción..."
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        className="w-full max-w-sm px-3 py-2 bg-surface-2 border border-border rounded-md text-sm text-ink-primary placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-signal-cyan/50"
      />

      {filtered.length === 0 ? (
        <EmptyState
          icon={Settings2}
          title="Sin servicios"
          description="No se encontraron servicios con el filtro actual"
        />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="text-left p-3 font-medium text-muted-foreground">Servicio</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Estado</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Tipo de inicio</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Acciones</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((svc: Service) => (
                <tr key={svc.name} className="border-b border-border/50 hover:bg-accent/30">
                  <td className="p-3">
                    <div className="font-medium">{svc.displayName}</div>
                    <div className="text-xs text-muted-foreground font-mono">{svc.name}</div>
                    {svc.description && (
                      <div className="text-xs text-muted-foreground mt-1">{svc.description}</div>
                    )}
                  </td>
                  <td className="p-3">
                    <Badge variant={stateVariant(svc.state)}>{svc.state}</Badge>
                  </td>
                  <td className="p-3">
                    <Badge variant={startTypeVariant(svc.startType)}>{svc.startType}</Badge>
                  </td>
                  <td className="p-3 text-right">
                    <div className="flex gap-1 justify-end">
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={!isElevated || setStateMutation.isPending}
                        onClick={() =>
                          setStateMutation.mutate({ name: svc.name, type: "Disabled" })
                        }
                      >
                        Deshabilitar
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={!isElevated || setStateMutation.isPending}
                        onClick={() =>
                          setStateMutation.mutate({ name: svc.name, type: "Manual" })
                        }
                      >
                        Manual
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={!isElevated || setStateMutation.isPending}
                        onClick={() =>
                          setStateMutation.mutate({ name: svc.name, type: "Automatic" })
                        }
                      >
                        Automático
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {presetMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Preset aplicado correctamente.
        </div>
      )}

      {presetMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error al aplicar preset: {formatError(presetMutation.error)}
        </div>
      )}
    </div>
  );
}
