// drive-selector — dropdown que lista los discos del sistema con barras
// de uso. La selección es controlada por el padre vía `value` / `onChange`.
import * as Dropdown from "@radix-ui/react-dropdown-menu";
import { motion, AnimatePresence } from "framer-motion";
import {
  ChevronDown,
  HardDrive,
  Usb,
  Network,
  Disc,
  CircuitBoard,
  RefreshCw,
} from "lucide-react";
import { useState } from "react";
import { cn } from "../../lib/utils";
import { formatBytes } from "../../lib/utils";
import type { DriveListing, DriveType } from "../../api/types";

interface DriveSelectorProps {
  drives: DriveListing[];
  value: DriveListing | null;
  onChange: (drive: DriveListing) => void;
  onRefresh: () => void;
  loading?: boolean;
}

const DRIVE_TYPE_ICON: Record<DriveType, typeof HardDrive> = {
  fixed: HardDrive,
  removable: Usb,
  network: Network,
  cdRom: Disc,
  ramDisk: CircuitBoard,
  unknown: HardDrive,
};

const DRIVE_TYPE_LABEL: Record<DriveType, string> = {
  fixed: "Interno",
  removable: "USB / Extraíble",
  network: "Red",
  cdRom: "Óptico",
  ramDisk: "RAM disk",
  unknown: "Otro",
};

function usagePercent(d: DriveListing): number {
  if (!d.totalBytes) return 0;
  const used = d.totalBytes - d.freeBytes;
  return Math.max(0, Math.min(100, (used / d.totalBytes) * 100));
}

function usageColor(pct: number): string {
  if (pct >= 90) return "bg-red-500";
  if (pct >= 75) return "bg-amber-500";
  if (pct >= 50) return "bg-cyan-500";
  return "bg-emerald-500";
}

