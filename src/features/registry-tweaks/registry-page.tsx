import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Database, Check, X, RotateCcw, Layers } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { Checkbox } from "../../components/ui/checkbox";
import {
  listRegistryTweaks,
  readRegistryTweakState,
  applyRegistryTweak,
  applyRegistryTweakBatch,
  revertRegistryTweak,
  type RegistryTweak,
  type TweakState,
} from "../../api";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";

function riskVariant(risk?: string | null): "success" | "warning" | "destructive" {
  const r = (risk ?? "").toLowerCase();
  if (r === "high") return "destructive";
  if (r === "medium") return "warning";
  return "success";
}

export function RegistryPage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const qc = useQueryClient();
  const [tweakStates, setTweakStates] = useState<Record<string, TweakState>>({});
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [showBatch, setShowBatch] = useState(false);

  const { data: tweaks = [], isLoading } = useQuery({
    queryKey: ["registry-tweaks"],
    queryFn: listRegistryTweaks,
  });

  const readStateMutation = useMutation({
    mutationFn: (id: string) => readRegistryTweakState(id),
    onSuccess: (state) => {
      setTweakStates((prev) => ({ ...prev, [state.id]: state }));
    },
  });

  const applyMutation = useMutation({
    mutationFn: ({ id, enable }: { id: string; enable: boolean }) =>
      applyRegistryTweak({ id, enable, dryRun: false }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["registry-tweaks"] });
    },
  });

  const batchMutation = useMutation({
    mutationFn: (inputs: { id: string; enable: boolean; dryRun: boolean }[]) =>
      applyRegistryTweakBatch(inputs),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["registry-tweaks"] });
      setSelected(new Set());
    },
  });

  const revertMutation = useMutation({
    mutationFn: (id: string) => revertRegistryTweak(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["registry-tweaks"] });
    },
  });

  useEffect(() => {
    if (tweaks.length > 0) {
      tweaks.forEach((t) => {
        if (!tweakStates[t.id]) {
          readStateMutation.mutate(t.id);
        }
      });
    }
  }, [tweaks]);

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  const applyBatch = (enable: boolean) => {
    const inputs = Array.from(selected).map((id) => ({ id, enable, dryRun: false }));
    batchMutation.mutate(inputs);
  };

  if (isLoading) {
    return <div className="p-6 text-muted-foreground">Cargando tweaks...</div>;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Tweaks de Registro</h2>
          <p className="text-muted-foreground text-sm">
            {tweaks.length} tweaks disponibles
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant={showBatch ? "default" : "outline"}
            size="sm"
            onClick={() => setShowBatch(!showBatch)}
          >
            <Layers className="h-4 w-4 mr-1" />
            {showBatch ? "Salir de lote" : "Modo lote"}
          </Button>
        </div>
      </div>

      {showBatch && selected.size > 0 && (
        <div className="flex items-center gap-2 p-3 bg-cyan-900/30 border border-cyan-700/50 rounded-lg">
          <span className="text-sm text-cyan-400">
            {selected.size} tweaks seleccionados
          </span>
          <Button
            variant="default"
            size="sm"
            disabled={!isElevated || batchMutation.isPending}
            onClick={() => applyBatch(true)}
          >
            <Check className="h-3 w-3 mr-1" />
            Habilitar todos
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={!isElevated || batchMutation.isPending}
            onClick={() => applyBatch(false)}
          >
            <X className="h-3 w-3 mr-1" />
            Deshabilitar todos
          </Button>
        </div>
      )}

      {tweaks.length === 0 ? (
        <EmptyState
          icon={Database}
          title="Sin tweaks"
          description="No hay tweaks de registro configurados en el catálogo"
        />
      ) : (
        <div className="flex-1 overflow-auto border border-border rounded-lg">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-card border-b border-border z-10">
              <tr>
                {showBatch && (
                  <th className="p-3 w-10">
                    <Checkbox
                      checked={selected.size === tweaks.length && tweaks.length > 0}
                      onCheckedChange={(v) =>
                        v
                          ? setSelected(new Set(tweaks.map((t) => t.id)))
                          : setSelected(new Set())
                      }
                    />
                  </th>
                )}
                <th className="text-left p-3 font-medium text-muted-foreground">Tweak</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Categoría</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Estado actual</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Riesgo</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Acciones</th>
              </tr>
            </thead>
            <tbody>
              {tweaks.map((tweak: RegistryTweak) => {
                const state = tweakStates[tweak.id];
                const isSelected = selected.has(tweak.id);

                return (
                  <tr
                    key={tweak.id}
                    className={`border-b border-border/50 hover:bg-accent/30 ${
                      showBatch && isSelected ? "bg-cyan-900/20" : ""
                    }`}
                    onClick={() => showBatch && toggle(tweak.id)}
                  >
                    {showBatch && (
                      <td className="p-3" onClick={(e) => e.stopPropagation()}>
                        <Checkbox checked={isSelected} onCheckedChange={() => toggle(tweak.id)} />
                      </td>
                    )}
                    <td className="p-3">
                      <div className="font-medium">{tweak.displayName}</div>
                      <div className="text-xs text-muted-foreground">{tweak.description}</div>
                      <div className="text-xs text-muted-foreground font-mono mt-1">
                        {tweak.hive}\{tweak.path}
                      </div>
                      {tweak.requiresReboot && (
                        <span className="text-xs text-yellow-400">Requiere reinicio</span>
                      )}
                    </td>
                    <td className="p-3">
                      <Badge variant="secondary">{tweak.category ?? "—"}</Badge>
                    </td>
                    <td className="p-3">
                      {state?.isEnabled === true && (
                        <span className="text-signal-emerald text-xs">Habilitado</span>
                      )}
                      {state?.isEnabled === false && (
                        <span className="text-muted-foreground text-xs">Deshabilitado</span>
                      )}
                      {state?.isEnabled === null && (
                        <span className="text-muted-foreground text-xs">Desconocido</span>
                      )}
                      {!state && readStateMutation.isPending && (
                        <span className="text-muted-foreground text-xs">Leyendo...</span>
                      )}
                    </td>
                    <td className="p-3">
                      <Badge variant={riskVariant(tweak.risk)}>{tweak.risk ?? "low"}</Badge>
                    </td>
                    <td className="p-3 text-right">
                      <div className="flex gap-1 justify-end">
                        <Button
                          variant={state?.isEnabled ? "outline" : "default"}
                          size="sm"
                          disabled={!isElevated || applyMutation.isPending}
                          onClick={(e) => {
                            e.stopPropagation();
                            applyMutation.mutate({ id: tweak.id, enable: true });
                          }}
                          title={!isElevated ? "Requiere administrador" : undefined}
                        >
                          Habilitar
                        </Button>
                        <Button
                          variant={!state?.isEnabled ? "outline" : "default"}
                          size="sm"
                          disabled={!isElevated || applyMutation.isPending}
                          onClick={(e) => {
                            e.stopPropagation();
                            applyMutation.mutate({ id: tweak.id, enable: false });
                          }}
                          title={!isElevated ? "Requiere administrador" : undefined}
                        >
                          Deshabilitar
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          disabled={!isElevated || revertMutation.isPending}
                          onClick={(e) => {
                            e.stopPropagation();
                            revertMutation.mutate(tweak.id);
                          }}
                          title="Revertir a valor original"
                        >
                          <RotateCcw className="h-3 w-3" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {batchMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Lote aplicado correctamente.
        </div>
      )}

      {applyMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Tweak aplicado correctamente.
        </div>
      )}

      {revertMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Tweak revertido al valor original.
        </div>
      )}
    </div>
  );
}
