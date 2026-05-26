/*
 * window-controls — fallback HTML para min/max/close.
 *
 * En Windows, decorum inyecta un overlay nativo que reemplaza estos botones
 * y conserva Snap Layouts. Este componente sólo se renderiza si decorum no
 * está activo (p. ej. macOS/Linux durante desarrollo). En Windows queda
 * oculto vía `data-show-on` para que no aparezcan duplicados.
 */
import { useEffect, useState } from "react";
import { Minus, Square, X, Copy } from "lucide-react";

export function WindowControls() {
  const [isMaximized, setIsMaximized] = useState(false);
  const [tauriApi, setTauriApi] = useState<null | { getCurrentWindow: () => unknown }>(null);

  useEffect(() => {
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => {
        setTauriApi({ getCurrentWindow });
      })
      .catch(() => {
        // Not running in Tauri
      });
  }, []);

  useEffect(() => {
    if (!tauriApi) return;
    const { getCurrentWindow } = tauriApi;
    const win = getCurrentWindow() as { isMaximized: () => Promise<boolean>; onResized: (cb: () => void) => Promise<() => void> };

    let unlisten: (() => void) | undefined;

    void win.isMaximized().then(setIsMaximized);
    void win
      .onResized(() => {
        void win.isMaximized().then(setIsMaximized);
      })
      .then((u) => {
        unlisten = u;
      });

    return () => {
      unlisten?.();
    };
  }, [tauriApi]);

  const handleAction = async (action: "minimize" | "toggleMaximize" | "close") => {
    if (!tauriApi) return;
    const { getCurrentWindow } = tauriApi;
    const win = getCurrentWindow() as { minimize: () => void; toggleMaximize: () => void; close: () => void };
    if (action === "minimize") void win.minimize();
    else if (action === "toggleMaximize") void win.toggleMaximize();
    else void win.close();
  };

  return (
    <div className="flex items-center h-full no-drag" data-platform-controls>
      <button
        type="button"
        aria-label="Minimizar"
        onClick={() => void handleAction("minimize")}
        className="h-full w-11 inline-flex items-center justify-center
                   text-ink-tertiary hover:text-ink-primary
                   hover:bg-white/[0.04] transition-colors duration-120"
      >
        <Minus className="h-3.5 w-3.5" strokeWidth={1.5} />
      </button>
      <button
        type="button"
        aria-label={isMaximized ? "Restaurar" : "Maximizar"}
        onClick={() => void handleAction("toggleMaximize")}
        className="h-full w-11 inline-flex items-center justify-center
                   text-ink-tertiary hover:text-ink-primary
                   hover:bg-white/[0.04] transition-colors duration-120"
      >
        {isMaximized ? (
          <Copy className="h-3 w-3 -scale-x-100" strokeWidth={1.5} />
        ) : (
          <Square className="h-3 w-3" strokeWidth={1.5} />
        )}
      </button>
      <button
        type="button"
        aria-label="Cerrar"
        onClick={() => void handleAction("close")}
        className="h-full w-11 inline-flex items-center justify-center
                   text-ink-tertiary hover:text-white hover:bg-signal-red
                   transition-colors duration-120"
      >
        <X className="h-3.5 w-3.5" strokeWidth={1.5} />
      </button>
    </div>
  );
}
