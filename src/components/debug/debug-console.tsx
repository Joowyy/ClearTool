// Panel de consola de debug para ver eventos del backend en tiempo real.
import { useState, useRef, useEffect } from "react";
import { Terminal, Copy, Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { Button } from "../ui/button";
import { useCacheDebug } from "../../hooks/use-cache-debug";

function levelColor(level: string): string {
  switch (level) {
    case "error":
      return "text-red-400";
    case "warn":
      return "text-yellow-400";
    default:
      return "text-green-400";
  }
}

export function DebugConsole() {
  const { messages, clear, copyLog } = useCacheDebug();
  const [collapsed, setCollapsed] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  if (messages.length === 0 && collapsed) return null;

  return (
    <div
      className={`border-t border-border bg-zinc-950 transition-all duration-200 ${
        collapsed ? "h-8" : "h-40"
      }`}
    >
      <div className="flex items-center justify-between px-3 h-8 bg-zinc-900 border-b border-zinc-800">
        <button
          onClick={() => setCollapsed((p) => !p)}
          className="flex items-center gap-2 text-xs text-zinc-400 hover:text-zinc-200"
        >
          <Terminal className="h-3.5 w-3.5" />
          <span>Consola de debug</span>
          <span className="text-zinc-600">({messages.length})</span>
          {collapsed ? (
            <ChevronDown className="h-3 w-3" />
          ) : (
            <ChevronUp className="h-3 w-3" />
          )}
        </button>
        {!collapsed && (
          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              className="h-6 px-2 text-xs text-zinc-500 hover:text-zinc-300"
              onClick={copyLog}
            >
              <Copy className="h-3 w-3 mr-1" />
              Copiar
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 px-2 text-xs text-zinc-500 hover:text-zinc-300"
              onClick={clear}
            >
              <Trash2 className="h-3 w-3 mr-1" />
              Limpiar
            </Button>
          </div>
        )}
      </div>
      {!collapsed && (
        <div
          ref={scrollRef}
          className="h-32 overflow-y-auto px-3 py-2 font-mono text-xs"
        >
          {messages.map((m) => (
            <div key={m.id} className="flex gap-2 py-0.5">
              <span className="text-zinc-600 flex-shrink-0">
                {m.timestamp.toLocaleTimeString()}
              </span>
              <span className={`flex-shrink-0 ${levelColor(m.level)}`}>
                [{m.level.toUpperCase()}]
              </span>
              <span className="text-cyan-400 flex-shrink-0">
                {m.location}
              </span>
              <span className="text-zinc-300">{m.message}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
