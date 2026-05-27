/*
 * debloat-page — Inventario universal de apps instaladas con desinstalación completa.
 *
 * Fuentes: Win32, Appx, Steam, Epic, GOG. Filtros por fuente, publisher,
 * nombre y catálogo de bloatware curado.
 */
import { useState, useMemo, useCallback } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Package, Search, Scan, AlertTriangle, Trash2, ChevronDown, ChevronUp,
  Loader2, Check, X, Filter,
} from "lucide-react";
import { Button } from "../../components/ui/button";
import { Checkbox } from "../../components/ui/checkbox";
import { EmptyState } from "../../components/empty-state";
import { cn, formatBytes } from "../../lib/utils";
import { formatError } from "../../lib/errors";
import { toast } from "../../lib/toast";
import {
  listInstalledApps,
  uninstallAppComplete,
} from "../../api/client";
import type { InstalledApp } from "../../api/types";

// ── Helpers ────────────────────────────────────────────────────────────

function sourceLabel(app: InstalledApp): string {
  switch (app.source.kind) {
    case "appxPackage": return "Appx";
    case "appxProvisioned": return "Appx (provisioned)";
    case "win32Uninstaller": return "Win32";
    case "steam": return "Steam";
    case "epicGames": return "Epic Games";
    case "gog": return "GOG";
    case "xbox": return "Xbox";
    case "winget": return "Winget";
    default: return "Unknown";
  }
}

function sourceColor(kind: string): string {
  const map: Record<string, string> = {
    appxPackage: "bg-blue-500/15 text-blue-400",
    appxProvisioned: "bg-indigo-500/15 text-indigo-400",
    win32Uninstaller: "bg-gray-500/15 text-gray-400",
    steam: "bg-cyan-500/15 text-cyan-400",
    epicGames: "bg-purple-500/15 text-purple-400",
    gog: "bg-yellow-500/15 text-yellow-400",
    xbox: "bg-green-500/15 text-green-400",
    winget: "bg-orange-500/15 text-orange-400",
  };
  return map[kind] ?? "bg-gray-500/15 text-gray-400";
}

function riskColor(risk: string): string {
  if (risk === "high") return "text-signal-red";
  if (risk === "medium") return "text-signal-amber";
  return "text-signal-green";
}

// ── Filters type ───────────────────────────────────────────────────────

interface Filters {
  text: string;
  sources: Set<string>;
  publishers: Set<string>;
  onlyCatalog: boolean;
  hideSystem: boolean;
  sortBy: "name" | "size" | "date";
}

const DEFAULT_FILTERS: Filters = {
  text: "",
  sources: new Set(),
  publishers: new Set(),
  onlyCatalog: false,
  hideSystem: false,
  sortBy: "name",
};

// ── AppRow ─────────────────────────────────────────────────────────────

interface AppRowProps {
  app: InstalledApp;
  selected: boolean;
  onToggle: () => void;
  onUninstall: (app: InstalledApp) => void;
  uninstalling: boolean;
}

