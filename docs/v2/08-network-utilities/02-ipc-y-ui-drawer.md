# Paso 02 — IPC + UI drawer en Settings

**Área**: 08-network-utilities
**Tiempo estimado**: 3 horas
**Dependencias**: Paso 01

## Qué hacemos

Exponer los 6 comandos vía IPC y crear un drawer/sección en Settings con los 6 botones.

## Archivos

- `src-tauri/src/ipc/network.rs` (nuevo)
- `src-tauri/src/lib.rs` (registrar)
- `src/api/client.ts` (wrappers)
- `src/features/settings/components/network-tools-card.tsx` (nuevo)

## Cómo

### IPC

```rust
// src-tauri/src/ipc/network.rs
use crate::core::AppResult;
use crate::domain::network;

#[tauri::command] pub async fn flush_dns() -> AppResult<String> { network::flush_dns() }
#[tauri::command] pub async fn renew_ip() -> AppResult<String> { network::renew_ip() }
#[tauri::command] pub async fn reset_winsock() -> AppResult<String> { network::reset_winsock() }
#[tauri::command] pub async fn reset_tcpip() -> AppResult<String> { network::reset_tcpip() }
#[tauri::command] pub async fn reset_proxy() -> AppResult<String> { network::reset_proxy() }
#[tauri::command] pub async fn restore_hosts_file() -> AppResult<()> { network::restore_hosts_file() }
```

Registrar en `lib.rs`.

### TS wrappers

```ts
export const flushDns = () => invoke<string>("flush_dns");
export const renewIp = () => invoke<string>("renew_ip");
export const resetWinsock = () => invoke<string>("reset_winsock");
export const resetTcpip = () => invoke<string>("reset_tcpip");
export const resetProxy = () => invoke<string>("reset_proxy");
export const restoreHostsFile = () => invoke<void>("restore_hosts_file");
```

### Card UI

```tsx
// src/features/settings/components/network-tools-card.tsx
import { useMutation } from "@tanstack/react-query";
import {
  flushDns, renewIp, resetWinsock, resetTcpip, resetProxy, restoreHostsFile,
} from "../../../api";
import { Button } from "../../../components/ui/button";
import { toast } from "../../../lib/toast";

interface NetTool {
  id: string;
  label: string;
  description: string;
  warning?: string;
  fn: () => Promise<unknown>;
  requiresRestart?: boolean;
}

const TOOLS: NetTool[] = [
  {
    id: "flush-dns",
    label: "Limpiar caché DNS",
    description: "Borra entradas DNS en caché. Útil tras cambios de DNS.",
    fn: flushDns,
  },
  {
    id: "renew-ip",
    label: "Renovar IP",
    description: "Libera y solicita IP nueva al DHCP.",
    fn: renewIp,
  },
  {
    id: "reset-winsock",
    label: "Resetear Winsock",
    description: "Restaura el stack Winsock a defaults.",
    warning: "Requiere reinicio. Pierde configuración custom de red.",
    fn: resetWinsock,
    requiresRestart: true,
  },
  {
    id: "reset-tcpip",
    label: "Resetear TCP/IP",
    description: "Restaura el stack TCP/IP a defaults.",
    warning: "Requiere reinicio. Pierde configuración custom de IPs estáticas.",
    fn: resetTcpip,
    requiresRestart: true,
  },
  {
    id: "reset-proxy",
    label: "Limpiar caché proxy",
    description: "Quita la configuración de proxy de WinHTTP.",
    fn: resetProxy,
  },
  {
    id: "restore-hosts",
    label: "Restaurar archivo hosts",
    description: "Resetea C:\\Windows\\System32\\drivers\\etc\\hosts a defaults.",
    warning: "Backup automático al lado del hosts original.",
    fn: restoreHostsFile,
  },
];

export function NetworkToolsCard() {
  return (
    <div className="border border-border rounded-lg p-4">
      <h3 className="font-medium mb-3">Herramientas de red</h3>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {TOOLS.map(tool => <NetToolRow key={tool.id} tool={tool} />)}
      </div>
    </div>
  );
}

function NetToolRow({ tool }: { tool: NetTool }) {
  const mut = useMutation({
    mutationFn: tool.fn,
    onSuccess: () => {
      toast.success(`${tool.label} completado`, {
        description: tool.requiresRestart ? "Reinicia para que tenga efecto." : undefined,
      });
    },
    onError: (err) => toast.error(`Error en ${tool.label}`, err),
  });

  const handleClick = () => {
    if (tool.warning) {
      if (!confirm(`${tool.label}\n\n${tool.warning}\n\n¿Continuar?`)) return;
    }
    mut.mutate();
  };

  return (
    <div className="border border-border rounded p-3 flex flex-col gap-2">
      <div className="font-medium text-sm">{tool.label}</div>
      <div className="text-xs text-muted-foreground">{tool.description}</div>
      {tool.warning && (
        <div className="text-xs text-warning">⚠ {tool.warning}</div>
      )}
      <Button
        size="sm"
        variant="outline"
        onClick={handleClick}
        disabled={mut.isPending}
        className="mt-auto"
      >
        {mut.isPending ? "Ejecutando..." : "Ejecutar"}
      </Button>
    </div>
  );
}
```

Integrar en `settings-page.tsx` dentro de tab "Avanzado":

```tsx
<TabsContent value="advanced">
  <NetworkToolsCard />
  <PendingRenamesCard />
  {/* otros */}
</TabsContent>
```

## Criterio de done

- [ ] 6 comandos accesibles vía IPC.
- [ ] Card en Settings → Avanzado con grid de 6 botones.
- [ ] Cada uno: descripción + warning si aplica.
- [ ] Confirm dialog para los destructivos (Winsock, TCP/IP, hosts).
- [ ] Toast tras éxito con hint de reiniciar si aplica.
