import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Database } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { listRegistryTweaks, applyRegistryTweak } from "../../lib/tauri";
import type { RegistryTweak } from "../../bindings";
import { EmptyState } from "../../components/empty-state";
import { useAppStore } from "../../lib/store";

function riskVariant(risk: string): "success" | "warning" | "destructive" {
  const r = risk.toLowerCase();
  if (r === "high") return "destructive";
  if (r === "medium") return "warning";
  return "success";
}

export function RegistryPage() {
  const isElevated = useAppStore((s) => s.isElevated);
  const qc = useQueryClient();

  const { data: tweaks = [], isLoading } = useQuery({
    queryKey: ["registry-tweaks"],
    queryFn: listRegistryTweaks,
  });

  const applyMutation = useMutation({
    mutationFn: ({ id, enable }: { id: string; enable: boolean }) =>
      applyRegistryTweak({ id, enable, dry_run: false }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["registry-tweaks"] }),
  });

  if (isLoading) {
    return <div className="p-6 text-muted-foreground">Cargando tweaks...</div>;
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div>
        <h2 className="text-2xl font-bold">Tweaks de Registro</h2>
        <p className="text-muted-foreground text-sm">
          {tweaks.length} tweaks disponibles
        </p>
      </div>

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
                <th className="text-left p-3 font-medium text-muted-foreground">Tweak</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Categoría</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Riesgo</th>
                <th className="text-right p-3 font-medium text-muted-foreground">Acción</th>
              </tr>
            </thead>
            <tbody>
              {tweaks.map((tweak: RegistryTweak) => (
                <tr key={tweak.id} className="border-b border-border/50 hover:bg-accent/30">
                  <td className="p-3">
                    <div className="font-medium">{tweak.display_name}</div>
                    <div className="text-xs text-muted-foreground">{tweak.description}</div>
                    <div className="text-xs text-muted-foreground font-mono mt-1">
                      {tweak.hive}\{tweak.path}
                    </div>
                    {tweak.requires_reboot && (
                      <span className="text-xs text-yellow-400">Requiere reinicio</span>
                    )}
                  </td>
                  <td className="p-3">
                    <Badge variant="secondary">{tweak.category}</Badge>
                  </td>
                  <td className="p-3">
                    <Badge variant={riskVariant(tweak.risk)}>{tweak.risk}</Badge>
                  </td>
                  <td className="p-3 text-right">
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={!isElevated || applyMutation.isPending}
                      onClick={() => applyMutation.mutate({ id: tweak.id, enable: true })}
                      title={!isElevated ? "Requiere administrador" : undefined}
                    >
                      Aplicar
                    </Button>
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
