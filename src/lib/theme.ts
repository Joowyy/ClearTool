import { useEffect } from "react";
import { useAppStore } from "./store";

export function useThemeEffect() {
  const theme = useAppStore((s) => s.theme);
  const followSystem = useAppStore((s) => s.followSystem);

  useEffect(() => {
    const root = document.documentElement;
    const effective = followSystem
      ? (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark-cyan" : "light")
      : theme;
    root.dataset.theme = effective;

    if (followSystem) {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = (e: MediaQueryListEvent) => {
        root.dataset.theme = e.matches ? "dark-cyan" : "light";
      };
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    }
  }, [theme, followSystem]);
}
