# 07 — Re-copy del PlanView para usuarios no técnicos

> **Severidad:** 🟡 P1 — el usuario pide que la app sea "para tontos".
> El copy actual usa términos técnicos que no comunican qué va a pasar.

## 1. Problema

Pantalla actual (según screenshot del usuario):

```
Plan de limpieza
12 listas · 9 bloqueadas · 0 permisos · 35 omitidas

[ Listas: 4.03 GB ]  [ Bloqueadas: 1.06 GB ]

▼ Listas para limpiar (12)            4.03 GB
▼ Bloqueadas por procesos (9)         1.06 GB
▼ Omitidas (35)                       0 B
```

Problemas:

- **"Listas"** suena a sustantivo plural (listas = arrays), no a
  adjetivo. Confunde.
- **"Bloqueadas por procesos"** no dice qué va a pasar con ellas.
- **"Omitidas"** se interpreta como un error ("se saltó algo").
- **"0 permisos"** no se entiende. ¿Cero permisos de qué?
- El usuario tuvo que preguntar qué significa cada cosa.

## 2. Causa raíz

`src/features/cache-cleaner/cache-page.tsx` — texts hardcoded
heredados de la implementación inicial cuando el target era admin
técnico, no usuario doméstico.

## 3. Fix propuesto

### 3.1 Tabla de re-copy

| Antes | Después | Justificación |
|---|---|---|
| **Plan de limpieza** | **Resumen** | Más corto, menos formal |
| `12 listas · 9 bloqueadas · 0 permisos · 35 omitidas` | `12 se pueden limpiar ahora · 9 al reiniciar · 35 ya vacías` | Acción + estado |
| **Listas: 4.03 GB** | **Se libera ahora: 4.03 GB** | Beneficio directo |
| **Bloqueadas: 1.06 GB** | **Se libera al reiniciar: 1.06 GB** | Acción concreta |
| Sección `Listas para limpiar (12)` | `Se limpian ahora (12)` con check verde ✓ | Verbo, sin ambigüedad |
| Sección `Bloqueadas por procesos (9)` | `Esperan al próximo reinicio (9)` con icono ↻ ámbar | Verbo, sin "bloqueado" (suena a error) |
| Sección `Omitidas (35)` | `Ya estaban vacías (35)` con icono ✓ gris | Explica el motivo |
| Sección `Permisos (0)` | sólo visible si > 0; entonces `Necesitan modo admin (N)` | Esconder cuando vacío |

### 3.2 Tooltips por sección

Cada sección debe explicar qué va a hacer:

- **"Se limpian ahora"**: tooltip *"ClearTool puede borrar estos archivos
  directamente sin reiniciar."*
- **"Esperan al próximo reinicio"**: tooltip *"Windows tiene estos
  archivos abiertos. ClearTool los marca para borrarlos automáticamente
  la próxima vez que reinicies el PC."*
- **"Ya estaban vacías"**: tooltip *"Estas ubicaciones no tienen nada
  que limpiar. Es normal — Windows las limpió solo o nunca acumularon
  caché."*
- **"Necesitan modo admin"**: tooltip *"Para limpiar estas ubicaciones,
  cierra ClearTool y vuelve a abrirlo como administrador (clic derecho
  → Ejecutar como administrador)."*

### 3.3 Cambiar la fila inferior de checks

Actualmente:

```
[ ] Cerrar procesos bloqueantes automáticamente
[✓] Programar bloqueados para reboot
[ ] Dry-run (simular)
```

Propuesto:

```
[ ] Avanzado: cerrar apps de usuario que estén bloqueando archivos
[✓] Limpiar al reiniciar lo que no se pueda ahora    ← marcado por defecto
[ ] Sólo simular (no borra nada, sólo te dice qué pasaría)
```

Notas:

- Etiquetar "Cerrar procesos" como **Avanzado** y desmarcado por defecto
  (ya lo está). Tooltip: *"Por seguridad, ClearTool nunca cierra el
  Explorador de Windows, la barra de tareas ni procesos del sistema —
  sólo navegadores, mensajería y reproductores de música."*
- "Limpiar al reiniciar" marcado por defecto: es lo que el usuario
  espera (que se libere TODO eventualmente).
- "Sólo simular" desambigua "dry-run".

### 3.4 Acción "Cancelar todos" del header amber (cuando hay reboot pendiente)

`"1 archivo programado para borrar en el próximo reboot"` ya es claro.
El botón "Cancelar todos" debería renombrarse a **"Quitar de la cola"**
porque "Cancelar" puede sonar a "deshacer la limpieza".

### 3.5 Renombrar el botón principal cuando dry-run está activo

Actual:

- Si dry-run: `Simular (4.03 GB)` ✓ bien
- Si no: `Limpiar (4.03 GB)` → cambiar a `Limpiar 4.03 GB` (sin
  paréntesis, suena más natural)

### 3.6 Header de página

Actual: `Plan de limpieza` con `← Volver`.
Propuesto: `Limpieza de caché` (verbo + sustantivo principal). El botón
`← Volver` puede quedarse, pero si entramos directos al PlanView (doc
02) ya no haría falta.

## 4. Criterio de done

- [ ] Un usuario que nunca ha visto ClearTool entiende, sin pulsar
      nada, qué pasa en cada sección.
- [ ] Ningún copy usa "bloqueado", "omitido", "permisos" en seco; siempre
      acción + razón.
- [ ] Los tooltips son visibles al pasar el ratón por la fila o el
      icono de info.
- [ ] La sección "Permisos" se esconde si su contador es 0.
- [ ] La opción "Cerrar procesos bloqueantes" lleva la palabra
      **Avanzado** y un tooltip explícito.

## 5. Riesgos / efectos secundarios

- Strings duros que tendrán que pasar por i18n cuando se haga la
  internacionalización (ES/EN). El cambio no es mayor pero conviene
  centralizarlos en `src/i18n/es.json` desde ya.
- Los snapshots de tests (si los hay) habrá que actualizarlos.
