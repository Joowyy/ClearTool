import { create } from "zustand";

type Theme = "dark" | "light" | "system";

interface AppStore {
  theme: Theme;
  setTheme: (t: Theme) => void;
  isElevated: boolean;
  setElevated: (v: boolean) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  theme: "dark",
  setTheme: (theme) => set({ theme }),
  isElevated: false,
  setElevated: (isElevated) => set({ isElevated }),
}));
