import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Power, Loader2 } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import {
  listStartup,
  disableStartup,
  enableStartup,
  type StartupEntry,
  type StartupOrigin,
  type StartupImpact,
  type StartupCategory,
} from "../../api";
import { EmptyState } from "../../components/empty-state";
import { formatError } from "../../lib/errors";
import { toast } from "../../lib/toast";

function impactBadge(impact: StartupImpact): { label: string; variant: "destructive" | "warning" | "secondary" | "default" } {
  switch (impact) {
    case "high":
      return { label: "Alto", variant: "destructive" };
    case "medium":
      return { label: "Medio", variant: "warning" };
    case "low":
      return { label: "Bajo", variant: "secondary" };
    default:
      return { label: "Desconocido", variant: "default" };
  }
}

function originLabel(origin: StartupOrigin): string {
  switch (origin.kind) {
    case "registry":
      return `Registro (${origin.hive})`;
    case "startupFolder":
      return "Carpeta Inicio";
    case "scheduledTask":
      return "Tarea programada";
    case "service":
      return "Servicio";
    case "uwpAutoStart":
      return "UWP";
  }
}

function categoryLabel(cat: StartupCategory): string {
  const labels: Record<string, string> = {
    "updater": "Actualizador",
    "launcher": "Lanzador",
    "widget": "Widget",
    "cloud-sync": "Cloud Sync",
    "communication": "Comunicación",
    "media": "Media",
    "security": "Seguridad",
    "driver": "Driver",
    "user-app": "App usuario",
    "system": "Sistema",
    "unknown": "Desconocido",
  };
  return labels[cat] ?? cat;
}

function ImpactGroup({
  title,
  items,
  level,
  defaultOpen,
}: {
  title: string;
  items: StartupEntry[];
  level: string;
  defaultOpen?: boolean;
}) {
  if (items.length === 0) return null;

  const colorMap: Record<string, string> = {
    high: "border-red-700/50 bg-red-900/10",
    medium: "border-yellow-700/50 bg-yellow-900/10",
    low: "border-green-700/50 bg-green-900/10",
    disabled: "border-gray-700/50 bg-gray-900/10",
  };

  return (
    <details open={defaultOpen} className={`border ${colorMap[level] || "border-border"} rounded-lg`}>
      <summary className="cursor-pointer p-3 font-medium">
        {title} ({items.length})
      </summary>
      <div className="divide-y divide-border/50">
        {items.map((e) => (
          <StartupRow key={e.id} entry={e} />
        ))}
      </div>
    </details>
  );
}

function StartupRow({ entry }: { entry: StartupEntry }) {
  const qc = useQueryClient();

  const toggleMutation = useMutation({
    mutationFn: () =>
      entry.enabled
        ? disableStartup(entry.id)
        : enableStartup(entry.id),
    onSuccess: () => {
      toast.success(
        `${entry.displayName} ${entry.enabled ? "deshabilitada" : "habilitada"}`
      );
      void qc.invalidateQueries({ queryKey: ["startup"] });
    },
    onError: (err) =>
      toast.error("Error cambiando estado", { description: formatError(err) }),
  });

  return (
    <div className="px-4 py-3 flex items-center gap-3 text-sm">
      <div className="flex-1 min-w-0">
        <div className="font-medium">{entry.displayName}</div>
        <div className="text-xs text-muted-foreground truncate font-mono">
          {entry.command}
        </div>
        <div className="flex gap-2 mt-1 flex-wrap">
          <Badge variant="secondary" className="text-xs">
            {originLabel(entry.origin)}
          </Badge>
          <Badge variant="outline" className="text-xs">
            {categoryLabel(entry.category)}
          </Badge>
          <Badge variant={impactBadge(entry.impact).variant} className="text-xs">
            {impactBadge(entry.impact).label}
          </Badge>
        </div>
      </div>
      <Button
        size="sm"
        variant={entry.enabled ? "outline" : "default"}
        onClick={() => toggleMutation.mutate()}
        disabled={toggleMutation.isPending}
        className="min-w-[100px]"
      >
        {toggleMutation.isPending ? (
          <Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />
        ) : (
          <Power className="h-3.5 w-3.5 mr-1" />
        )}
        {entry.enabled ? "Deshabilitar" : "Habilitar"}
      </Button>
    </div>
  );
}

export function StartupPage() {
  const {
    data: entries = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ["startup"],
    queryFn: listStartup,
  });

  const grouped = {
    high: entries.filter((e) => e.impact === "high" && e.enabled),
    medium: entries.filter((e) => e.impact === "medium" && e.enabled),
    low: entries.filter((e) => e.impact === "low" && e.enabled),
    unknown: entries.filter((e) => e.impact === "unknown" && e.enabled),
    disabled: entries.filter((e) => !e.enabled),
  };

  if (isLoading)
    return <div className="p-6 text-muted-foreground">Cargando entradas de arranque...</div>;

  if (error) {
    return (
      <div className="p-6">
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-300 text-sm whitespace-pre-wrap">
          Error al cargar arranque: {formatError(error)}
        </div>
      </div>
    );
  }

  const totalEnabled = entries.filter((e) => e.enabled).length;

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div>
        <h2 className="text-2xl font-bold">Arranque automático</h2>
        <p className="text-muted-foreground text-sm">
          {entries.length} entradas · {totalEnabled} habilitadas ·{" "}
          {grouped.disabled.length} deshabilitadas
        </p>
      </div>

      {entries.length === 0 ? (
        <EmptyState icon={Power} title="Sin entradas de arranque" />
      ) : (
        <div className="flex-1 overflow-auto flex flex-col gap-3">
          <ImpactGroup
            title="⚠ Alto impacto"
            items={grouped.high}
            level="high"
            defaultOpen
          />
          <ImpactGroup
            title="Medio impacto"
            items={grouped.medium}
            level="medium"
            defaultOpen
          />
          <ImpactGroup
            title="Bajo impacto"
            items={grouped.low}
            level="low"
          />
          {grouped.unknown.length > 0 && (
            <ImpactGroup
              title="Impacto desconocido"
              items={grouped.unknown}
              level="disabled"
            />
          )}
          {grouped.disabled.length > 0 && (
            <ImpactGroup
              title="Deshabilitadas"
              items={grouped.disabled}
              level="disabled"
            />
          )}
        </div>
      )}
    </div>
  );
}
