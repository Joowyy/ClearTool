import { useQuery } from "@tanstack/react-query";
import { systemSummary } from "../lib/tauri";

export function useSystemSummary() {
  return useQuery({
    queryKey: ["system-summary"],
    queryFn: systemSummary,
    staleTime: 30_000,
  });
}
