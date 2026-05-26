/*
 * confirm-dialog — sistema de confirmación promise-based.
 *
 * Patrón: const ok = await confirm({ title, description, ... });
 * Evita state local repetido en cada página.
 */
import { useState, useCallback, createContext, useContext, type ReactNode } from "react";
import * as AlertDialog from "@radix-ui/react-alert-dialog";
import { AlertTriangle } from "lucide-react";
import { cn } from "../../lib/utils";

type ConfirmOptions = {
  title: string;
  description: string;
  consequences?: string[];
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};

type ConfirmResolve = (value: boolean) => void;

interface ConfirmContextValue {
  confirm: (opts: ConfirmOptions) => Promise<boolean>;
}

const ConfirmContext = createContext<ConfirmContextValue | null>(null);

let pendingResolve: ConfirmResolve | null = null;

function ConfirmProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const [opts, setOpts] = useState<ConfirmOptions | null>(null);

  const confirm = useCallback((options: ConfirmOptions) => {
    setOpts(options);
    setOpen(true);
    return new Promise<boolean>((resolve) => {
      pendingResolve = resolve;
    });
  }, []);

  const handleClose = useCallback(
    (value: boolean) => {
      setOpen(false);
      pendingResolve?.(value);
      pendingResolve = null;
    },
    [],
  );

  return (
    <ConfirmContext.Provider value={{ confirm }}>
      {children}
      <AlertDialog.Root open={open} onOpenChange={(o) => !o && handleClose(false)}>
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 bg-black/60 z-40" />
          <AlertDialog.Content className="fixed z-50 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md panel-raised p-6 animate-slide-up">
            <div className="flex items-start gap-3">
              {opts?.danger && (
                <AlertTriangle className="h-5 w-5 text-signal-red flex-shrink-0 mt-0.5" />
              )}
              <div className="flex-1">
                <AlertDialog.Title className="text-sm font-semibold text-ink-primary">
                  {opts?.title}
                </AlertDialog.Title>
                <AlertDialog.Description className="mt-1 text-xs text-ink-secondary leading-relaxed">
                  {opts?.description}
                </AlertDialog.Description>
                {opts?.consequences && opts.consequences.length > 0 && (
                  <ul className="mt-3 space-y-1">
                    {opts.consequences.map((c, i) => (
                      <li key={i} className="text-2xs text-ink-tertiary flex items-start gap-1.5">
                        <span className="text-signal-amber mt-px">•</span>
                        {c}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
            <div className="mt-5 flex justify-end gap-2">
              <AlertDialog.Cancel asChild>
                <button
                  className={cn(
                    "inline-flex items-center justify-center px-3 h-8 rounded-md text-xs font-medium",
                    "bg-surface-2 text-ink-secondary hover:text-ink-primary hover:bg-surface-3",
                    "transition-colors duration-120",
                  )}
                >
                  {opts?.cancelLabel || "Cancelar"}
                </button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <button
                  onClick={() => handleClose(true)}
                  className={cn(
                    "inline-flex items-center justify-center px-3 h-8 rounded-md text-xs font-medium",
                    "transition-colors duration-120",
                    opts?.danger
                      ? "bg-signal-red text-white hover:bg-signal-red-press"
                      : "bg-signal-cyan text-surface-canvas hover:bg-signal-cyan-press",
                  )}
                >
                  {opts?.confirmLabel || "Confirmar"}
                </button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>
    </ConfirmContext.Provider>
  );
}

function useConfirm() {
  const ctx = useContext(ConfirmContext);
  if (!ctx) throw new Error("useConfirm must be used within ConfirmProvider");
  return ctx.confirm;
}

export { ConfirmProvider, useConfirm };
export type { ConfirmOptions };
