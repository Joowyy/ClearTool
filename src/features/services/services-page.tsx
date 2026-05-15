import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, RefreshCw } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { listServices, setServiceState } from "../../lib/tauri";
import type { Service } from "../../bindings";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";

function stateVariant(state: string): "success" | "secondary" | "warning" {
  if (state === "Running") return "success";
  if (state === "Stopped") return "secondary";
  return "warning";
}

export function ServicesPage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const qc = useQueryClient();

  const { data: services = [], isLoading, refetch, isFetching } = useQuery({
    queryKey: ["services"],
    queryFn: listServices,
  });

  const setStateMutation = useMutation({
    mutationFn: ({ name, type }: { name: string; type: string }) =>
      setServiceState(name, type, false),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["services"] }),
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
        <Button variant="outline" size="sm" onClick={() => void refetch()} disabled={isFetching}>
          <RefreshCw className={`h-4 w-4 mr-1 ${isFetching ? "animate-spin" : ""}`} />
          Actualizar
        </Button>
      </div>

      {services.length === 0 ? (
        <EmptyState
          icon={Settings2}
          title="Sin servicios"
          description="No se pudieron cargar los servicios del sistema"
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
              {services.map((svc: Service) => (
                <tr key={svc.name} className="border-b border-border/50 hover:bg-accent/30">
                  <td className="p-3">
                    <div className="font-medium">{svc.display_name}</div>
                    <div className="text-xs text-muted-foreground font-mono">{svc.name}</div>
                  </td>
                  <td className="p-3">
                    <Badge variant={stateVariant(svc.state)}>{svc.state}</Badge>
                  </td>
                  <td className="p-3 text-muted-foreground text-xs">{svc.start_type}</td>
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
                    </div>
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
