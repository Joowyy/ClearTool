import { RefreshCw, Search } from "lucide-react";
import { Input } from "../../../components/ui/input";
import { Checkbox } from "../../../components/ui/checkbox";
import { Button } from "../../../components/ui/button";

export function ProcessFilters({
  search, setSearch,
  showSystem, setShowSystem,
  onlyWithUI, setOnlyWithUI,
  isFetching, onRefresh,
}: {
  search: string; setSearch: (v: string) => void;
  showSystem: boolean; setShowSystem: (v: boolean) => void;
  onlyWithUI: boolean; setOnlyWithUI: (v: boolean) => void;
  isFetching: boolean; onRefresh: () => void;
}) {
  return (
    <div className="flex items-center gap-4 flex-wrap">
      <div className="relative flex-1 min-w-48">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder="Buscar proceso..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="pl-9"
        />
      </div>

      <label className="flex items-center gap-2 text-sm cursor-pointer">
        <Checkbox checked={showSystem} onCheckedChange={(v) => setShowSystem(!!v)} />
        Mostrar sistema
      </label>

      <label className="flex items-center gap-2 text-sm cursor-pointer">
        <Checkbox checked={onlyWithUI} onCheckedChange={(v) => setOnlyWithUI(!!v)} />
        Solo con interfaz
      </label>

      <Button
        size="sm"
        variant="outline"
        onClick={onRefresh}
        disabled={isFetching}
      >
        <RefreshCw className={`h-4 w-4 ${isFetching ? "animate-spin" : ""}`} />
      </Button>
    </div>
  );
}