function AppRow({ app, selected, onToggle, onUninstall, uninstalling }: AppRowProps) {
  const [expanded, setExpanded] = useState(false);
  const cat = app.catalogMatch;

  return (
    <>
      <tr
        className={cn(
          "border-b border-edge-default/10 cursor-pointer transition-colors",
          selected ? "bg-signal-cyan/5" : "hover:bg-surface-2",
        )}
        onClick={onToggle}
      >
        <td className="p-3 w-10" onClick={(e) => e.stopPropagation()}>
          <Checkbox
            checked={selected}
            onCheckedChange={onToggle}
            disabled={app.isSystemCritical}
          />
        </td>
        <td className="p-3">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-ink-primary truncate max-w-[220px]">
              {app.displayName}
            </span>
            {app.isSystemCritical && (
              <span className="text-[10px] text-signal-amber bg-signal-amber/10 px-1.5 py-0.5 rounded">
                Sistema
              </span>
            )}
            {cat && (
              <span className={cn("text-[10px] px-1.5 py-0.5 rounded", riskColor(cat.risk))}>
                {cat.category}
              </span>
            )}
          </div>
          {app.publisher && (
            <div className="text-xs text-ink-muted mt-0.5 truncate max-w-[220px]">
              {app.publisher}
            </div>
          )}
        </td>
        <td className="p-3">
          <span className={cn("text-[11px] px-1.5 py-0.5 rounded font-medium", sourceColor(app.source.kind))}>
            {sourceLabel(app)}
          </span>
        </td>
        <td className="p-3 text-xs text-ink-tertiary">
          {app.version ?? "—"}
        </td>
        <td className="p-3 text-xs text-ink-tertiary">
          {app.sizeBytes ? formatBytes(app.sizeBytes) : "—"}
        </td>
        <td className="p-3" onClick={(e) => e.stopPropagation()}>
          <div className="flex items-center gap-1.5">
            <Button
              size="sm"
              variant="outline"
              className="h-6 px-2 text-[11px]"
              disabled={app.isSystemCritical || uninstalling}
              onClick={() => onUninstall(app)}
            >
              {uninstalling ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Trash2 className="h-3 w-3" />
              )}
            </Button>
            <button
              className="p-1 text-ink-muted hover:text-ink-primary"
              onClick={() => setExpanded((v) => !v)}
            >
              {expanded ? <ChevronUp className="h-3.5 w-3.5" /> : <ChevronDown className="h-3.5 w-3.5" />}
            </button>
          </div>
        </td>
      </tr>
      {expanded && (
        <tr className="border-b border-edge-default/10 bg-surface-2/30">
          <td colSpan={6} className="px-6 py-3">
            <div className="text-xs space-y-1 text-ink-tertiary">
              {app.installLocation && (
                <div><span className="text-ink-muted">Ruta:</span> <code className="font-mono">{String(app.installLocation)}</code></div>
              )}
              {app.installDate && (
                <div><span className="text-ink-muted">Instalado:</span> {app.installDate}</div>
              )}
              {cat?.requiresDisclaimer && (
                <div className="flex items-start gap-1.5 text-signal-amber mt-1">
                  <AlertTriangle className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
                  <span>Catálogo: {cat.catalogId} — revisar consecuencias antes de desinstalar</span>
                </div>
              )}
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

// ── ConfirmModal ───────────────────────────────────────────────────────

interface ConfirmModalProps {
  apps: InstalledApp[];
  dryRun: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

function ConfirmModal({ apps, dryRun, onConfirm, onCancel }: ConfirmModalProps) {
  const hasDisclaimer = apps.some((a) => a.catalogMatch?.requiresDisclaimer);
  const hasSystem = apps.some((a) => a.isSystemCritical);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="panel rounded-xl p-6 max-w-md w-full mx-4 space-y-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-5 w-5 text-signal-amber" />
          <h2 className="text-sm font-semibold text-ink-primary">
            {dryRun ? "Simular desinstalación" : "Confirmar desinstalación"}
          </h2>
        </div>
        <p className="text-xs text-ink-secondary">
          {dryRun
            ? `Se simulará la desinstalación de ${apps.length} aplicación${apps.length !== 1 ? "es" : ""} sin modificar el sistema.`
            : `Se desinstalarán ${apps.length} aplicación${apps.length !== 1 ? "es" : ""}. Se creará un punto de restauración previo.`}
        </p>
        <ul className="max-h-32 overflow-y-auto text-xs text-ink-tertiary space-y-0.5">
          {apps.map((a) => (
            <li key={a.id} className="flex items-center gap-2">
              <span className="text-ink-muted">•</span>
              {a.displayName}
              {a.catalogMatch?.requiresDisclaimer && (
                <AlertTriangle className="h-3 w-3 text-signal-amber" />
              )}
            </li>
          ))}
        </ul>
        {hasDisclaimer && (
          <div className="panel p-3 border-signal-amber/20 bg-signal-amber/5 text-xs text-ink-secondary">
            Algunas apps requieren revisión de consecuencias. Verifica que entiendes el impacto.
          </div>
        )}
        {hasSystem && (
          <div className="panel p-3 border-signal-red/20 bg-signal-red/5 text-xs text-signal-red">
            ⚠ Algunas apps están marcadas como críticas del sistema. La desinstalación podría afectar la estabilidad.
          </div>
        )}
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={onCancel}>
            <X className="h-3.5 w-3.5 mr-1.5" />
            Cancelar
          </Button>
          <Button onClick={onConfirm}>
            {dryRun ? (
              <Check className="h-3.5 w-3.5 mr-1.5" />
            ) : (
              <Trash2 className="h-3.5 w-3.5 mr-1.5" />
            )}
            {dryRun ? "Simular" : "Desinstalar"}
          </Button>
        </div>
      </div>
    </div>
  );
}

// ── DebloatPage ────────────────────────────────────────────────────────

export function DebloatPage() {
  const qc = useQueryClient();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [filters, setFilters] = useState<Filters>(DEFAULT_FILTERS);
  const [showFilters, setShowFilters] = useState(false);
  const [confirmApps, setConfirmApps] = useState<InstalledApp[] | null>(null);
  const [dryRun, setDryRun] = useState(false);
  const [uninstallingId, setUninstallingId] = useState<string | null>(null);

  const { data: apps = [], isLoading, error, refetch } = useQuery({
    queryKey: ["installed-apps"],
    queryFn: listInstalledApps,
    staleTime: 5 * 60_000,
    retry: false,
  });

  const uninstallMutation = useMutation({
    mutationFn: ({ app, dry }: { app: InstalledApp; dry: boolean }) =>
      uninstallAppComplete(app, dry),
    onSuccess: (report) => {
      const { uninstall } = report;
      if (report.dryRun) {
        toast.success("Simulación completada", {
          description: `${report.displayName} — sin cambios reales.`,
        });
      } else if (uninstall.success) {
        toast.success(`Desinstalado: ${report.displayName}`, {
          description: report.restorePointSeq
            ? `Restore point #${report.restorePointSeq} creado.`
            : undefined,
        });
        void qc.invalidateQueries({ queryKey: ["installed-apps"] });
      } else {
        toast.error(`Error al desinstalar ${report.displayName}`, {
          description: uninstall.error ?? undefined,
        });
      }
    },
    onError: (err) => toast.error("Error en desinstalación", { description: formatError(err) }),
    onSettled: () => setUninstallingId(null),
  });

  // ── Derived data ──────────────────────────────────────────────────

  const allSources = useMemo(
    () => [...new Set(apps.map((a) => a.source.kind))].sort(),
    [apps],
  );

  const filtered = useMemo(() => {
    let result = apps;
    const text = filters.text.toLowerCase();
    if (text) {
      result = result.filter(
        (a) =>
          a.displayName.toLowerCase().includes(text) ||
          (a.publisher ?? "").toLowerCase().includes(text),
      );
    }
    if (filters.sources.size > 0) {
      result = result.filter((a) => filters.sources.has(a.source.kind));
    }
    if (filters.publishers.size > 0) {
      result = result.filter((a) => a.publisher && filters.publishers.has(a.publisher));
    }
    if (filters.onlyCatalog) {
      result = result.filter((a) => a.catalogMatch !== null);
    }
    if (filters.hideSystem) {
      result = result.filter((a) => !a.isSystemCritical);
    }
    return [...result].sort((a, b) => {
      if (filters.sortBy === "size") return (b.sizeBytes ?? 0) - (a.sizeBytes ?? 0);
      if (filters.sortBy === "date") {
        return (b.installDate ?? "").localeCompare(a.installDate ?? "");
      }
      return a.displayName.localeCompare(b.displayName);
    });
  }, [apps, filters]);

  // ── Handlers ──────────────────────────────────────────────────────

  const toggle = useCallback((id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  }, []);

  const toggleAll = useCallback(() => {
    const eligible = filtered.filter((a) => !a.isSystemCritical).map((a) => a.id);
    setSelected((prev) => {
      const allSelected = eligible.every((id) => prev.has(id));
      if (allSelected) {
        const next = new Set(prev);
        eligible.forEach((id) => next.delete(id));
        return next;
      }
      return new Set([...prev, ...eligible]);
    });
  }, [filtered]);

  const handleSingleUninstall = (app: InstalledApp) => {
    setConfirmApps([app]);
  };

  const handleBatchUninstall = () => {
    const toUninstall = filtered.filter((a) => selected.has(a.id));
    if (toUninstall.length === 0) return;
    setConfirmApps(toUninstall);
  };

  const runUninstall = async () => {
    const appsToUninstall = confirmApps;
    setConfirmApps(null);
    if (!appsToUninstall) return;

    for (const app of appsToUninstall) {
      setUninstallingId(app.id);
      try {
        await uninstallMutation.mutateAsync({ app, dry: dryRun });
      } catch {
        // error handled in onError
      }
    }
  };

  const setFilter = <K extends keyof Filters>(key: K, value: Filters[K]) =>
    setFilters((prev) => ({ ...prev, [key]: value }));

  const toggleSetFilter = (key: "sources" | "publishers", value: string) => {
    setFilters((prev) => {
      const next = new Set(prev[key]);
      next.has(value) ? next.delete(value) : next.add(value);
      return { ...prev, [key]: next };
    });
  };

  // ── Render ────────────────────────────────────────────────────────

  const selectedEligible = filtered.filter((a) => selected.has(a.id) && !a.isSystemCritical);

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-edge-default/10 gap-4 flex-wrap">
        <div>
          <h1 className="text-sm font-semibold text-ink-primary">Debloat</h1>
          <p className="text-xs text-ink-tertiary mt-0.5">
            {isLoading
              ? "Escaneando sistema..."
              : `${filtered.length} de ${apps.length} apps · ${selected.size} seleccionadas`}
          </p>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          <div className="flex items-center gap-1.5">
            <Checkbox
              checked={dryRun}
              onCheckedChange={(v) => setDryRun(!!v)}
              id="dry-run"
            />
            <label htmlFor="dry-run" className="text-xs text-ink-tertiary cursor-pointer">
              Dry-run
            </label>
          </div>
          <Button variant="outline" onClick={() => void refetch()} disabled={isLoading}>
            <Scan className={cn("h-3.5 w-3.5 mr-1.5", isLoading && "animate-spin")} />
            {isLoading ? "Escaneando..." : "Re-escanear"}
          </Button>
          {selectedEligible.length > 0 && (
            <Button onClick={handleBatchUninstall} disabled={uninstallMutation.isPending}>
              <Trash2 className="h-3.5 w-3.5 mr-1.5" />
              Desinstalar {selectedEligible.length}
            </Button>
          )}
        </div>
      </div>

      {/* Search + filter bar */}
      <div className="flex items-center gap-2 px-4 py-2 border-b border-edge-default/10 flex-wrap">
        <div className="relative flex-1 min-w-[180px] max-w-xs">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-ink-muted" />
          <input
            type="text"
            placeholder="Buscar por nombre o publisher..."
            value={filters.text}
            onChange={(e) => setFilter("text", e.target.value)}
            className="inset pl-8 pr-2.5 h-8 text-xs text-ink-primary w-full"
          />
        </div>
        <button
          onClick={() => setShowFilters((v) => !v)}
          className={cn(
            "flex items-center gap-1.5 px-2.5 h-8 rounded text-xs transition-colors",
            showFilters
              ? "bg-signal-cyan/10 text-signal-cyan"
              : "text-ink-tertiary hover:text-ink-primary hover:bg-surface-2",
          )}
        >
          <Filter className="h-3.5 w-3.5" />
          Filtros
          {(filters.sources.size > 0 || filters.publishers.size > 0 || filters.onlyCatalog || filters.hideSystem) && (
            <span className="bg-signal-cyan text-surface-canvas text-[10px] rounded-full w-4 h-4 flex items-center justify-center">
              {filters.sources.size + filters.publishers.size + (filters.onlyCatalog ? 1 : 0) + (filters.hideSystem ? 1 : 0)}
            </span>
          )}
        </button>
        <select
          value={filters.sortBy}
          onChange={(e) => setFilter("sortBy", e.target.value as Filters["sortBy"])}
          className="inset h-8 text-xs text-ink-primary px-2"
        >
          <option value="name">Nombre</option>
          <option value="size">Tamaño</option>
          <option value="date">Fecha</option>
        </select>
      </div>

      {/* Extended filters panel */}
      {showFilters && (
        <div className="px-4 py-3 border-b border-edge-default/10 flex flex-wrap gap-4">
          <div className="space-y-1">
            <p className="text-[10px] uppercase tracking-wider text-ink-muted">Fuente</p>
            <div className="flex flex-wrap gap-1.5">
              {allSources.map((src) => (
                <button
                  key={src}
                  onClick={() => toggleSetFilter("sources", src)}
                  className={cn(
                    "text-[11px] px-2 py-0.5 rounded-full border transition-colors",
                    filters.sources.has(src)
                      ? "border-signal-cyan/40 bg-signal-cyan/10 text-signal-cyan"
                      : "border-edge-default/20 text-ink-muted hover:text-ink-secondary",
                  )}
                >
                  {src}
                </button>
              ))}
            </div>
          </div>
          <div className="space-y-1 flex flex-col gap-1.5">
            <p className="text-[10px] uppercase tracking-wider text-ink-muted">Opciones</p>
            <label className="flex items-center gap-2 text-xs text-ink-tertiary cursor-pointer">
              <Checkbox
                checked={filters.onlyCatalog}
                onCheckedChange={(v) => setFilter("onlyCatalog", !!v)}
              />
              Solo catálogo de bloatware
            </label>
            <label className="flex items-center gap-2 text-xs text-ink-tertiary cursor-pointer">
              <Checkbox
                checked={filters.hideSystem}
                onCheckedChange={(v) => setFilter("hideSystem", !!v)}
              />
              Ocultar componentes del sistema
            </label>
          </div>
        </div>
      )}

      {/* Table */}
      <div className="flex-1 overflow-auto min-h-0">
        {isLoading ? (
          <div className="flex items-center justify-center h-full gap-2 text-xs text-ink-muted">
            <Loader2 className="h-4 w-4 animate-spin" />
            Escaneando todas las fuentes de apps...
          </div>
        ) : error ? (
          <div className="flex items-center justify-center h-full">
            <EmptyState
              icon={AlertTriangle}
              title="Error al escanear"
              description={formatError(error)}
              action={{ label: "Reintentar", onClick: () => void refetch() }}
            />
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <EmptyState
              icon={Package}
              title="Sin resultados"
              description="Prueba ajustando los filtros"
            />
          </div>
        ) : (
          <table className="w-full text-sm border-separate border-spacing-0">
            <thead className="sticky top-0 z-10 bg-surface-base">
              <tr className="border-b border-edge-default/10">
                <th className="p-3 w-10">
                  <Checkbox
                    checked={
                      filtered.filter((a) => !a.isSystemCritical).length > 0 &&
                      filtered.filter((a) => !a.isSystemCritical).every((a) => selected.has(a.id))
                    }
                    onCheckedChange={toggleAll}
                  />
                </th>
                <th className="text-left p-3 text-[11px] font-medium text-ink-muted uppercase tracking-wide">
                  Aplicación
                </th>
                <th className="text-left p-3 text-[11px] font-medium text-ink-muted uppercase tracking-wide">
                  Fuente
                </th>
                <th className="text-left p-3 text-[11px] font-medium text-ink-muted uppercase tracking-wide">
                  Versión
                </th>
                <th className="text-left p-3 text-[11px] font-medium text-ink-muted uppercase tracking-wide">
                  Tamaño
                </th>
                <th className="p-3 w-24" />
              </tr>
            </thead>
            <tbody>
              {filtered.map((app) => (
                <AppRow
                  key={app.id}
                  app={app}
                  selected={selected.has(app.id)}
                  onToggle={() => toggle(app.id)}
                  onUninstall={handleSingleUninstall}
                  uninstalling={uninstallingId === app.id}
                />
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Confirm modal */}
      {confirmApps && (
        <ConfirmModal
          apps={confirmApps}
          dryRun={dryRun}
          onConfirm={() => void runUninstall()}
          onCancel={() => setConfirmApps(null)}
        />
      )}
    </div>
  );
}
