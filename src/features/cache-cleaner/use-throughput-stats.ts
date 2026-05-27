import { useQuery } from "@tanstack/react-query";
import { getThroughputStats } from "../../api/client";

export interface ThroughputStats {
  samples: number[];
  meanBytesPerSec: number;
  p95BytesPerSec: number;
}

export function useThroughputStats() {
  return useQuery({
    queryKey: ["throughput-stats"],
    queryFn: getThroughputStats,
    staleTime: 60_000,
    refetchOnWindowFocus: false,
  });
}
