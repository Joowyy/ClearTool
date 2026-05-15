import { useSystemSummary } from "../../hooks/use-system-summary";
import { useAppStore } from "../../lib/store";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { formatBytes } from "../../lib/utils";
import { HardDrive, Cpu, User, Shield, ShieldOff } from "lucide-react";
import { Link } from "react-router-dom";
import { ROUTES } from "../../lib/routes";

export function HomePage() {
  const { data: summary, isLoading, error } = useSystemSummary();
  const isElevated = useAppStore((s) => s.isElevated);

  if (isLoading) {
    return (
      <div className="p-6 space-y-4">
        <div className="h-8 bg-muted rounded animate-pulse w-48" />
        <div className="grid grid-cols-2 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="h-32 bg-muted rounded-lg animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <p className="text-destructive-foreground bg-destructive/20 p-3 rounded-md text-sm">
          Error al cargar información del sistema
        </p>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Panel de control</h2>
          <p className="text-muted-foreground text-sm">
            {summary?.os_name} — Build {summary?.build_number}
          </p>
        </div>
        <Badge variant={isElevated ? "success" : "warning"}>
          {isElevated ? (
            <>
              <Shield className="h-3 w-3 mr-1" /> Administrador
            </>
          ) : (
            <>
              <ShieldOff className="h-3 w-3 mr-1" /> Modo limitado
            </>
          )}
        </Badge>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <User className="h-4 w-4" /> Usuario
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{summary?.username}</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <Cpu className="h-4 w-4" /> RAM instalada
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {formatBytes(summary?.total_ram_bytes ?? 0)}
            </p>
          </CardContent>
        </Card>

        {summary?.drives.map((drive) => (
          <Card key={drive.letter}>
            <CardHeader className="pb-2">
              <CardTitle className="flex items-center gap-2 text-base">
                <HardDrive className="h-4 w-4" /> Disco {drive.letter}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Libre</span>
                <span className="font-medium">{formatBytes(drive.free_bytes)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Total</span>
                <span className="font-medium">{formatBytes(drive.total_bytes)}</span>
              </div>
              <div className="w-full bg-muted rounded-full h-2">
                <div
                  className="bg-primary h-2 rounded-full"
                  style={{
                    width: `${Math.min(100, ((drive.total_bytes - drive.free_bytes) / drive.total_bytes) * 100)}%`,
                  }}
                />
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-3">Acciones rápidas</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {[
            { to: ROUTES.CACHE, label: "Limpiar caché", color: "text-blue-400" },
            { to: ROUTES.DEBLOAT, label: "Eliminar bloatware", color: "text-red-400" },
            { to: ROUTES.SERVICES, label: "Gestionar servicios", color: "text-yellow-400" },
            { to: ROUTES.RESTORE, label: "Puntos de restauración", color: "text-green-400" },
          ].map(({ to, label, color }) => (
            <Link
              key={to}
              to={to}
              className="p-4 rounded-lg border border-border bg-card hover:bg-accent/50 transition-colors text-sm font-medium"
            >
              <span className={color}>{label}</span>
            </Link>
          ))}
        </div>
      </div>
    </div>
  );
}