export function DriveSelector({
  drives,
  value,
  onChange,
  onRefresh,
  loading = false,
}: DriveSelectorProps) {
  const [open, setOpen] = useState(false);

  const Icon = value ? DRIVE_TYPE_ICON[value.driveType] : HardDrive;

  return (
    <div className="flex items-center gap-2">
      <Dropdown.Root open={open} onOpenChange={setOpen}>
        <Dropdown.Trigger
          className={cn(
            "no-drag inline-flex items-center gap-2 h-9 px-3 rounded-md",
            "bg-surface-inset border border-edge-default/15",
            "hover:border-signal-cyan/30 hover:bg-signal-cyan/[0.04]",
            "transition-all duration-150 text-left min-w-[220px]",
          )}
        >
          <Icon className="h-4 w-4 text-signal-cyan flex-shrink-0" strokeWidth={1.75} />
          {value ? (
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-1.5">
                <span className="font-mono text-xs text-ink-primary font-medium">
                  {value.letter}:
                </span>
                {value.label && (
                  <span className="text-xs text-ink-tertiary truncate">
                    {value.label}
                  </span>
                )}
                <span className="text-[10px] text-ink-muted uppercase tracking-wider ml-auto">
                  {DRIVE_TYPE_LABEL[value.driveType]}
                </span>
              </div>
              <div className="mt-1 flex items-center gap-1.5">
                <div className="relative flex-1 h-1 rounded-full bg-edge-default/20 overflow-hidden">
                  <div
                    className={cn(
                      "absolute inset-y-0 left-0 rounded-full",
                      usageColor(usagePercent(value)),
                    )}
                    style={{ width: `${usagePercent(value)}%` }}
                  />
                </div>
                <span className="text-[10px] font-mono text-ink-muted tabular-nums">
                  {Math.round(usagePercent(value))}%
                </span>
              </div>
            </div>
          ) : (
            <span className="text-xs text-ink-tertiary flex-1">
              {loading ? "Detectando discos…" : "Selecciona un disco"}
            </span>
          )}
          <ChevronDown
            className={cn(
              "h-3.5 w-3.5 text-ink-muted transition-transform duration-150",
              open && "rotate-180",
            )}
          />
        </Dropdown.Trigger>

        <Dropdown.Portal>
          <Dropdown.Content
            sideOffset={4}
            align="start"
            className={cn(
              "z-50 min-w-[340px] panel-raised rounded-lg p-1.5",
              "border border-edge-default/15 shadow-2xl",
              "data-[state=open]:animate-in data-[state=closed]:animate-out",
              "data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0",
              "data-[state=open]:zoom-in-95",
            )}
          >
            <div className="px-2 py-1.5 text-[10px] uppercase tracking-wider text-ink-muted">
              Discos del sistema
            </div>
            {drives.length === 0 && !loading && (
              <div className="px-2 py-3 text-xs text-ink-tertiary text-center">
                No se detectaron discos.
              </div>
            )}
            {drives.map((d) => {
              const ItemIcon = DRIVE_TYPE_ICON[d.driveType];
              const pct = usagePercent(d);
              const disabled = !d.isReady || d.driveType === "network";
              const selected = value?.rootPath === d.rootPath;
              return (
                <Dropdown.Item
                  key={d.rootPath}
                  disabled={disabled}
                  onSelect={() => !disabled && onChange(d)}
                  className={cn(
                    "group flex items-center gap-2.5 px-2 py-2 rounded-md cursor-pointer outline-none",
                    "data-[highlighted]:bg-signal-cyan/[0.06]",
                    "data-[disabled]:opacity-40 data-[disabled]:cursor-not-allowed",
                    selected && "bg-signal-cyan/[0.08]",
                  )}
                >
                  <ItemIcon
                    className={cn(
                      "h-4 w-4 flex-shrink-0",
                      selected ? "text-signal-cyan" : "text-ink-tertiary",
                    )}
                    strokeWidth={1.75}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-baseline gap-1.5">
                      <span className="font-mono text-xs font-semibold text-ink-primary">
                        {d.letter}:
                      </span>
                      {d.label && (
                        <span className="text-xs text-ink-secondary truncate">
                          {d.label}
                        </span>
                      )}
                      {d.filesystem && (
                        <span className="text-[10px] uppercase tracking-wider text-ink-muted ml-auto">
                          {d.filesystem}
                        </span>
                      )}
                    </div>
                    <div className="mt-1 flex items-center gap-1.5">
                      <div className="relative flex-1 h-1 rounded-full bg-edge-default/20 overflow-hidden">
                        <div
                          className={cn(
                            "absolute inset-y-0 left-0 rounded-full",
                            usageColor(pct),
                          )}
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                      <span className="text-[10px] font-mono text-ink-muted tabular-nums whitespace-nowrap">
                        {formatBytes(d.totalBytes - d.freeBytes)} / {formatBytes(d.totalBytes)}
                      </span>
                    </div>
                    {d.driveType === "network" && (
                      <div className="mt-1 text-[10px] text-amber-400/80">
                        Drives de red no soportados todavía
                      </div>
                    )}
                    {!d.isReady && (
                      <div className="mt-1 text-[10px] text-ink-muted">No listo</div>
                    )}
                  </div>
                </Dropdown.Item>
              );
            })}
          </Dropdown.Content>
        </Dropdown.Portal>
      </Dropdown.Root>

      <button
        onClick={onRefresh}
        disabled={loading}
        title="Refrescar discos"
        className={cn(
          "no-drag inline-flex items-center justify-center h-9 w-9 rounded-md",
          "bg-surface-inset border border-edge-default/15",
          "hover:border-signal-cyan/30 hover:text-signal-cyan",
          "text-ink-tertiary transition-colors disabled:opacity-50",
        )}
      >
        <AnimatePresence mode="wait" initial={false}>
          <motion.div
            key={loading ? "spinning" : "idle"}
            animate={loading ? { rotate: 360 } : { rotate: 0 }}
            transition={loading ? { repeat: Infinity, duration: 1, ease: "linear" } : { duration: 0 }}
          >
            <RefreshCw className="h-3.5 w-3.5" strokeWidth={1.75} />
          </motion.div>
        </AnimatePresence>
      </button>
    </div>
  );
}
