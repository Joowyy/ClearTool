import { toast as sonner } from "sonner";
import { normalizeError } from "./errors";

interface ToastErrorOptions {
  action?: { label: string; onClick: () => void };
  duration?: number;
  actionVariant?: "primary" | "destructive";
}

export const toast = {
  success(message: string, opts?: { description?: string; duration?: number }) {
    sonner.success(message, {
      description: opts?.description,
      duration: opts?.duration ?? 4000,
    });
  },

  info(message: string, opts?: { description?: string }) {
    sonner.info(message, { description: opts?.description, duration: 5000 });
  },

  warning(message: string, opts?: { description?: string; action?: ToastErrorOptions["action"] }) {
    sonner.warning(message, {
      description: opts?.description,
      duration: 6000,
      action: opts?.action,
    });
  },

  error(message: string, err?: unknown, opts?: ToastErrorOptions) {
    if (err === undefined) {
      sonner.error(message, { duration: opts?.duration ?? 8000, action: opts?.action });
      return;
    }
    const n = normalizeError(err);
    const action = opts?.action ?? (n.isPermission
      ? {
          label: "Reiniciar como admin",
          onClick: () => {
            void import("../api").then(m => m.relaunchAsAdmin?.());
          },
        }
      : undefined);

    sonner.error(message, {
      description: n.message,
      duration: opts?.duration ?? 8000,
      action,
    });
  },

  promise<T>(
    promise: Promise<T>,
    msgs: { loading: string; success: string | ((data: T) => string); error: string }
  ) {
    return sonner.promise(promise, {
      loading: msgs.loading,
      success: msgs.success,
      error: (err) => `${msgs.error}: ${normalizeError(err).message}`,
    });
  },

  dismiss() {
    sonner.dismiss();
  },
};
