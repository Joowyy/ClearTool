import { create } from "zustand";
import { persist } from "zustand/middleware";

type Theme = "dark-cyan" | "dark-amber" | "light";
type Density = "comfortable" | "compact";

interface AppStore {
  theme: Theme;
  density: Density;
  sidebarCollapsed: boolean;
  followSystem: boolean;
  isElevated: boolean;
  shortcutsModalOpen: boolean;
  setTheme: (t: Theme) => void;
  setDensity: (d: Density) => void;
  toggleSidebar: () => void;
  setFollowSystem: (v: boolean) => void;
  setElevated: (v: boolean) => void;
  toggleShortcutsModal: () => void;
  applyAppearance: () => void;
}

function applyCssVars(theme: Theme, density: Density) {
  document.documentElement.setAttribute("data-theme", theme);
  document.documentElement.setAttribute("data-density", density);
}

export const useAppStore = create<AppStore>()(
  persist(
    (set, get) => ({
      theme: "dark-cyan",
      density: "comfortable",
      sidebarCollapsed: false,
      followSystem: false,
      isElevated: false,
      shortcutsModalOpen: false,
      setTheme: (theme) => {
        set({ theme });
        applyCssVars(theme, get().density);
      },
      setDensity: (density) => {
        set({ density });
        applyCssVars(get().theme, density);
      },
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      setFollowSystem: (followSystem) => set({ followSystem }),
      setElevated: (isElevated) => set({ isElevated }),
      toggleShortcutsModal: () => set((s) => ({ shortcutsModalOpen: !s.shortcutsModalOpen })),
      applyAppearance: () => {
        const { theme, density } = get();
        applyCssVars(theme, density);
      },
    }),
    {
      name: "cleartool-settings",
      partialize: (s) => ({
        theme: s.theme,
        density: s.density,
        sidebarCollapsed: s.sidebarCollapsed,
        followSystem: s.followSystem,
      }),
      onRehydrateStorage: () => (state) => {
        if (state) {
          applyCssVars(state.theme, state.density);
        }
      },
    },
  ),
);
