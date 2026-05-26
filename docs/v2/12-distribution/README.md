# 12 — Distribución y Release (M5, P1)

**Objetivo**: que un usuario aleatorio pueda descargar ClearTool 1.0, instalarlo sin que SmartScreen lo bloquee, y recibir auto-updates firmadas.

**Spec de referencia**: `.claude/specs/v2/07-distribution.md`.

## Pasos en orden

1. [01 — Setup Azure Trusted Signing](01-azure-trusted-signing.md)
2. [02 — CI workflow con firma automática](02-ci-signing-workflow.md)
3. [03 — NSIS installer polish](03-nsis-polish.md)
4. [04 — Auto-updater + endpoint](04-auto-updater-y-endpoint.md)
5. [05 — winget submission](05-winget-submission.md)
6. [06 — Web propia + README + LICENSE](06-web-y-readme.md)

## Dependencias

- **Iniciar paso 01 al inicio de M3** (no esperar a M5). El proceso de identity validation de Azure tarda 1-3 días, mejor solapar.
- Resto de pasos pueden ir en M5 tras tener M2+M3+M4 estables.

## Criterio de done (área)

- [ ] Cert válido firmando binarios y MSI.
- [ ] CI build pipeline reproducible: tag `v1.0.0` → release firmado en GitHub Releases.
- [ ] Instalador NSIS testeado en 3 VMs (22H2, 23H2, 24H2).
- [ ] Auto-updater operativo, probado en upgrade `v0.9-rc → v1.0`.
- [ ] `winget install ClearTool.ClearTool` funciona.
- [ ] Dominio web up con landing + docs + changelog + endpoint updater.
- [ ] README serio en GitHub con screenshots, install, FAQ.
- [ ] LICENSE + PRIVACY-POLICY presentes.

## Coste total estimado

| Item | Coste anual |
|------|-------------|
| Azure Trusted Signing | $120 |
| Dominio | $15 |
| Hosting web | $0 (free tier Vercel/Cloudflare) |
| **Total** | **~$135/año** |

## Tiempo total

20-30 horas (mucho de esto es esperar — identity validation, DNS propagation, winget PR review).
