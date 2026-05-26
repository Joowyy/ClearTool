import { useEffect, useState } from "react";

/// Suscripción ligera al `document.visibilityState`. Útil para pausar
/// polling cuando la ventana de la app pasa a segundo plano.
export function useDocumentVisibility(): boolean {
  const [visible, setVisible] = useState(
    typeof document !== "undefined" ? !document.hidden : true,
  );

  useEffect(() => {
    const handler = () => setVisible(!document.hidden);
    document.addEventListener("visibilitychange", handler);
    return () => document.removeEventListener("visibilitychange", handler);
  }, []);

  return visible;
}
