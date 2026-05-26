# Paso 03 — VM snapshot plan (testing manual pre-release)

**Área**: 13-testing-ci
**Tiempo estimado**: 4-6 horas por ronda (manual)
**Dependencias**: una versión de ClearTool funcional + Hyper-V o VMware

## Qué hacemos

Plan formal de testing manual en VMs limpias antes de cada release. No es automatizable razonablemente, pero documentar el procedimiento garantiza que se hace siempre.

## Por qué

Los módulos destructivos (cache cleaner, debloat, registry tweaks) tocan el SO de formas que **no se pueden testear con mocks fielmente**. Una VM limpia con snapshot pre/post es la única manera honesta de validar.

## Setup

### VMs requeridas

| VM | Build Windows | Tipo |
|----|---------------|------|
| **VM-A** | Windows 11 22H2 OEM (ISO Lenovo o HP) | OEM bloatware test |
| **VM-B** | Windows 11 23H2 limpio (ISO MS oficial) | Default test |
| **VM-C** | Windows 11 24H2 limpio | Compat con nueva build |

Para cada VM:
- 8 GB RAM, 60 GB disco.
- Sin cuenta MS (cuenta local).
- Snapshot inicial llamado `clean-pristine`.

### Tools en la VM

- ClearTool instalador firmado (artifact del CI).
- Sysinternals Process Explorer (para verificar handle locks).
- Notepad++ (para inspeccionar audit.jsonl).
- WinDirStat (para comparar treemap visualmente).

## Los 5 escenarios obligatorios pre-release

### Escenario 1 — Smoke test instalación

1. Restaurar VM al snapshot `clean-pristine`.
2. Copiar `ClearTool-Setup-X.Y.Z-x64.exe` a la VM.
3. **Verificar SmartScreen NO bloquea** (no debe aparecer ventana azul "publisher unknown").
4. Instalar. Verificar:
   - License page muestra texto.
   - Idioma selector funciona.
   - Post-install ofrece restore point.
5. Lanzar ClearTool desde Inicio.
6. Verificar:
   - No errores en consola.
   - UAC prompt al abrir (admin requested).
   - UI carga, sidebar visible, todas las páginas accesibles.

**Pass criteria**: SmartScreen verde + UI funcional sin crash.

### Escenario 2 — Cache cleaner con apps abiertas (CRÍTICO)

1. Restaurar snapshot.
2. Instalar ClearTool.
3. Abrir Spotify (UWP), Chrome con 5 tabs, Discord.
4. Esperar 2 minutos a que generen caché.
5. ClearTool → Caché → Escanear todo.
6. Verificar plan:
   - Algunas locations en "Listas".
   - Spotify/Chrome/Discord en "Bloqueadas" con `lockedBy` correcto.
7. Click "Cerrar Spotify" → Spotify se cierra.
8. Re-scan automático.
9. Aplicar limpieza.
10. Tras completar, verify report.

**Pass criteria**: bytes_actually_freed > 100MB. No archivos críticos UWP (Terminal, Store) borrados.

### Escenario 3 — Debloat detección en OEM (CRÍTICO en VM-A)

1. Restaurar snapshot OEM (Lenovo).
2. Instalar ClearTool.
3. Debloat → Detectar instalados.
4. Verificar resultados:
   - Mínimo 40 entries detectadas.
   - OEM bloatware (Lenovo Vantage, McAfee LiveSafe) marcadas.
   - Microsoft Copilot, Solitaire, Office Hub marcados.
5. Aplicar preset "Recomendado".
6. Verificar:
   - Restore point creado.
   - Apps marcadas desaparecen del Inicio.
   - Audit log con entries por cada removal.
7. Test reversa: revert una operación específica desde Auditoría.
8. Verificar que la app reaparece (o queda documentado por qué no se puede revertir).

**Pass criteria**: 80%+ de packages seleccionados removidos correctamente, restore point intacto.

### Escenario 4 — Privacy preset Paranoid

1. Restaurar snapshot.
2. Instalar ClearTool.
3. Privacidad → seleccionar "Paranoid" → confirm disclaimer → Aplicar.
4. Esperar a que complete.
5. Reiniciar VM.
6. Verificar tras reboot:
   - Cortana no aparece.
   - Widgets desactivados.
   - Telemetría DiagTrack disabled (`Get-Service DiagTrack`).
   - Windows Update sigue funcionando (importante — no debe romperse).
7. Revertir desde Restore Point.
8. Verificar reversión completa.

**Pass criteria**: 90%+ de cambios aplicados, Windows Update intacto, restore point funciona.

### Escenario 5 — Upgrade flow (auto-updater)

1. Restaurar snapshot.
2. Instalar v0.9-rc1 (versión anterior).
3. Configurar a usar canal beta.
4. Settings → Comprobar actualizaciones.
5. Detecta v1.0.0.
6. Descargar + instalar.
7. Tras restart automático:
   - Versión = v1.0.0.
   - Settings preservados.
   - Audit log anterior preservado.

**Pass criteria**: upgrade sin pérdida de data, app funcional tras restart.

## Checklist por release

```markdown
## Release v1.X.Y — VM testing checklist

Date: ____________
Tester: ____________

### Escenarios
- [ ] Escenario 1 (Smoke install) — VM-B ___ pass / ___ fail
- [ ] Escenario 2 (Cache con apps) — VM-B ___ pass / ___ fail
- [ ] Escenario 3 (Debloat OEM) — VM-A ___ pass / ___ fail
- [ ] Escenario 4 (Privacy Paranoid) — VM-B ___ pass / ___ fail
- [ ] Escenario 5 (Upgrade) — VM-B ___ pass / ___ fail
- [ ] Repeat Escenarios 1+2 en VM-C (24H2) — ___ pass / ___ fail

### Issues observadas
1. _______________
2. _______________

### Decisión
- [ ] APROBADO para release público
- [ ] BLOQUEADO — fixes requeridos: _______________
```

Guardar checklist completado como `docs/v2/13-testing-ci/releases/v1.X.Y-tested.md`.

## Automatización futura (post-v1.0)

Para v1.1+:
- Packer scripts para generar VMs reproducibles.
- Vagrant boxes con Windows 11 (cumple con licencias evaluation).
- PowerShell scripts que automatizan parte de los escenarios.

No prioritario para v1.0 — el coste de tiempo de hacer manual 1-2 veces al mes es menor que el de mantener automatización compleja.

## Criterio de done

- [ ] 3 VMs preparadas con snapshots base.
- [ ] Checklist documentado para los 5 escenarios.
- [ ] Una ronda completa ejecutada para v0.5.0-rc o equivalente.
- [ ] Issues encontradas reportadas como GitHub issues.
- [ ] Procedimiento añadido al CONTRIBUTING como "Pre-release testing".
