import { useEffect, useRef, useState } from "react";
import { Check, X, AlertTriangle } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import type { CleanLogLinePayload } from "../../api/events";

interface Props {
  log: CleanLogLinePayload[];
}

const LEVEL = {
  info:    { mark: <span aria-hidden className="text-ink-muted">·</span>, className: "text-ink-secondary" },
  success: { mark: <Check className="h-3.5 w-3.5 text-emerald-400" aria-label="ok" />, className: "text-ink-secondary" },
  warn:    { mark: <AlertTriangle className="h-3.5 w-3.5 text-amber-400" aria-label="aviso" />, className: "text-amber-300/90" },
  error:   { mark: <X className="h-3.5 w-3.5 text-rose-400" aria-label="error" />, className: "text-rose-300/90" },
} as const;

const ANIMATED_TAIL = 12;

export function CleanConsoleLog({ log }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    if (!autoScroll || !containerRef.current) return;
    containerRef.current.scrollTop = containerRef.current.scrollHeight;
  }, [log.length, autoScroll]);

  function onScroll() {
    if (!containerRef.current) return;
    const el = containerRef.current;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 24;
    setAutoScroll(nearBottom);
  }

  const animatedStart = Math.max(0, log.length - ANIMATED_TAIL);

  return (
    <div
      ref={containerRef}
      onScroll={onScroll}
      className="h-64 overflow-y-auto rounded-lg bg-bg-canvas/80 border border-edge-default/40 p-3 font-sans text-[13px] leading-relaxed text-ink-primary/90"
    >
      {log.slice(0, animatedStart).map((line, i) => (
        <LogLine key={i} line={line} animated={false} />
      ))}
      <AnimatePresence initial={false}>
        {log.slice(animatedStart).map((line, i) => (
          <LogLine
            key={animatedStart + i}
            line={line}
            animated={true}
          />
        ))}
      </AnimatePresence>

      {!autoScroll && (
        <button
          className="sticky bottom-1 left-1/2 -translate-x-1/2 text-[10px] px-2 py-0.5 rounded bg-ink-primary/10 text-ink-primary hover:bg-ink-primary/20"
          onClick={() => {
            setAutoScroll(true);
            if (containerRef.current) {
              containerRef.current.scrollTop = containerRef.current.scrollHeight;
            }
          }}
        >
          ↓ Ir al final
        </button>
      )}
    </div>
  );
}

function LogLine({ line, animated }: { line: CleanLogLinePayload; animated: boolean }) {
  const lvl = LEVEL[line.level as keyof typeof LEVEL] ?? LEVEL.info;
  const time = new Date(line.timestampMs).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  const content = (
    <div className={`flex items-center gap-2.5 py-1 ${lvl.className}`}>
      <span className="text-ink-muted tabular-nums text-[11px] w-[58px] shrink-0">
        {time}
      </span>
      <span className="w-4 flex items-center justify-center shrink-0">{lvl.mark}</span>
      <span className="text-ink-tertiary text-[12px] truncate max-w-[180px]" title={line.location}>
        {line.location}
      </span>
      <span className="flex-1 break-words">{line.message}</span>
    </div>
  );

  if (!animated) return content;

  return (
    <motion.div
      initial={{ opacity: 0, x: -3 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.12 }}
    >
      {content}
    </motion.div>
  );
}
