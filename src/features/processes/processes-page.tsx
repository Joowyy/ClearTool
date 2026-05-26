import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listProcesses, type ProcessInfo, type ProcessCategory } from "../../api";
import { ProcessGroup } from "./components/process-group";
import { ProcessFilters } from "./components/process-filters";
import { ReleaseButton } from "./components/release-button";
import { Cpu } from "lucide-react";

export function ProcessesPage() {
  const [search, setSearch] = useState("");
  const [showSystem, setShowSystem] = useState(false);
  const [onlyWithUI, setOnlyWithUI] = useState(false);

  const { data: processes = [], refetch, isFetching } = useQuery({
    queryKey: ["processes"],
    queryFn: listProcesses,
    refetchInterval: 2000,
    refetchIntervalInBackground: false,
  });

  const filtered = processes.filter((p) => {
    if (!showSystem && p.category === "system") return false;
    if (onlyWithUI && p.threadCount === 0) return false;
    if (search && !p.name.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  const grouped = filtered.reduce<Record<string, ProcessInfo[]>>((acc, p) => {
    if (!acc[p.category]) acc[p.category] = [];
    acc[p.category].push(p);
    return acc;
  }, {});

  if (processes.length === 0 && !isFetching) {
    return (
      <div className="flex flex-col items-center justify-center h-full py-16 text-center">
        <Cpu className="h-12 w-12 text-muted-foreground mb-4" />
        <h3 className="text-lg font-semibold text-foreground">Sin procesos</h3>
        <p className="text-sm text-muted-foreground mt-1 max-w-sm">
          No se pudieron cargar los procesos del sistema.
        </p>
        <button
          onClick={() => refetch()}
          className="mt-4 px-4 py-2 bg-cyan-600 hover:bg-cyan-700 rounded-md text-sm"
        >
          Reintentar
        </button>
      </div>
    );
  }

  return (
    <div className="p-6 flex flex-col gap-4 h-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Procesos</h2>
          <p className="text-muted-foreground text-sm">
            {processes.length} procesos &middot; {Object.keys(grouped).length} categor&iacute;as
          </p>
        </div>
        <ReleaseButton />
      </div>

      <ProcessFilters
        search={search} setSearch={setSearch}
        showSystem={showSystem} setShowSystem={setShowSystem}
        onlyWithUI={onlyWithUI} setOnlyWithUI={setOnlyWithUI}
        isFetching={isFetching} onRefresh={refetch}
      />

      <div className="flex-1 overflow-auto space-y-3">
        {Object.entries(grouped)
          .sort(([a], [b]) => CATEGORY_ORDER.indexOf(a) - CATEGORY_ORDER.indexOf(b))
          .map(([cat, procs]) => (
            <ProcessGroup key={cat} category={cat as ProcessCategory} processes={procs} />
          ))}
      </div>
    </div>
  );
}

const CATEGORY_ORDER: string[] = [
  "browser", "media", "communication", "development",
  "user-app", "background", "service", "system", "unknown",
];
