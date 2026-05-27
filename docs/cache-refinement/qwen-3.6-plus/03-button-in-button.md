# 03 — `<button>` dentro de `<button>` en PendingRenamesPanel

> **Severidad:** 🟢 P2 — DevTools warning. No rompe, pero invalida el
> HTML y puede causar comportamientos raros con `tabIndex` y `Enter`.

## 1. Problema

Warning de React en consola:

```
Warning: validateDOMNesting(...): <button> cannot appear as a descendant of <button>.
    at button
    at _c (src/components/ui/button.tsx:20:11)
    at div
    at button
    at div
    at PendingRenamesPanel (src/features/cache-cleaner/cache-page.tsx:825:14)
```

(El stack apunta a la línea reportada por DevTools cuando la app está
en runtime; en el código fuente el componente vive más arriba.)

## 2. Causa raíz

`src/features/cache-cleaner/cache-page.tsx:500-535` — el contenedor
exterior de `PendingRenamesPanel` es un `<button>` que abre/cierra el
panel acordeón. Dentro renderiza un `<Button>` ("Cancelar todos")
que es un `<button>` real.

```tsx
<button
  className="w-full flex items-center justify-between..."
  onClick={() => setExpanded((e) => !e)}
>
  <div>...</div>
  <div>
    <Button variant="ghost" onClick={(e) => { e.stopPropagation(); ... }}>
      Cancelar todos
    </Button>
    ...
  </div>
</button>
```

`e.stopPropagation()` evita el click bubble pero **no arregla el HTML**.

## 3. Fix propuesto

Cambiar el contenedor exterior a un `<div>` con role `button` y manejo
explícito de teclado. Eso es accesible y no anida buttons.

`src/features/cache-cleaner/cache-page.tsx:500-535`:

```tsx
return (
  <div className="panel rounded-lg border border-edge-default/20 overflow-hidden">
    <div
      role="button"
      tabIndex={0}
      aria-expanded={expanded}
      aria-controls="pending-renames-list"
      className="w-full flex items-center justify-between px-4 py-2.5 cursor-pointer hover:bg-white/5 transition-colors"
      onClick={() => setExpanded((e) => !e)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          setExpanded((v) => !v);
        }
      }}
    >
      <div className="flex items-center gap-2">
        <Clock className="h-3.5 w-3.5 text-amber-400" />
        <span className="text-xs font-medium text-ink-primary">
          {entries.length} archivo{entries.length !== 1 ? "s" : ""} programado{entries.length !== 1 ? "s" : ""} para borrar en el próximo reboot
        </span>
        <Badge variant="warning" className="text-[10px]">reboot pendiente</Badge>
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          className="h-6 text-[11px] text-red-400 hover:text-red-300"
          onClick={(e) => {
            e.stopPropagation();
            clearAllMutation.mutate();
          }}
          disabled={clearAllMutation.isPending}
        >
          {clearAllMutation.isPending ? (
            <Loader2 className="h-3 w-3 animate-spin mr-1" />
          ) : null}
          Cancelar todos
        </Button>
        {expanded ? (
          <ChevronUp className="h-3.5 w-3.5 text-ink-tertiary" />
        ) : (
          <ChevronDown className="h-3.5 w-3.5 text-ink-tertiary" />
        )}
      </div>
    </div>

    {expanded && (
      <div id="pending-renames-list" className="border-t border-edge-default/10 ...">
        {/* ... resto sin cambios ... */}
      </div>
    )}
  </div>
);
```

Cambios mínimos:

- `<button>` → `<div role="button" tabIndex={0} aria-expanded aria-controls>`.
- `onKeyDown` para soporte de Enter/Space.
- `cursor-pointer` para mantener feedback visual.

## 4. Criterio de done

- [ ] DevTools sin el warning `validateDOMNesting`.
- [ ] Click en cualquier sitio del header expande/colapsa.
- [ ] Click en "Cancelar todos" no expande/colapsa.
- [ ] Tab navega: header → "Cancelar todos" → si expandido, primer item.
- [ ] Enter/Space sobre el header expande/colapsa.

## 5. Riesgos / efectos secundarios

- Ninguno. Es puramente HTML semántico.
- `aria-expanded` + `aria-controls` mejoran la accesibilidad respecto al
  estado actual.
