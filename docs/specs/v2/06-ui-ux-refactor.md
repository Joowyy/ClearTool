# v2 · 06 — UI/UX Refactor (P2)

## El punto de partida

UI actual ("Quirófano Cyan"):
- Titlebar HTML con 9 tabs.
- Páginas con tablas densas, todas con la misma estructura repetida.
- Dashboard con tilt cards y particles.
- Tema único cyan oscuro.
- Sin búsqueda global.
- Sin atajos de teclado documentados.
- Errores inline en cada página.

Lo que funciona:
- Estética coherente.
- Densidad alta — power user agradece.
- Animaciones discretas pero presentes.

Lo que está roto / por hacer:
- Titlebar saturada cuando hay 9 tabs + status chips + window controls. Con un idioma más largo (inglés) se rompe.
- Doble titlebar bug en v0.1 (ya parcheado).
- Errores `[object Object]` (ya tratado en spec `01`).
- Sin acceso rápido a acciones frecuentes (sin Cmd+K).
- Sin localización: todo hardcoded en español.
- Tema único — usuarios con monitores diurnos o con preferencias OS pueden querer light theme.
- El árbol del Explorador no tiene los estilos 3D pedidos (parcheado en `docs/fixes/02-tree-recursive-3d.md`).

## Decisiones de diseño v2

### Decisión 1 — Sidebar colapsable en lugar de tabs en titlebar

Razones:
1. **Escala**. Con 12+ módulos planeados (los actuales 9 + procesos + startup + disk analyzer), la titlebar no aguanta.
2. **i18n**. Strings más largos en inglés/portugués revientan los pills horizontales.
3. **Información por sección**. Un sidebar puede mostrar badges (RP cantidad, audit unread, pending reboot) sin saturar.
4. **Conformidad con apps modernas**. VSCode, Linear, Discord, Spotify — todo sidebar a la izquierda en Windows.

```
┌──────────────────────────────────────────────────────────────────────┐
│ ─── ClearTool v1.0                       [Admin] [joels] [_][□][×]   │ ← titlebar fina, solo logo + status chips + window controls
├──────────────────────────────────────────────────────────────────────┤
│ ☰  │  📊  Inicio                                                     │
│    │  📁  Explorador                                                 │
│    │  🧹  Caché                              [3.2 GB →]              │
│ ☷  │  🛒  Debloat                            [25 ★]                  │
│    │  ⚙   Servicios                                                  │
│    │  📋  Registro                                                   │
│    │  🔄  Restauración                       [4 puntos]              │
│ 📈 │  📜  Auditoría                          [12 nuevos]             │
│    │  ⚡  Procesos                                                   │
│    │  🚀  Arranque                           [3 alto]                │
│    │  💾  Disco                                                      │
│    │  🛡   Privacidad                                                 │
│    │  ⚙   Ajustes                                                    │
└──────────────────────────────────────────────────────────────────────┘
```

Width:
- Expanded: 220px.
- Collapsed (default): 56px (solo iconos).
- Estado persistido en Settings (`appearance.sidebarCollapsed`).

Animación: 180ms ease para colapsar/expandir.

### Decisión 2 — Command Palette (Ctrl+K / Cmd+K)

Atajo global que abre un overlay con búsqueda fuzzy contra:
- Páginas (`Ir a → Caché`, `Ir a → Procesos`).
- Acciones (`Limpiar caché`, `Crear restore point`, `Detectar bloatware`).
- Entradas de catálogo (`Debloat → Microsoft Copilot`, `Tweak → Desactivar widgets`).
- Settings (`Cambiar tema`, `Toggle dry-run global`).

Lib: `cmdk` (Vercel) — ~6KB, A11y impecable, integración React limpia.

```tsx
// src/components/command-palette.tsx
<Command.Dialog open={open} onOpenChange={setOpen}>
  <Command.Input placeholder="Busca acciones, módulos, apps..." />
  <Command.List>
    <Command.Group heading="Navegar">
      <Command.Item onSelect={() => navigate('/cache')}>Caché</Command.Item>
      <Command.Item onSelect={() => navigate('/debloat')}>Debloat</Command.Item>
    </Command.Group>
    <Command.Group heading="Acciones rápidas">
      <Command.Item onSelect={runScanCache}>Escanear caché</Command.Item>
      <Command.Item onSelect={createRestorePoint}>Crear restore point</Command.Item>
    </Command.Group>
    <Command.Group heading="Debloat (catálogo)">
      {bloatwareEntries.map(e => (
        <Command.Item key={e.id} onSelect={() => navigate(`/debloat?selected=${e.id}`)}>
          {e.displayName}
        </Command.Item>
      ))}
    </Command.Group>
  </Command.List>
</Command.Dialog>
```

