/*
 * use-keyboard-shortcuts — atajos de teclado globales.
 *
 * Se monta en <AppShell> y captura atajos antes que cualquier componente.
 */
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ROUTES } from "../lib/routes";
import { openCommandPalette } from "../components/command-palette";

type ShortcutDef = {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: () => void;
};

const SHORTCUTS: (navigate: ReturnType<typeof useNavigate>) => ShortcutDef[] = (
  navigate,
) => [
  {
    key: "k",
    ctrl: true,
    handler: () => openCommandPalette(),
  },
  {
    key: ",",
    ctrl: true,
    handler: () => navigate(ROUTES.SETTINGS),
  },
  {
    key: "e",
    ctrl: true,
    handler: () => navigate(ROUTES.EXPLORER),
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
    handler: () => {
      /* TODO: open shortcuts modal */
    },
  },
];

export function useKeyboardShortcuts() {
  const navigate = useNavigate();

  useEffect(() => {
    const shortcuts = SHORTCUTS(navigate);
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
