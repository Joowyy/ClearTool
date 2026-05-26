import { type LucideIcon } from "lucide-react";
import { cn } from "../lib/utils";

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

export function EmptyState({ icon: Icon, title, description, action, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center h-full py-16 text-center", className)}>
      <Icon className="h-12 w-12 text-ink-muted mb-4" />
      <h3 className="text-sm font-semibold text-ink-primary">{title}</h3>
      {description && (
        <p className="text-xs text-ink-tertiary mt-1 max-w-sm">{description}</p>
      )}
      {action && (
        <button
          onClick={action.onClick}
          className="mt-4 inline-flex items-center gap-1.5 px-3 h-8 rounded-md text-xs font-medium bg-signal-cyan/10 text-signal-cyan hover:bg-signal-cyan/20 transition-colors duration-120"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
