# 04 — Debloat Catalog Expansion (M3, P1)

**Objetivo**: pasar de 11 entradas curadas a ~120, en 8 categorías, con detección multi-método y reverse recipes específicas.

**Spec de referencia**: `.claude/specs/v2/04-debloat-catalog-expansion.md`.

## Pasos en orden

1. [01 — Schema v2 + migration del JSON](01-schema-v2.md)
2. [02 — Detect multi-método (Appx + uninstaller + service + scheduled-task)](02-detect-multi-metodo.md)
3. [03 — Curación Tier 1 (50 entradas — MS first party + UI clutter)](03-tier1-curation.md)
4. [04 — Curación Tier 2 (40 entradas — OEM Lenovo/HP/Dell/Asus)](04-tier2-oem-curation.md)
5. [05 — Curación Tier 3+4 (30 entradas — third-party + avanzado)](05-tier3-4-curation.md)
6. [06 — Disclaimers UI + user catalog merge](06-disclaimers-y-user-merge.md)

## Notas

- Los pasos 03/04/05 son **curación humana**, no coding. Plan: 16-24h totales repartidas.
- Cada Tier puede correr en paralelo si tienes ayuda. Si trabajas solo, hacer Tier 1 primero (mayor impacto).
- Hacer en una VM Win 11 OEM real para validar detección.

## Criterio de done (área)

- [ ] Schema v2 validado en CI.
- [ ] `bloatware-catalog.json` con ≥120 entradas.
- [ ] Detect cubre 5 métodos: appx-user, appx-provisioned, uninstaller-string, service, scheduled-task.
- [ ] Cada entrada tiene `reverseRecipe` específica o `noop` con razón clara.
- [ ] Disclaimers UI para 5 packages destructivos (Edge, Store, OneDrive, Cortana, AV OEM).
- [ ] Smoke test en VM OEM: detecta ≥40 de las 120.
- [ ] `bloatware-catalog.user.json` se mergea correctamente con override.

## Tiempo total

20-25 horas (la mayor parte es curación).
