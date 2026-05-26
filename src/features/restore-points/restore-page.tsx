import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { RotateCcw, Plus, AlertTriangle, CheckCircle, RefreshCw } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import {
  listRestorePoints,
  createRestorePoint,
  ensureRestoreEnabled,
  restoreToPoint,
  type RestorePoint,
} from "../../api";
import { formatDate } from "../../lib/utils";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";
import { formatError } from "../../lib/errors";

export function RestorePage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const qc = useQueryClient();
  const [restoreEnabled, setRestoreEnabled] = useState<boolean | null>(null);
  const [restoringTo, setRestoringTo] = useState<number | null>(null);
  const [confirmRestore, setConfirmRestore] = useState<number | null>(null);

  const {
    data: points = [],
    isLoading,
    isError,
    error,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ["restore-points"],
    queryFn: listRestorePoints,
  });

  const ensureMutation = useMutation({
    mutationFn: ensureRestoreEnabled,
    onSuccess: (result) => {
      setRestoreEnabled(result);
    },
  });

  const createMutation = useMutation({
    mutationFn: () =>
      createRestorePoint({ description: "ClearTool — punto manual" }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["restore-points"] });
    },
  });

  const restoreMutation = useMutation({
    mutationFn: (seq: number) => restoreToPoint(seq),
    onMutate: (seq) => setRestoringTo(seq),
    onSettled: () => setRestoringTo(null),
  });

  const handleCheckEnabled = () => {
    ensureMutation.mutate();
  };

  const handleRestore = (seq: number) => {
    setConfirmRestore(seq);
  };

  const confirmRestoreAction = () => {
    if (confirmRestore !== null) {
      restoreMutation.mutate(confirmRestore);
      setConfirmRestore(null);
    }
  };

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
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => void refetch()}
            disabled={isFetching}
          >
            <RefreshCw className={`h-4 w-4 mr-1 ${isFetching ? "animate-spin" : ""}`} />
            Actualizar
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleCheckEnabled}
            disabled={!isElevated || ensureMutation.isPending}
          >
            {ensureMutation.isPending ? "Verificando..." : "Verificar estado"}
          </Button>
          <Button
            onClick={() => createMutation.mutate()}
            disabled={!isElevated || createMutation.isPending}
            size="sm"
          >
            <Plus className="h-4 w-4 mr-1" />
            {createMutation.isPending ? "Creando..." : "Crear punto"}
          </Button>
        </div>
      </div>

      {restoreEnabled !== null && (
        <div className={`flex items-center gap-2 p-3 rounded-lg text-sm ${
          restoreEnabled
            ? "bg-green-900/30 border border-green-700/50 text-green-400"
            : "bg-red-900/30 border border-red-700/50 text-red-400"
        }`}>
          <CheckCircle className="h-4 w-4 flex-shrink-0" />
          Restauración del sistema: {restoreEnabled ? "Habilitada" : "Deshabilitada"}
        </div>
      )}

      {isError && (
        <div className="flex items-center gap-2 p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          Error al cargar puntos de restauración: {formatError(error)}. Verifica que la app tiene permisos de administrador.
        </div>
      )}

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
                <th className="text-right p-3 font-medium text-muted-foreground">Acción</th>
              </tr>
            </thead>
            <tbody>
              {points.map((point: RestorePoint) => (
                <tr
                  key={point.sequenceNumber}
                  className="border-b border-border/50 hover:bg-accent/30"
                >
                  <td className="p-3 font-mono text-muted-foreground text-xs">
                    #{point.sequenceNumber}
                  </td>
                  <td className="p-3 font-medium">{point.description}</td>
                  <td className="p-3 text-muted-foreground text-xs">
                    {formatDate(point.creationTime)}
                  </td>
                  <td className="p-3">
                    <Badge variant="secondary">{point.restorePointType}</Badge>
                  </td>
                  <td className="p-3 text-right">
                    {confirmRestore === point.sequenceNumber ? (
                      <div className="flex gap-1 justify-end">
                        <Button
                          variant="destructive"
                          size="sm"
                          disabled={!isElevated || restoreMutation.isPending}
                          onClick={confirmRestoreAction}
                        >
                          {restoreMutation.isPending && restoringTo === point.sequenceNumber
                            ? "Restaurando..."
                            : "Confirmar"}
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => setConfirmRestore(null)}
                        >
                          Cancelar
                        </Button>
                      </div>
                    ) : (
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={!isElevated}
                        onClick={() => handleRestore(point.sequenceNumber)}
                        title={!isElevated ? "Requiere administrador" : undefined}
                      >
                        <RotateCcw className="h-3 w-3 mr-1" />
                        Restaurar
                      </Button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {createMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Punto de restauración #{createMutation.data?.sequenceNumber} creado correctamente.
        </div>
      )}

      {restoreMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Restauración iniciada. El sistema se reiniciará para completar el proceso.
        </div>
      )}

      {restoreMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error al restaurar: {formatError(restoreMutation.error)}
        </div>
      )}
    </div>
  );
}
