/*
 * table-skeleton — placeholder animado para tablas en carga.
 *
 * Mimic la estructura final con pulse animation.
 * Uso: {isLoading ? <TableSkeleton rows={8} columns={4} /> : <Table>...</Table>}
 */
import { cn } from "../../lib/utils";

type TableSkeletonProps = {
  rows?: number;
  columns?: number;
  className?: string;
};

export function TableSkeleton({ rows = 8, columns = 4, className }: TableSkeletonProps) {
  return (
    <div className={cn("w-full", className)}>
      {/* Header skeleton */}
      <div className="flex items-center gap-4 px-4 py-2 border-b border-edge-default/10">
        {Array.from({ length: columns }).map((_, i) => (
          <div
            key={`h-${i}`}
            className="h-3 w-20 rounded bg-surface-3 animate-pulse"
          />
        ))}
      </div>
      {/* Row skeletons */}
      {Array.from({ length: rows }).map((_, row) => (
        <div
          key={row}
          className="flex items-center gap-4 px-4 py-3 border-b border-edge-default/5"
        >
          {Array.from({ length: columns }).map((_, col) => (
            <div
              key={`${row}-${col}`}
              className={cn(
                "h-3 rounded bg-surface-3 animate-pulse",
                col === 0 ? "w-40" : "w-24",
              )}
              style={{ animationDelay: `${row * 50 + col * 30}ms` }}
            />
          ))}
        </div>
      ))}
    </div>
  );
}
