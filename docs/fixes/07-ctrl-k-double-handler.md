# Fix 07 — Ctrl+K duplicado: cerrar la paleta la reabre

## Síntoma

Abrir el Command Palette con Ctrl+K funciona. Pulsar Ctrl+K otra vez (esperando
cerrarla) **deja la paleta abierta**.

## Causa raíz

Hay dos listeners de Ctrl+K registrados en `document` simultáneamente:

1. [`src/components/command-palette.tsx:170-179`](../../src/components/command-palette.tsx#L170-L179)
   ```ts
   if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
     e.preventDefault();
     setOpen((o) => !o);   // ← toggle
   }
   ```
2. [`src/hooks/use-keyboard-shortcuts.ts:22-26`](../../src/hooks/use-keyboard-shortcuts.ts#L22-L26)
   ```ts
   { key: "k", ctrl: true, handler: () => openCommandPalette() }
   //                       openCommandPalette llama a globalOpen()
   //                       globalOpen = () => setOpen(true)   ← siempre abre
   ```

Ambos llaman `preventDefault` pero ambos ejecutan su `handler`. Resultado:

| Estado | Toggle (#1) ejecuta | Abrir-siempre (#2) ejecuta | Estado final |
|---|---|---|---|
| Cerrado → pulsar Ctrl+K | `setOpen(true)` | `setOpen(true)` | **Abierto** ✓ |
| Abierto → pulsar Ctrl+K | `setOpen(false)` | `setOpen(true)` | **Abierto** ✗ |

## Fix propuesto

Eliminar el listener interno de `CommandPalette` y dejar `useKeyboardShortcuts`
como única fuente, cambiando `openCommandPalette` por un toggle:

```ts
// command-palette.tsx
let globalToggle: (() => void) | null = null;

export function toggleCommandPalette() {
  globalToggle?.();
}

export function CommandPalette() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    globalToggle = () => setOpen((o) => !o);
    return () => { globalToggle = null; };
  }, []);

  // BORRAR el useEffect que escucha keydown — ya no es necesario.

  return <Command.Dialog open={open} onOpenChange={setOpen} ... />;
}
```

```ts
// use-keyboard-shortcuts.ts
import { toggleCommandPalette } from "../components/command-palette";

{ key: "k", ctrl: true, handler: () => toggleCommandPalette() }
```

## Coste

- < 30 minutos.

## Archivos a tocar

- `src/components/command-palette.tsx` (renombrar export, quitar listener).
- `src/hooks/use-keyboard-shortcuts.ts` (importar `toggleCommandPalette`).

## Aceptación

1. Ctrl+K abre.
2. Ctrl+K cierra.
3. Escape cierra (sin cambios).
4. Click fuera cierra (sin cambios).
