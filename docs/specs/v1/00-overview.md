# 00 — Overview

## Producto

**ClearTool** es una aplicación de escritorio nativa para Windows 11 enfocada en tres ejes:

1. **Explorar** el sistema de archivos en árbol con tamaños reales y filtros.
2. **Limpiar** cachés del sistema en ubicaciones documentadas y poco visibles.
3. **Debloat** agresivo: eliminar apps preinstaladas, parte del ecosistema Microsoft, telemetría y servicios; aplicar tweaks de registro que reducen ruido del SO.

## Por qué desktop y no web

La app necesita acceso directo a:

- Sistema de archivos (incluyendo paths >MAX_PATH y carpetas protegidas).
- Registro Win32 (HKLM, HKCU).
- WMI (System Restore, servicios).
- PowerShell con privilegios elevados.
- Procesos elevados (UAC).

Una app web no puede tocar nada de esto sin un agent local — y si vas a tener un agent local, ya tienes una app desktop. Tauri da el mejor balance: binario pequeño, lenguaje nativo (Rust) para la lógica peligrosa, UI moderna en web.

## Objetivos

1. Que un usuario avanzado pueda recuperar **GBs de espacio** y **eliminar bloatware** sin abrir cmd, PowerShell ni regedit.
2. Que **toda operación destructiva sea reversible** (restore point + log).
3. Que el código sea **auditable**: catálogos en JSON versionados, sin "magia" ni listas hardcodeadas en binarios.
4. Que la app sea **idempotente**: ejecutar la misma operación dos veces no rompe ni miente.

## No-objetivos (por ahora)

- Multilenguaje SO completo (la UI sí debe internacionalizar a ES/EN).
- Soporte Windows 10 — solo Windows 11 22H2 en adelante.
- Limpieza de archivos de usuario (documentos, fotos): nunca, eso lo hace el usuario.
- Sincronización cloud, cuentas, telemetría propia: cero.
- Antivirus, antimalware: queda fuera de scope.

## Audiencia

- Usuarios técnicos / power users de Windows 11.
- Profesionales que reciben PCs con bloatware OEM.
- Personas que quieren un Windows 11 "delgado" sin el ruido de Microsoft.

## Métricas de éxito

- Tiempo desde primera ejecución hasta primer GB liberado: < 5 minutos.
- Tasa de operaciones que dejan el sistema en estado inconsistente: 0%.
- % de operaciones destructivas con reversa documentada y probada: 100%.
- Cobertura de tests para módulos destructivos: > 90%.

## Stack resumido

- **Backend:** Rust 1.78+, Tauri 2.x, windows-rs 0.58, winreg, tokio.
- **Frontend:** React 18, TypeScript 5.4+, Vite, Tailwind 3, shadcn/ui, Zustand, React Query.
- **Empaquetado:** Tauri bundler (MSI + NSIS).
- **Tests:** cargo test, vitest, Playwright + Tauri WebDriver.

## Stakeholders y agentes

- Usuario final: Jowy (autor del proyecto).
- Subagentes que orquestan el desarrollo: ver `.claude/agents/README.md`.
- Skills de soporte: ver `.claude/skills/README.md`.