### Decisión 3 — Sistema de temas

Tres themes built-in:
- `dark-cyan` (actual)
- `dark-amber` (alternativa cálida)
- `light` (para usuarios diurnos)

Implementación: CSS variables en `:root` + clase en `<html>`:
```css
:root[data-theme="dark-cyan"] { --signal-accent: 6 182 212; ... }
:root[data-theme="dark-amber"] { --signal-accent: 245 158 11; ... }
:root[data-theme="light"]     { --signal-accent: 8 145 178; --bg-canvas: 250 250 250; ... }
```

Settings: dropdown que escribe a `appearance.theme`. La pref se persiste y aplica al arrancar.

Adicional: `appearance.followSystem: bool` que escucha `prefers-color-scheme` y cambia automáticamente.

### Decisión 4 — Density toggle

Comfortable vs Compact:
- Comfortable (default): padding 12-16px, font 14px.
- Compact: padding 6-10px, font 13px. Para power user con muchos datos.

CSS:
```css
:root[data-density="compact"] {
  --table-cell-padding: 6px 10px;
  --row-height: 32px;
}
:root[data-density="comfortable"] {
  --table-cell-padding: 12px 16px;
  --row-height: 44px;
}
```

### Decisión 5 — Sistema de toasts (sonner)

Ver `01-error-model-fix.md` parte D. Detalles:
- Position: `bottom-right`.
- Theme: sincronizado con el tema activo.
- Rich colors: success/error/warning con bg coloreado.
- Action buttons inline.
- Stack: máx 4 toasts simultáneos, auto-dismiss 4s success / 8s error.

### Decisión 6 — Confirm dialogs estandarizados

Hoy las páginas tienen lógica de confirm ad hoc (restore-page tiene `confirmRestore` state). Estandarizar:

```tsx
// src/components/ui/confirm-dialog.tsx
const confirm = useConfirm();

const onClickDelete = async () => {
  const ok = await confirm({
    title: "Eliminar Microsoft Copilot",
    description: "Esto desinstalará el package Appx y aplicará las policies. Se creará un restore point antes.",
    consequences: [
      "El icono Copilot desaparece de la barra de tareas.",
      "El servicio de Copilot deja de arrancar.",
    ],
    confirmLabel: "Eliminar",
    cancelLabel: "Cancelar",
    danger: true,
  });
  if (ok) removeMutation.mutate(...);
};
```

Patrón promise-based, simple, evita el state local repetido.

### Decisión 7 — Empty states con icono + acción

Hoy `EmptyState` muestra icono + título + descripción. Falta la **acción recomendada**. v2:

```tsx
<EmptyState
  icon={FileText}
  title="Sin registros"
  description="No hay operaciones destructivas registradas."
  action={{
    label: "Crear restore point",
    onClick: () => navigate('/restore'),
  }}
/>
```

### Decisión 8 — Skeleton loaders consistentes

Hoy las páginas hacen `{isLoading ? "Cargando..." : ...}`. Reemplazar por skeleton rows que mimicen la estructura final.

```tsx
<TableSkeleton rows={8} columns={4} />
```

Patrón: durante el primer fetch, mostrar 8 rows con pulse animation; durante refetches, mantener data anterior + indicator sutil (header con spinner).

### Decisión 9 — Atajos de teclado documentados

| Atajo | Acción |
|-------|--------|
| `Ctrl+K` | Command palette |
| `Ctrl+/` | Modal con todos los atajos |
| `Ctrl+,` | Settings |
| `Ctrl+E` | Ir a Explorer |
| `Ctrl+B` | Ir a Debloat |
| `Ctrl+L` | Ir a Caché (Limpiar) |
| `Esc` | Cerrar modal/palette |
| `?` | Modal de ayuda |

Implementación: hook `useKeyboardShortcuts()` en `<AppShell>`.

### Decisión 10 — Modales para operaciones largas

Hoy una "limpieza" puede tardar 30+ segundos sin feedback. Modal que muestra:

