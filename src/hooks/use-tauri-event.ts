import { useEffect, useRef, useState } from "react";
import type { EventPayloads, TauriEventName } from "../api/events";

/// Suscripción tipada a un evento del backend Tauri.
///
/// Usa una ref para el handler de modo que el efecto sólo se re-suscribe
/// al cambiar el nombre del evento, evitando stale closures sin tener que
/// silenciar la regla de exhaustive-deps.
export function useTauriEvent<N extends TauriEventName>(
  name: N,
  handler: (payload: EventPayloads[N]) => void,
) {
  const handlerRef = useRef(handler);
  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  const [listenFn, setListenFn] = useState<null | typeof import("@tauri-apps/api/event").listen>(null);

  useEffect(() => {
    import("@tauri-apps/api/event")
      .then(({ listen }) => setListenFn(() => listen))
      .catch(() => {
        // Not running in Tauri
      });
  }, []);

  useEffect(() => {
    if (!listenFn) return;

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listenFn<EventPayloads[N]>(name, (e) => handlerRef.current(e.payload)).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [name, listenFn]);
}
