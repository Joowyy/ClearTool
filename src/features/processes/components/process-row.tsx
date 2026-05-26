import { useMutation } from "@tanstack/react-query";
import { Pause, Play, X, Skull } from "lucide-react";
import { Button } from "../../../components/ui/button";
import { killProcess, suspendProcess, resumeProcess, closeGracefully, type ProcessInfo } from "../../../api";
import { toast } from "../../../lib/toast";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export function ProcessRow({ process }: { process: ProcessInfo }) {
  const close = useMutation({
    mutationFn: () => closeGracefully(process.pid, 5000),
    onSuccess: (graceful) => toast.success(
      graceful ? `${process.name} cerrada` : `${process.name} forzada (no respond\u00eda)`
    ),
    onError: (err) => toast.error(`No se pudo cerrar ${process.name}`, err),
  });

  const kill = useMutation({
    mutationFn: () => killProcess(process.pid),
    onSuccess: () => toast.success(`${process.name} terminada`),
    onError: (err) => toast.error(`No se pudo terminar ${process.name}`, err),
  });

  const suspend = useMutation({
    mutationFn: () => suspendProcess(process.pid),
    onSuccess: () => toast.success(`${process.name} suspendida`),
    onError: (err) => toast.error(`No se pudo suspender ${process.name}`, err),
  });

  const resume = useMutation({
    mutationFn: () => resumeProcess(process.pid),
    onSuccess: () => toast.success(`${process.name} reanudada`),
    onError: (err) => toast.error(`No se pudo reanudar ${process.name}`, err),
  });

  return (
    <div className="px-4 py-2 flex items-center gap-3 hover:bg-accent/20 text-sm">
      <div className="flex-1 min-w-0">
        <div className="font-medium truncate">
          {process.displayName ?? process.name}
        </div>
        <div className="text-xs text-muted-foreground truncate">
          PID {process.pid} &middot; {process.exePath ?? "(sin path)"}
        </div>
      </div>
      <div className="text-xs font-mono text-muted-foreground">
        {process.cpuPercent.toFixed(1)}%
      </div>
      <div className="text-xs font-mono text-muted-foreground w-20 text-right">
        {formatBytes(process.memoryBytes)}
      </div>
      <div className="flex gap-1">
        <Button
          size="sm"
          variant="outline"
          disabled={process.isProtected || close.isPending}
          onClick={() => close.mutate()}
          title={process.isProtected ? "Proceso protegido" : "Cerrar (WM_CLOSE)"}
        >
          <X className="h-3 w-3" />
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={process.isProtected || kill.isPending}
          onClick={() => {
            if (confirm(`\u00bfTerminar ${process.name} forzadamente? Puede perderse datos.`)) {
              kill.mutate();
            }
          }}
          title="Kill (forzar)"
        >
          <Skull className="h-3 w-3" />
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={process.isProtected || suspend.isPending}
          onClick={() => suspend.mutate()}
          title="Suspender"
        >
          <Pause className="h-3 w-3" />
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={process.isProtected || resume.isPending}
          onClick={() => resume.mutate()}
          title="Reanudar"
        >
          <Play className="h-3 w-3" />
        </Button>
      </div>
    </div>
  );
}
