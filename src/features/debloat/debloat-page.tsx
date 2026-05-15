import { useState, useEffect } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Package, Search } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Checkbox } from "../../components/ui/checkbox";
import { Badge } from "../../components/ui/badge";
import { Input } from "../../components/ui/input";
import { listBloatwareCatalog, removeBloatware } from "../../lib/tauri";
import type { BloatwareEntry } from "../../bindings";
import { EmptyState } from "../../components/empty-state";
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

  const { data: catalog = [], isLoading } = useQuery({
    queryKey: ["bloatware-catalog"],
    queryFn: listBloatwareCatalog,
  });

  // Preset "Total" por defecto al cargar
  useEffect(() => {
    if (catalog.length > 0 && selected.size === 0) {
      setSelected(new Set(catalog.map((e) => e.id)));
    }
  }, [catalog]);

  const removeMutation = useMutation({
    mutationFn: () =>
      removeBloatware({
        entry_ids: Array.from(selected),
        dry_run: false,
        create_restore_point: true,
        apply_policies: true,
        disable_services: true,
      }),
  });

  const filtered = catalog.filter(
    (e) =>
      e.display_name.toLowerCase().includes(filter.toLowerCase()) ||
      e.category.toLowerCase().includes(filter.toLowerCase())
  );

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
                e.category.toLowerCase()
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
                ["consumer-app", "consumerapp", "ai"].includes(e.category.toLowerCase()) &&
                e.risk.toLowerCase() === "low"
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
          </p>
        </div>
        <div className="flex gap-2 flex-wrap">
          <Button variant="outline" size="sm" onClick={() => applyPreset("minimal")}>
            Mínimo
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("recommended")}>
            Recomendado
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("total")}>
            Total
          </Button>
          <Button
            variant="destructive"
            size="sm"
            disabled={selected.size === 0 || !isElevated || removeMutation.isPending}
            onClick={() => removeMutation.mutate()}
            title={!isElevated ? "Requiere permisos de administrador" : undefined}
          >
            <Package className="h-4 w-4 mr-1" />
            {removeMutation.isPending ? "Eliminando..." : "Eliminar seleccionados"}
          </Button>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Search className="h-4 w-4 text-muted-foreground flex-shrink-0" />
        <Input
          placeholder="Filtrar por nombre o categoría..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="max-w-sm"
        />
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
                    checked={selected.size === catalog.length && catalog.length > 0}
                    onCheckedChange={(v) =>
                      v
                        ? setSelected(new Set(catalog.map((e) => e.id)))
                        : setSelected(new Set())
                    }
                  />
                </th>
                <th className="text-left p-3 font-medium text-muted-foreground">Aplicación</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Categoría</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Estrategia</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Riesgo</th>
                <th className="text-left p-3 font-medium text-muted-foreground">Reversa</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((entry: BloatwareEntry) => (
                <tr
                  key={entry.id}
                  className="border-b border-border/50 hover:bg-accent/30 cursor-pointer"
                  onClick={() => toggle(entry.id)}
                >
                  <td className="p-3" onClick={(e) => e.stopPropagation()}>
                    <Checkbox
                      checked={selected.has(entry.id)}
                      onCheckedChange={() => toggle(entry.id)}
                    />
                  </td>
                  <td className="p-3">
                    <div className="font-medium">{entry.display_name}</div>
                    {entry.consequences[0] && (
                      <div className="text-xs text-muted-foreground">{entry.consequences[0]}</div>
                    )}
                  </td>
                  <td className="p-3">
                    <Badge variant="secondary">{entry.category}</Badge>
                  </td>
                  <td className="p-3 text-xs text-muted-foreground">{entry.removal_strategy}</td>
                  <td className="p-3">
                    <Badge variant={riskVariant(entry.risk)}>{entry.risk}</Badge>
                  </td>
                  <td className="p-3 text-xs text-muted-foreground">{entry.reversal_method}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {removeMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Eliminación completada: {removeMutation.data?.total_removed} paquetes removidos,{" "}
          {removeMutation.data?.total_failed} fallidos.
        </div>
      )}

      {removeMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error: {String(removeMutation.error)}
        </div>
      )}
    </div>
  );
}
