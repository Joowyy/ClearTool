import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { isElevated } from "../api";
import { useAppStore } from "../lib/store";

export function useElevation() {
  const setElevated = useAppStore((s) => s.setElevated);
  const query = useQuery({
    queryKey: ["elevation"],
    queryFn: isElevated,
    staleTime: 60_000,
  });

  useEffect(() => {
    if (query.data !== undefined) {
      setElevated(query.data);
    }
  }, [query.data, setElevated]);

  return query;
}
