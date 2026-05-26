# Paso 02 — Persistencia del estado del sidebar

**Área**: 11-ui-refactor
**Tiempo estimado**: 1.5 horas
**Dependencias**: Paso 01

## Qué hacemos

Persistir `sidebarCollapsed` en Settings para que el usuario no tenga que volver a colapsarlo cada vez que abre la app.

## Archivos

- `src/lib/store.ts` (modificar — añadir state)
- `src-tauri/src/models/settings.rs` (modificar — añadir campo)
- `src-tauri/src/core/settings.rs` (modificar — defaults)

## Cómo

### 1. Backend: añadir campo a Settings

```rust
// src-tauri/src/models/settings.rs (modificar AppearanceSettings)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,
    pub language: String,
    pub density: String,
    #[serde(default)]
    pub sidebar_collapsed: bool,    // ← NUEVO
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "dark-cyan".into(),
            language: "system".into(),
            density: "comfortable".into(),
            sidebar_collapsed: false,
        }
    }
}
```

### 2. Frontend: cargar y persistir

```ts
// src/lib/store.ts
import { create } from "zustand";
import { getSettings, updateSettings, type Settings } from "../api";

interface AppState {
  isElevated: boolean;
  setElevated: (v: boolean) => void;
  // Sidebar
  sidebarCollapsed: boolean;
  setSidebarCollapsed: (v: boolean) => Promise<void>;
  // Settings cache
  settings: Settings | null;
  loadSettings: () => Promise<void>;
  // ...
}

export const useAppStore = create<AppState>((set, get) => ({
  isElevated: false,
  setElevated: (v) => set({ isElevated: v }),

  sidebarCollapsed: false,
  setSidebarCollapsed: async (v) => {
    set({ sidebarCollapsed: v });
    const settings = get().settings;
    if (settings) {
      const updated = {
        ...settings,
        appearance: { ...settings.appearance, sidebarCollapsed: v },
      };
      set({ settings: updated });
      try { await updateSettings(updated); } catch {}
    }
  },

  settings: null,
  loadSettings: async () => {
    try {
      const s = await getSettings();
      set({
        settings: s,
        sidebarCollapsed: s.appearance.sidebarCollapsed ?? false,
      });
    } catch { /* ignore */ }
  },
}));
```

### 3. Cargar al arrancar

En `AppShell` o donde montes el provider:

```tsx
useEffect(() => {
  void useAppStore.getState().loadSettings();
}, []);
```

### 4. Debounce de save

Si el usuario alterna colapso rápido, hacemos save por cada cambio. Para evitar I/O excesivo, debounce:

```ts
setSidebarCollapsed: (() => {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return async (v: boolean) => {
    set({ sidebarCollapsed: v });
    if (timer) clearTimeout(timer);
    timer = setTimeout(async () => {
      const s = get().settings;
      if (s) {
        const updated = { ...s, appearance: { ...s.appearance, sidebarCollapsed: v } };
        set({ settings: updated });
        try { await updateSettings(updated); } catch {}
      }
    }, 500);
  };
})(),
```

## Criterio de done

- [ ] `sidebarCollapsed` se carga desde settings al arrancar.
- [ ] Al togglear, se persiste con debounce de 500ms.
- [ ] Recargar la app conserva el estado del sidebar.
- [ ] Settings → Apariencia muestra el sidebarCollapsed (informativo, sin necesidad de UI propia).
