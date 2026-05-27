import { useAppStore } from "../lib/store";

const SHORTCUTS = [
  { keys: ["Ctrl", "K"], description: "Abrir paleta de comandos" },
  { keys: ["Ctrl", ","], description: "Ir a Ajustes" },
  { keys: ["Ctrl", "E"], description: "Ir a Explorador" },
  { keys: ["Ctrl", "B"], description: "Ir a Debloat" },
  { keys: ["Ctrl", "L"], description: "Ir a Caché" },
  { keys: ["Ctrl", "/"], description: "Mostrar esta ayuda" },
  { keys: ["Esc"], description: "Cerrar diálogos / paleta" },
];

export function ShortcutsModal() {
  const { shortcutsModalOpen, toggleShortcutsModal } = useAppStore();

  if (!shortcutsModalOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => toggleShortcutsModal()}>
      <div className="bg-surface-1 border border-edge-default rounded-lg p-6 max-w-md mx-4 panel-raised" onClick={(e) => e.stopPropagation()}>
        <h3 className="text-lg font-semibold mb-4">Atajos de teclado</h3>
        <div className="space-y-2">
          {SHORTCUTS.map(({ keys, description }) => (
            <div key={keys.join("+")} className="flex items-center justify-between">
              <span className="text-sm text-ink-secondary">{description}</span>
              <div className="flex gap-1">
                {keys.map((key) => (
                  <kbd key={key} className="px-2 py-1 rounded bg-surface-3 text-ink-tertiary text-xs font-mono border border-edge-default/10">
                    {key}
                  </kbd>
                ))}
              </div>
            </div>
          ))}
        </div>
        <div className="mt-4 text-xs text-ink-muted text-center">
          Pulsa <kbd className="px-1.5 py-0.5 rounded bg-surface-3 font-mono">Esc</kbd> o haz clic fuera para cerrar
        </div>
      </div>
    </div>
  );
}
