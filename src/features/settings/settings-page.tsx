import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAppStore } from "../../lib/store";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Checkbox } from "../../components/ui/checkbox";
import { Badge } from "../../components/ui/badge";
import { Input } from "../../components/ui/input";
import { Moon, Sun, Monitor, RotateCcw, FolderOpen, AlertTriangle, Save, RefreshCw } from "lucide-react";
import {
  getSettings,
  updateSettings,
  resetSettingsToDefaults,
  settingsFilePath,
  openSettingsFile,
  type Settings,
} from "../../api";
import { formatError } from "../../lib/errors";

export function SettingsPage() {
  const { setTheme } = useAppStore();
  const qc = useQueryClient();
  const [localSettings, setLocalSettings] = useState<Settings | null>(null);

  const {
    data: settings,
    isLoading,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
  });

  const updateMutation = useMutation({
    mutationFn: (s: Settings) => updateSettings(s),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["settings"] });
    },
  });

  const resetMutation = useMutation({
    mutationFn: resetSettingsToDefaults,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["settings"] });
      setLocalSettings(null);
    },
  });

  const { data: filePath } = useQuery({
    queryKey: ["settings-file-path"],
    queryFn: settingsFilePath,
    enabled: !!settings,
  });

  const active = localSettings ?? settings;

  if (isLoading) {
    return <div className="p-6 text-muted-foreground">Cargando ajustes...</div>;
  }

  if (!active) return null;

  const hasChanges = localSettings !== null;

  const updateNested = <K extends keyof Settings, N extends keyof Settings[K]>(
    section: K,
    key: N,
    value: Settings[K][N]
  ) => {
    setLocalSettings({
      ...active,
      [section]: { ...(active[section] as unknown as Record<string, unknown>), [key]: value },
    } as Settings);
  };

  const handleSave = () => {
    if (localSettings) {
      updateMutation.mutate(localSettings);
      setLocalSettings(null);
    }
  };

  const handleReset = () => {
    resetMutation.mutate();
  };

  const syncTheme = () => {
    if (settings) {
      setTheme(settings.appearance.theme as "dark" | "light" | "system");
    }
  };

  return (
    <div className="p-6 flex flex-col gap-4 h-full overflow-auto">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Ajustes</h2>
          <p className="text-muted-foreground text-sm">
            Preferencias persistentes de ClearTool
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void refetch()} disabled={isFetching}>
            <RefreshCw className={`h-4 w-4 mr-1 ${isFetching ? "animate-spin" : ""}`} />
            Recargar
          </Button>
          {hasChanges && (
            <Button size="sm" onClick={handleSave} disabled={updateMutation.isPending}>
              <Save className="h-4 w-4 mr-1" />
              {updateMutation.isPending ? "Guardando..." : "Guardar cambios"}
            </Button>
          )}
          <Button variant="outline" size="sm" onClick={handleReset} disabled={resetMutation.isPending}>
            <RotateCcw className="h-4 w-4 mr-1" />
            Restablecer
          </Button>
        </div>
      </div>

      {hasChanges && (
        <div className="flex items-center gap-2 p-3 bg-cyan-900/30 border border-cyan-700/50 rounded-lg text-cyan-400 text-sm">
          <AlertTriangle className="h-4 w-4 flex-shrink-0" />
          Hay cambios sin guardar. Haz clic en "Guardar cambios" para persistirlos.
        </div>
      )}

      {/* Apariencia */}
      <Card>
        <CardHeader>
          <CardTitle>Apariencia</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium mb-2 block">Tema visual</label>
            <div className="flex gap-2">
              <Button
                variant={active.appearance.theme === "dark" ? "default" : "outline"}
                size="sm"
                onClick={() => updateNested("appearance", "theme", "dark")}
              >
                <Moon className="h-4 w-4 mr-1" /> Oscuro
              </Button>
              <Button
                variant={active.appearance.theme === "light" ? "default" : "outline"}
                size="sm"
                onClick={() => updateNested("appearance", "theme", "light")}
              >
                <Sun className="h-4 w-4 mr-1" /> Claro
              </Button>
              <Button
                variant={active.appearance.theme === "system" ? "default" : "outline"}
                size="sm"
                onClick={() => updateNested("appearance", "theme", "system")}
              >
                <Monitor className="h-4 w-4 mr-1" /> Sistema
              </Button>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={syncTheme}>
              Sincronizar tema con la UI
            </Button>
          </div>
          <div>
            <label className="text-sm font-medium mb-1 block">Idioma</label>
            <Input
              value={active.appearance.language}
              onChange={(e) => updateNested("appearance", "language", e.target.value)}
              className="max-w-xs"
            />
          </div>
          <div>
            <label className="text-sm font-medium mb-1 block">Densidad</label>
            <div className="flex gap-2">
              {["compact", "normal", "comfortable"].map((d) => (
                <Button
                  key={d}
                  variant={active.appearance.density === d ? "default" : "outline"}
                  size="sm"
                  onClick={() => updateNested("appearance", "density", d)}
                >
                  {d === "compact" ? "Compacta" : d === "normal" ? "Normal" : "Cómoda"}
                </Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Seguridad */}
      <Card>
        <CardHeader>
          <CardTitle>Seguridad</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.safety.dryRunGlobal}
              onCheckedChange={(v) => updateNested("safety", "dryRunGlobal", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Dry-run global</div>
              <div className="text-xs text-muted-foreground">
                Todas las operaciones serán simuladas por defecto
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.safety.autoCreateRestorePoint}
              onCheckedChange={(v) => updateNested("safety", "autoCreateRestorePoint", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Crear punto de restauración automático</div>
              <div className="text-xs text-muted-foreground">
                Antes de cada operación destructiva
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.safety.requireConfirmBeforeBatch}
              onCheckedChange={(v) => updateNested("safety", "requireConfirmBeforeBatch", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Confirmar antes de operaciones en lote</div>
              <div className="text-xs text-muted-foreground">
                Pedir confirmación explícita para acciones masivas
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.safety.bypassThrottlingRestore}
              onCheckedChange={(v) => updateNested("safety", "bypassThrottlingRestore", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Omitir limitación de 24h de Windows</div>
              <div className="text-xs text-muted-foreground">
                Permitir múltiples puntos de restauración en un día
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Comportamiento */}
      <Card>
        <CardHeader>
          <CardTitle>Comportamiento</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.behavior.checkUpdatesOnStart}
              onCheckedChange={(v) => updateNested("behavior", "checkUpdatesOnStart", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Buscar actualizaciones al inicio</div>
              <div className="text-xs text-muted-foreground">
                Verificar nuevas versiones al abrir la app
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.behavior.rememberWindowSize}
              onCheckedChange={(v) => updateNested("behavior", "rememberWindowSize", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Recordar tamaño de ventana</div>
              <div className="text-xs text-muted-foreground">
                Restaurar dimensiones al reabrir
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Avanzado */}
      <Card>
        <CardHeader>
          <CardTitle>Avanzado</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium mb-1 block">Nivel de log</label>
            <div className="flex gap-2">
              {["error", "warn", "info", "debug", "trace"].map((level) => (
                <Button
                  key={level}
                  variant={active.advanced.logLevel === level ? "default" : "outline"}
                  size="sm"
                  onClick={() => updateNested("advanced", "logLevel", level)}
                >
                  {level}
                </Button>
              ))}
            </div>
          </div>
          <div>
            <label className="text-sm font-medium mb-1 block">
              Tamaño máximo del log de auditoría (MB)
            </label>
            <Input
              type="number"
              value={active.advanced.auditLogMaxMb}
              onChange={(e) =>
                updateNested("advanced", "auditLogMaxMb", parseInt(e.target.value) || 10)
              }
              className="max-w-xs"
            />
          </div>
          <div className="flex items-center gap-3">
            <Checkbox
              checked={active.advanced.diagnosticMode}
              onCheckedChange={(v) => updateNested("advanced", "diagnosticMode", !!v)}
            />
            <div>
              <div className="text-sm font-medium">Modo diagnóstico</div>
              <div className="text-xs text-muted-foreground">
                Logging extendido para depuración
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Acerca de */}
      <Card>
        <CardHeader>
          <CardTitle>Acerca de ClearTool</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">Versión</span>
            <Badge variant="secondary">0.1.0</Badge>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">Stack</span>
            <span className="font-mono text-xs">Tauri 2.x + Rust + React + TypeScript</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">Archivo de ajustes</span>
            <div className="flex gap-1">
              {filePath && (
                <span className="font-mono text-xs text-muted-foreground truncate max-w-[200px]">
                  {filePath}
                </span>
              )}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => void openSettingsFile()}
                title="Abrir carpeta de ajustes"
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {updateMutation.isSuccess && (
        <div className="p-3 bg-green-900/30 border border-green-700/50 rounded-lg text-green-400 text-sm">
          Ajustes guardados correctamente.
        </div>
      )}

      {updateMutation.isError && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400 text-sm">
          Error al guardar: {formatError(updateMutation.error)}
        </div>
      )}
    </div>
  );
}
