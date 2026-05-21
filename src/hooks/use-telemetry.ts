import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { getTelemetrySnapshot, type TelemetrySnapshot } from "../api";
import { useDocumentVisibility } from "./use-document-visibility";

const POLL_INTERVAL_MS = 3000;
const HISTORY_LENGTH = 30; // ~90 s con 3000 ms

/// Hook principal. Devuelve el snapshot actual + el historial reciente
/// de CPU% para alimentar gráficas de línea.
export function useTelemetry() {
  const visible = useDocumentVisibility();

  const query = useQuery({
    queryKey: ["telemetry"],
    queryFn: getTelemetrySnapshot,
    refetchInterval: visible ? POLL_INTERVAL_MS : false,
    staleTime: 0,
  });

  const [cpuHistory, setCpuHistory] = useState<number[]>([]);

  useEffect(() => {
    if (query.data) {
      setCpuHistory((prev) => {
        const next = [...prev, query.data!.cpuTotalPercent];
        if (next.length > HISTORY_LENGTH) next.shift();
        return next;
      });
    }
  }, [query.data]);

  return { ...query, cpuHistory };
}

export type Telemetry = TelemetrySnapshot;
