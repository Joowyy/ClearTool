// Hook para eventos de debug del backend (cache:debug).
import { useState, useCallback, useRef } from "react";
import { useTauriEvent } from "./use-tauri-event";

export interface DebugMessage {
  id: number;
  level: "info" | "warn" | "error";
  location: string;
  message: string;
  bytesFreed: number;
  filesDeleted: number;
  timestamp: Date;
}

const MAX_MESSAGES = 500;

export function useCacheDebug() {
  const [messages, setMessages] = useState<DebugMessage[]>([]);
  const idRef = useRef(0);

  const handleEvent = useCallback(
    (event: {
      level: string;
      location: string;
      message: string;
      bytesFreed: number;
      filesDeleted: number;
    }) => {
      idRef.current += 1;
      setMessages((prev) => {
        const next = [
          ...prev,
          {
            id: idRef.current,
            level: (event.level as DebugMessage["level"]) || "info",
            location: event.location,
            message: event.message,
            bytesFreed: event.bytesFreed,
            filesDeleted: event.filesDeleted,
            timestamp: new Date(),
          },
        ];
        if (next.length > MAX_MESSAGES) {
          return next.slice(next.length - MAX_MESSAGES);
        }
        return next;
      });
    },
    []
  );

  useTauriEvent("cache:debug", handleEvent);

  const clear = useCallback(() => setMessages([]), []);

  const copyLog = useCallback(() => {
    const text = messages
      .map(
        (m) =>
          `[${m.timestamp.toLocaleTimeString()}] [${m.level.toUpperCase()}] ${m.location}: ${m.message}`
      )
      .join("\n");
    void navigator.clipboard.writeText(text);
  }, [messages]);

  return { messages, clear, copyLog };
}
