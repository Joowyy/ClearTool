import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useTauriEvent<T>(
  name: string,
  handler: (payload: T) => void
) {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<T>(name, (e) => handler(e.payload)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [name]);
}
