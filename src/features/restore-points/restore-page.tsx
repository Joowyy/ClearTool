import { useQuery, useMutation } from "@tanstack/react-query";
import { RotateCcw, Plus, AlertTriangle } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { listRestorePoints, createRestorePoint } from "../../lib/tauri";
import { formatDate } from "../../lib/utils";
import type { RestorePoint } from "../../bindings";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";

export function RestorePage() {
  const isElevated = useAppStore((s) => s.isElevated);

  const {
    data: points = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["restore-points"],
    queryFn: listRestorePoints,
  });

  const createMutation = useMutation({
    mutationFn: () =>
      createRestorePoint({ description: "ClearTool — punto manual" }),
    onSuccess: () => void refetch(),
  });

  if (isLoading) {
    return <div className="p-6 text-muted-foreground">Cargando puntos de restauración...</div>;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Puntos de Restauración</h2>
          <p className="text-muted-foreground text-sm">
            {points.length} puntos encontrados
          </p>
        </div>
        <Button
          onClick={() => createMutation.mutate()}
          disabled={!isElevated || createMutation.isPending}
          size="sm"
        >
          <Plus className="h-4 w-4 mr-1" />
          {createMutation.isPending ? "Creando..." : "Crear punto"}
        </Button>
      </div>

      {!isElevated && (
        <div className="flex items-center gap-2 p-3 bg-yellow-900/30 border border-yellow-700/50 rounded-lg text-yellow-400 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          Requiere permisos de administrador para crear o restaurar puntos del sistema.
        </div>
      )}

      {points.length === 0 ? (
        <EmptyState
          icon={RotateCcw}
          title="Sin puntos de restauración"
          description="No hay puntos de restauración del sistema disponibles"
        />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                <th className="text-left p-3 font-medium text-muted-foreground">#</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Descripción</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Fecha</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Tipo</th>
              </tr>
            </thead>
            <tbody>
              {points.map((point: RestorePoint) => (
                <tr
                  key={point.sequence_number}
                  className="border-b border-border/50 hover:bg-accent/30"
                >
                  <td className="p-3 font-mono text-muted-foreground text-xs">
                    #{point.sequence_number}
                  </td>
                  <td className="p-3 font-medium">{point.description}</td>
                  <td className="p-3 text-muted-foreground text-xs">
                    {formatDate(point.created_at)}
                  </td>
                  <td className="p-3">
                    <Badge variant="secondary">{point.restore_point_type}</Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {createMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Punto de restauración #{createMutation.data?.sequence_number} creado correctamente.
        </div>
      )}
    </div>
  );
}
