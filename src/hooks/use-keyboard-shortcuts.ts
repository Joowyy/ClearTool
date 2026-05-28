/*
 * use-keyboard-shortcuts — atajos de teclado globales.
 *
 * Se monta en <AppShell> y captura atajos antes que cualquier componente.
 */
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ROUTES } from "../lib/routes";
import { toggleCommandPalette } from "../components/command-palette";
import { useAppStore } from "../lib/store";

type ShortcutDef = {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: () => void;
};

const SHORTCUTS: (navigate: ReturnType<typeof useNavigate>, toggleShortcutsModal: () => void) => ShortcutDef[] = (
  navigate,
  toggleShortcutsModal,
) => [
  {
    key: "k",
    ctrl: true,
    handler: () => toggleCommandPalette(),
  },
  {
    key: ",",
    ctrl: true,
    handler: () => navigate(ROUTES.SETTINGS),
  },
  {
    key: "b",
    ctrl: true,
    handler: () => navigate(ROUTES.DEBLOAT),
  },
  {
    key: "l",
    ctrl: true,
    handler: () => navigate(ROUTES.CACHE),
  },
  {
    key: "/",
    ctrl: true,
    handler: () => toggleShortcutsModal(),
  },
];

export function useKeyboardShortcuts() {
  const navigate = useNavigate();
  const toggleShortcutsModal = useAppStore((s) => s.toggleShortcutsModal);

  useEffect(() => {
    const shortcuts = SHORTCUTS(navigate, toggleShortcutsModal);
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      for (const s of shortcuts) {
        if (
          e.key.toLowerCase() === s.key.toLowerCase() &&
          !!e.ctrlKey === !!s.ctrl &&
          !!e.shiftKey === !!s.shift &&
          !!e.altKey === !!s.alt
        ) {
          e.preventDefault();
          s.handler();
          return;
        }
      }
    };

    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [navigate]);
}