```
┌──────────────────────────────────────────────┐
│ Limpiando caché del sistema                  │
│                                              │
│ ▰▰▰▰▰▰▰▰▱▱▱▱▱▱  62%                          │
│                                              │
│ 3.1 GB liberados · 4 bloqueados              │
│ Procesando: %TEMP%\Microsoft\Windows\...     │
│                                              │
│ ⚠ Spotify bloquea 900 MB                     │
│   [Cerrar Spotify y continuar]               │
│                                              │
│              [Pausar]   [Cancelar]           │
└──────────────────────────────────────────────┘
```

## Routing v2

```
/                           → Home
/explorer                   → Disk Explorer (lista)
/explorer/treemap           → Treemap mode
/cache                      → Cache Cleaner
/debloat                    → Debloat catalog
/debloat?selected=:id       → Debloat con item preseleccionado (deep link desde palette)
/services                   → Services
/registry                   → Registry Tweaks
/restore                    → Restore Points
/audit                      → Audit Log
/processes                  → Process Manager (NUEVO)
/startup                    → Startup Manager (NUEVO)
/disk                       → Disk Analyzer (NUEVO, treemap visual)
/privacy                    → Privacy Hardening (NUEVO)
/settings                   → Settings (subpestañas: appearance, safety, behavior, advanced)
/settings/about             → About
```

Estado URL-driven para filtros/selecciones donde tenga sentido (debloat, audit) para permitir compartir links.

## i18n

Stack: `react-i18next` + JSON locales en `src/locales/{es,en,pt}.json`.

Plan v1.0: extraer **al menos** español + inglés. Portugués (BR) lo añade comunidad post-launch.

Estructura:
```
src/locales/
├── es/
│   ├── common.json       (botones, labels generales)
│   ├── cache.json
│   ├── debloat.json
│   ├── restore.json
│   ├── audit.json
│   └── settings.json
└── en/ (mismo)
```

Detección automática:
```ts
i18n.init({
  fallbackLng: 'en',
  lng: settings.language === 'system'
    ? (navigator.language.startsWith('es') ? 'es' : 'en')
    : settings.language,
});
```

## Accesibilidad

Mínimos no-negociables para v1.0:
- Todas las páginas navegables con teclado.
- Tab order coherente.
- `aria-label` en iconos clicables sin texto.
- Modales con focus trap y restore al cerrar.
- Color contrast WCAG AA (4.5:1 texto normal, 3:1 grande).
- `prefers-reduced-motion` respetado en framer-motion.

## Lazy loading

Hoy `react.lazy` no está usado. Para reducir bundle size inicial:

```ts
const DebloatPage = lazy(() => import('./features/debloat/debloat-page'));
const ProcessesPage = lazy(() => import('./features/processes/processes-page'));
// ...
```

Cada route lazy-loaded → mejor first paint.

## Performance budget

Targets para v1.0:
- Bundle size inicial (gzipped): < 250KB
- First Contentful Paint: < 800ms (en hardware Win11 medio)
- Time to Interactive: < 1.5s
- Re-render en cambio de página: < 100ms

## Páginas afectadas por el refactor

Cada página tiene que adaptarse al nuevo sistema. Cambios típicos:
- Reemplazar `String(err)` por `formatError(err)`.
- Reemplazar inline error banners por toasts donde aplique.
- Adoptar `<TableSkeleton>` para loading.
- Migrar confirm ad hoc a `useConfirm()`.
- Añadir `<EmptyState action={{...}}>` donde haya empty meaningful.

Sin tocar lógica funcional. Sólo capa de presentación.

## Tests visuales

Recomendado (no obligatorio v1.0): Playwright con screenshot diffing en CI. Por ahora un smoke test que abre cada ruta y verifica que no hay error en consola.

## Criterio de "done"

- [ ] Sidebar colapsable funcional + persistencia.
- [ ] Command Palette con búsqueda fuzzy entre módulos + acciones.
- [ ] 3 themes funcionales (dark-cyan, dark-amber, light).
- [ ] Density toggle.
- [ ] Toasts integrados, no quedan errores inline.
- [ ] Confirm dialogs estandarizados, no más state local de "confirmDelete".
- [ ] EmptyStates con acción donde aplique.
- [ ] TableSkeletons consistentes.
- [ ] Atajos de teclado documentados con modal Ctrl+/.
- [ ] i18n con español + inglés.
- [ ] Lazy loading en rutas pesadas (debloat, processes).
- [ ] WCAG AA contrast en los 3 themes.
