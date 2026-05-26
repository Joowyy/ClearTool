# 12 — Internacionalización (ES/EN) + Onboarding + Accesibilidad

> **Posición:** 12/14.
> **Dependencias:** [07-SETTINGS](07-SETTINGS.md) (idioma persiste).
> **Output:** i18next con ES/EN completos, first-run wizard de bienvenida, mejoras de accesibilidad WCAG AA mínima.

---

## 1. Resumen ejecutivo

ClearTool nace en español (mercado primario). Para v1.0 público se añade inglés para alcanzar comunidad global.

Estado actual:

| Item | Estado |
|---|---|
| Strings hardcoded en español | ✅ Existen, pero scattered en componentes |
| i18n framework | ❌ No instalado |
| First-run wizard | ❌ Inexistente |
| Accesibilidad: focus rings, aria-labels, keyboard nav | 🟡 Parcial (shadcn/ui ayuda) |
| Skip-to-content link | ❌ |
| Contraste WCAG AA verificado | 🟡 No medido |

Output del cierre:
- `i18next` + `react-i18next` configurados.
- 2 archivos de traducciones (`es.json`, `en.json`) con TODOS los strings extraídos.
- Onboarding wizard de 4 pasos (Bienvenida, Modelo de seguridad, Modo limitado, Listo).
- Auditoría a11y básica + fixes.

---

## 2. Diagnóstico

### 2.1 Inventario de strings hardcoded

Aproximadamente **~350 strings** de UI distribuidos en:

- `home-page.tsx` y subcomponents — ~40 strings.
- `cache-page.tsx` — ~25.
- `debloat-page.tsx` — ~30.
- `registry-page.tsx` — ~20.
- `services-page.tsx` — ~20.
- `restore-page.tsx` — ~15.
- `audit-page.tsx` — ~20.
- `settings-page.tsx` + tabs — ~50.
- `explorer-page.tsx` (post rediseño) — ~30.
- Modales (Confirm, Detail, Progress, Disclaimer) — ~60.
- Componentes compartidos (EmptyState, RiskBadge, etc.) — ~20.
- Navegación + layout — ~20.

### 2.2 i18n: framework

| Opción | Pros | Contras |
|---|---|---|
| `react-i18next` | Estándar, namespaces, lazy load | Setup verboso |
| `react-intl` (FormatJS) | ICU MessageFormat, mejor pluralización | Más grande |
| `next-intl` | Diseño moderno | Acoplado a Next |
| Custom (objeto con keys) | Cero deps | Sin features de pluralización, fechas, etc. |

**Decisión:** `react-i18next`. Es el estándar de facto, suficientemente liviano, soporta pluralización, namespaces.

### 2.3 Onboarding

Mostrado solo al primer arranque (cuando `settings.behavior.lastSeenAuditRunId === null` y no hay audit entries). El usuario puede dispararlo manualmente desde Settings → Acerca de → "Ver onboarding de nuevo".

### 2.4 Accesibilidad mínima

WCAG AA es realista para v1.0:

- Contraste 4.5:1 en texto normal.
- Focus visible (ring) en todos los interactivos.
- Aria-label en botones que solo tienen icono.
- `prefers-reduced-motion` respetado por framer-motion.
- Navegación por teclado completa (Tab + Enter + flechas en listas).

---

## 3. Decisiones arquitectónicas

| Decisión | Justificación |
|---|---|
| `react-i18next` con `i18next-browser-languagedetector` | Detecta idioma del SO automáticamente |
| Archivos `es.json` y `en.json` con namespaces por feature | Tree-shakeable, mantenible |
| Idioma persiste en `settings.appearance.language` | Single source of truth |
| `system` = detect del SO al startup, sino fallback `es` | Coherencia con tema |
| Onboarding usa el componente `Dialog` de shadcn/ui | Reusa look-and-feel |
| Toda traducción se extrae con `i18n.t("key")` o `<Trans>` | No literals en JSX |
| Strings vacíos como key — error en CI vía lint custom | Coverage de traducción enforzable |

---

## 4. Setup i18next

### 4.1 Dependencias

```bash
npm install i18next react-i18next i18next-browser-languagedetector
```

### 4.2 Configuración

**Archivo:** `src/i18n/index.ts` (nuevo)

```ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import es from "./locales/es.json";
import en from "./locales/en.json";

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: "es",
    supportedLngs: ["es", "en"],
    debug: false,
    resources: {
      es: { translation: es },
      en: { translation: en },
    },
    interpolation: {
      escapeValue: false, // React ya escapa
    },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "ct.language",
      caches: ["localStorage"],
    },
  });

export default i18n;

export function applyLanguageFromSettings(lang: "es" | "en" | "system") {
  if (lang === "system") {
    // i18next-browser-languagedetector ya detectó del navigator
    return;
  }
  void i18n.changeLanguage(lang);
  localStorage.setItem("ct.language", lang);
}
```

### 4.3 Bootstrap en `main.tsx`

```tsx
import "./i18n";
// ... resto del bootstrap
```

### 4.4 Sincronización con Settings

En `app.tsx` (o componente top-level):

```tsx
import { useEffect } from "react";
import { useSettings } from "./features/settings/use-settings";
import { applyLanguageFromSettings } from "./i18n";

function App() {
  const { data: settings } = useSettings();

  useEffect(() => {
    if (settings) {
      applyLanguageFromSettings(settings.appearance.language);
    }
  }, [settings?.appearance.language]);

  return /* ... */;
}
```

---

## 5. Estructura de keys

### 5.1 `src/i18n/locales/es.json`

```json
{
  "app": {
    "name": "ClearTool",
    "tagline": "Limpieza y optimización para Windows 11"
  },
  "nav": {
    "home": "Inicio",
    "explorer": "Explorador",
    "cache": "Caché",
    "debloat": "Debloat",
    "registry": "Registro",
    "services": "Servicios",
    "restorePoints": "Puntos de restauración",
    "audit": "Auditoría",
    "settings": "Ajustes"
  },
  "common": {
    "cancel": "Cancelar",
    "apply": "Aplicar",
    "close": "Cerrar",
    "save": "Guardar",
    "delete": "Eliminar",
    "refresh": "Refrescar",
    "loading": "Cargando...",
    "noResults": "Sin resultados",
    "dryRun": "Dry-run",
    "confirm": "Confirmar",
    "showDetails": "Ver detalles",
    "hideDetails": "Ocultar detalles",
    "revert": "Revertir",
    "categoryLabel": "Categoría",
    "riskLabel": "Riesgo",
    "stateLabel": "Estado",
    "actionLabel": "Acción"
  },
  "risk": {
    "low": "Bajo",
    "medium": "Medio",
    "high": "Alto"
  },
  "elevation": {
    "limitedMode": "Modo limitado — algunas operaciones requieren admin.",
    "restartAsAdmin": "Reiniciar como administrador"
  },
  "dryRunBanner": {
    "text": "Modo Dry-Run activo — ninguna operación toca el sistema realmente.",
    "deactivateLink": "Desactivar"
  },
  "home": {
    "loadGlobal": "Carga global",
    "cpu": "CPU",
    "ram": "RAM",
    "gpu": "GPU",
    "disk": "Discos",
    "topProcesses": "Procesos top"
  },
  "explorer": {
    "title": "Explorador",
    "selectDrive": "Seleccionar disco",
    "scanPath": "Escanear ruta",
    "rescan": "Re-escanear",
    "cancelScan": "Cancelar escaneo",
    "scanning": "Escaneando...",
    "drivesDetected": "Discos detectados",
    "diskUsed": "{{percent}}% usado",
    "diskFree": "{{free}} libres",
    "totalSize": "{{total}} totales",
    "confirmScanTitle": "Analizar {{path}}",
    "confirmScanIntro": "El escaneo es de solo lectura. No modifica nada en disco.",
    "includeHidden": "Incluir archivos ocultos",
    "followReparse": "Seguir junctions / symlinks (riesgo de loop)"
  },
  "cache": {
    "title": "Limpieza de caché",
    "summary": "{{locations}} ubicaciones · {{detected}} detectados · {{freeable}} liberables",
    "presetSystem": "Sistema",
    "presetUser": "Usuario",
    "presetBrowsers": "Navegadores",
    "presetPackageMgrs": "Package managers",
    "presetAll": "Todo",
    "presetNone": "Nada",
    "rescanSelected": "Re-escanear seleccionadas",
    "cleanButton": "Limpiar ({{freeable}})",
    "createRestorePoint": "Crear punto de restauración antes",
    "forceCloseProcesses": "Forzar cerrar procesos que bloquean",
    "confirmTitle": "Limpiar caché",
    "confirmIntroBytes": "Vas a liberar {{freeable}} de {{count}} ubicaciones.",
    "irreversibleNote": "Esta operación no es completamente reversible. Archivos en uso se marcarán para borrado en el próximo reboot."
  },
  "debloat": {
    "title": "Debloat",
    "summary": "{{catalog}} apps en catálogo · {{detected}} detectadas · {{selected}} seleccionadas",
    "presetMinimal": "Mínimo",
    "presetRecommended": "Recomendado",
    "presetTotal": "Total",
    "cleanProvisional": "Limpiar también la versión provisionada (admin)",
    "applyPolicies": "Aplicar políticas para evitar reinstall",
    "disableServices": "Detener servicios asociados",
    "removeSelected": "Eliminar selección ({{count}})",
    "disclaimerHeader": "Apps de alto riesgo seleccionadas",
    "disclaimerIntro": "Las siguientes apps tienen consecuencias importantes. Debés aceptar cada una antes de continuar.",
    "disclaimerAccept": "Entiendo y acepto eliminar {{name}}"
  },
  "registry": {
    "title": "Tweaks de Registro",
    "summary": "{{count}} tweaks · {{selected}} seleccionados",
    "operationCount": "{{count}} operación",
    "operationCount_plural": "{{count}} operaciones",
    "currentValue": "Estado actual",
    "applyValue": "Aplicar",
    "revertValue": "Revertir"
  },
  "services": {
    "title": "Servicios de Windows",
    "summary": "{{total}} servicios · {{catalog}} en catálogo · {{tunable}} tuneables",
    "onlyCatalog": "Solo catálogo",
    "stopNow": "Detener ahora si está corriendo",
    "applyPresetTitle": "Aplicar preset \"{{preset}}\"",
    "dependenciesHeader": "Dependencias",
    "dependsOn": "Depende de:",
    "neededBy": "Lo usan:"
  },
  "restore": {
    "title": "Puntos de restauración",
    "summary": "{{count}} puntos encontrados",
    "createPoint": "Crear punto",
    "protectionOff": "System Protection está DESACTIVADO para C:\\",
    "protectionOffNote": "Sin esta protección, ClearTool NO puede crear puntos antes de tocar el sistema.",
    "activate": "Activar protección para C:\\",
    "restoreToTitle": "Restaurar el sistema",
    "restoreWarning1": "Reinicia el sistema.",
    "restoreWarning2": "Revierte cambios de registro, drivers, apps.",
    "restoreWarning3": "NO afecta documentos del usuario.",
    "restoreWarning4": "Tarda 5-15 minutos.",
    "restoreWarning5": "Es IRREVERSIBLE una vez iniciada.",
    "continueAndRestart": "Continuar y reiniciar"
  },
  "audit": {
    "title": "Log de auditoría",
    "summary": "{{count}} entradas. Cada operación destructiva queda registrada con opción de reversa.",
    "empty": "Sin operaciones registradas",
    "emptyDescription": "Aquí aparecerán todas las operaciones destructivas con opción de revertirlas.",
    "revertTitle": "Revertir operación",
    "revertWarning": "La reversa se loguea como nueva entrada y no se puede deshacer fácilmente.",
    "exportLog": "Exportar log"
  },
  "settings": {
    "title": "Ajustes",
    "tabs": {
      "appearance": "Apariencia",
      "safety": "Seguridad",
      "behavior": "Comportamiento",
      "advanced": "Avanzado",
      "about": "Acerca de"
    },
    "appearance": {
      "theme": "Tema visual",
      "themeDark": "Oscuro",
      "themeLight": "Claro",
      "themeSystem": "Seguir sistema",
      "language": "Idioma",
      "langEs": "Español",
      "langEn": "English",
      "langSystem": "Seguir sistema",
      "density": "Densidad de UI",
      "densityCompact": "Compacta",
      "densityNormal": "Normal",
      "densityComfortable": "Cómoda"
    },
    "safety": {
      "dryRunGlobal": "Modo Dry-Run global",
      "dryRunGlobalDesc": "Fuerza todos los módulos destructivos a simular sin tocar nada.",
      "autoRestore": "Crear restore point automáticamente",
      "requireConfirm": "Pedir confirmación antes de aplicar un batch",
      "bypassThrottle": "Bypass del throttling 24h de System Restore"
    },
    "behavior": {
      "checkUpdates": "Buscar actualizaciones al arrancar",
      "checkUpdatesNote": "Sin datos personales enviados.",
      "rememberWindow": "Recordar tamaño y posición de la ventana",
      "auditPath": "Path",
      "auditSize": "Tamaño",
      "auditEntries": "Entradas",
      "openAudit": "Abrir audit log",
      "viewAudit": "Ver pantalla audit",
      "exportLog": "Exportar log"
    },
    "advanced": {
      "logLevel": "Nivel de log operativo",
      "auditMaxMb": "Tamaño máximo del audit antes de rotar (MB)",
      "diagnosticMode": "Modo diagnóstico",
      "diagnosticModeNote": "Habilita botones técnicos.",
      "updateChannel": "Canal de actualizaciones",
      "channelStable": "Estable",
      "channelBeta": "Beta (acceso temprano)",
      "resetDefaults": "Restaurar valores por defecto"
    },
    "about": {
      "version": "Versión",
      "build": "Build",
      "stack": "Stack",
      "license": "Licencia",
      "resources": "Recursos",
      "repo": "Repositorio en GitHub",
      "issues": "Reporte de issues",
      "docs": "Documentación del proyecto",
      "credits": "Créditos: comunidad open-source.",
      "tagline": "Hecho con cuidado, sin telemetría, sin tracking, sin anuncios.",
      "rerunOnboarding": "Ver onboarding de nuevo"
    }
  },
  "onboarding": {
    "step1Title": "Bienvenido a ClearTool",
    "step1Body": "Esta herramienta limpia, optimiza y libera espacio en Windows 11. Lo hace de forma transparente: vas a ver exactamente qué se modifica antes de aplicarlo.",
    "step2Title": "Modelo de seguridad",
    "step2Body": "Antes de cualquier operación destructiva, ClearTool crea un punto de restauración del sistema y registra la operación en un log de auditoría con instrucciones de reversa.",
    "step3Title": "Modo limitado",
    "step3Body": "Para escanear, ClearTool funciona como usuario normal. Para modificar el registro, servicios o eliminar bloatware, necesita admin. Si cancelás UAC, la app queda en \"modo limitado\" (solo lectura).",
    "step4Title": "Listo",
    "step4Body": "Empezá por el Dashboard para ver el estado de tu PC, o por Caché si querés liberar espacio rápido. Si dudás, activá \"Modo Dry-Run global\" en Ajustes → Seguridad — todo simula sin tocar nada.",
    "next": "Siguiente",
    "skip": "Saltar",
    "finish": "Empezar"
  },
  "updates": {
    "available": "Actualización disponible",
    "currentVersion": "ClearTool {{latest}} (tenés {{current}}).",
    "installAndRestart": "Instalar y reiniciar",
    "later": "Más tarde"
  }
}
```

### 5.2 `src/i18n/locales/en.json`

```json
{
  "app": {
    "name": "ClearTool",
    "tagline": "Cleanup and optimization for Windows 11"
  },
  "nav": {
    "home": "Home",
    "explorer": "Explorer",
    "cache": "Cache",
    "debloat": "Debloat",
    "registry": "Registry",
    "services": "Services",
    "restorePoints": "Restore points",
    "audit": "Audit",
    "settings": "Settings"
  },
  "common": {
    "cancel": "Cancel",
    "apply": "Apply",
    "close": "Close",
    "save": "Save",
    "delete": "Delete",
    "refresh": "Refresh",
    "loading": "Loading...",
    "noResults": "No results",
    "dryRun": "Dry-run",
    "confirm": "Confirm",
    "showDetails": "Show details",
    "hideDetails": "Hide details",
    "revert": "Revert",
    "categoryLabel": "Category",
    "riskLabel": "Risk",
    "stateLabel": "State",
    "actionLabel": "Action"
  },
  "risk": {
    "low": "Low",
    "medium": "Medium",
    "high": "High"
  },
  "elevation": {
    "limitedMode": "Limited mode — some operations require admin.",
    "restartAsAdmin": "Restart as administrator"
  },
  "dryRunBanner": {
    "text": "Dry-Run mode active — no operation actually touches the system.",
    "deactivateLink": "Deactivate"
  },
  "home": {
    "loadGlobal": "Global load",
    "cpu": "CPU",
    "ram": "RAM",
    "gpu": "GPU",
    "disk": "Disks",
    "topProcesses": "Top processes"
  },
  "explorer": {
    "title": "Explorer",
    "selectDrive": "Select drive",
    "scanPath": "Scan path",
    "rescan": "Rescan",
    "cancelScan": "Cancel scan",
    "scanning": "Scanning...",
    "drivesDetected": "Detected drives",
    "diskUsed": "{{percent}}% used",
    "diskFree": "{{free}} free",
    "totalSize": "{{total}} total",
    "confirmScanTitle": "Scan {{path}}",
    "confirmScanIntro": "Scanning is read-only. Nothing on disk is modified.",
    "includeHidden": "Include hidden files",
    "followReparse": "Follow junctions / symlinks (loop risk)"
  },
  "cache": {
    "title": "Cache cleanup",
    "summary": "{{locations}} locations · {{detected}} detected · {{freeable}} freeable",
    "presetSystem": "System",
    "presetUser": "User",
    "presetBrowsers": "Browsers",
    "presetPackageMgrs": "Package managers",
    "presetAll": "All",
    "presetNone": "None",
    "rescanSelected": "Rescan selected",
    "cleanButton": "Clean ({{freeable}})",
    "createRestorePoint": "Create restore point first",
    "forceCloseProcesses": "Force-close blocking processes",
    "confirmTitle": "Clean cache",
    "confirmIntroBytes": "You will free {{freeable}} from {{count}} locations.",
    "irreversibleNote": "This operation isn't fully reversible. Files in use will be marked for deletion on next reboot."
  },
  "debloat": {
    "title": "Debloat",
    "summary": "{{catalog}} apps in catalog · {{detected}} detected · {{selected}} selected",
    "presetMinimal": "Minimal",
    "presetRecommended": "Recommended",
    "presetTotal": "Total",
    "cleanProvisional": "Also clean provisional version (admin)",
    "applyPolicies": "Apply anti-reinstall policies",
    "disableServices": "Stop associated services",
    "removeSelected": "Remove selection ({{count}})",
    "disclaimerHeader": "High-risk apps selected",
    "disclaimerIntro": "The following apps have significant consequences. You must accept each one before continuing.",
    "disclaimerAccept": "I understand and accept removing {{name}}"
  },
  "registry": {
    "title": "Registry tweaks",
    "summary": "{{count}} tweaks · {{selected}} selected",
    "operationCount": "{{count}} operation",
    "operationCount_plural": "{{count}} operations",
    "currentValue": "Current value",
    "applyValue": "Apply",
    "revertValue": "Revert"
  },
  "services": {
    "title": "Windows services",
    "summary": "{{total}} services · {{catalog}} in catalog · {{tunable}} tunable",
    "onlyCatalog": "Catalog only",
    "stopNow": "Stop now if running",
    "applyPresetTitle": "Apply preset \"{{preset}}\"",
    "dependenciesHeader": "Dependencies",
    "dependsOn": "Depends on:",
    "neededBy": "Used by:"
  },
  "restore": {
    "title": "Restore points",
    "summary": "{{count}} points found",
    "createPoint": "Create point",
    "protectionOff": "System Protection is DISABLED for C:\\",
    "protectionOffNote": "Without this protection, ClearTool CANNOT create points before touching the system.",
    "activate": "Enable protection for C:\\",
    "restoreToTitle": "Restore the system",
    "restoreWarning1": "Restarts the system.",
    "restoreWarning2": "Reverts registry, driver and app changes.",
    "restoreWarning3": "DOES NOT affect user documents.",
    "restoreWarning4": "Takes 5-15 minutes.",
    "restoreWarning5": "It is IRREVERSIBLE once started.",
    "continueAndRestart": "Continue and restart"
  },
  "audit": {
    "title": "Audit log",
    "summary": "{{count}} entries. Each destructive operation is logged with a reverse option.",
    "empty": "No operations logged",
    "emptyDescription": "Here you'll see every destructive operation with an option to revert it.",
    "revertTitle": "Revert operation",
    "revertWarning": "The revert is logged as a new entry and can't be easily undone.",
    "exportLog": "Export log"
  },
  "settings": {
    "title": "Settings",
    "tabs": {
      "appearance": "Appearance",
      "safety": "Safety",
      "behavior": "Behavior",
      "advanced": "Advanced",
      "about": "About"
    },
    "appearance": {
      "theme": "Visual theme",
      "themeDark": "Dark",
      "themeLight": "Light",
      "themeSystem": "Follow system",
      "language": "Language",
      "langEs": "Español",
      "langEn": "English",
      "langSystem": "Follow system",
      "density": "UI density",
      "densityCompact": "Compact",
      "densityNormal": "Normal",
      "densityComfortable": "Comfortable"
    },
    "safety": {
      "dryRunGlobal": "Global Dry-Run mode",
      "dryRunGlobalDesc": "Forces all destructive modules to simulate without touching anything.",
      "autoRestore": "Auto-create restore point",
      "requireConfirm": "Require confirmation before batch",
      "bypassThrottle": "Bypass 24h System Restore throttling"
    },
    "behavior": {
      "checkUpdates": "Check for updates at startup",
      "checkUpdatesNote": "No personal data sent.",
      "rememberWindow": "Remember window size and position",
      "auditPath": "Path",
      "auditSize": "Size",
      "auditEntries": "Entries",
      "openAudit": "Open audit log",
      "viewAudit": "View audit page",
      "exportLog": "Export log"
    },
    "advanced": {
      "logLevel": "Operational log level",
      "auditMaxMb": "Max audit size before rotation (MB)",
      "diagnosticMode": "Diagnostic mode",
      "diagnosticModeNote": "Enables technical buttons.",
      "updateChannel": "Update channel",
      "channelStable": "Stable",
      "channelBeta": "Beta (early access)",
      "resetDefaults": "Reset to defaults"
    },
    "about": {
      "version": "Version",
      "build": "Build",
      "stack": "Stack",
      "license": "License",
      "resources": "Resources",
      "repo": "GitHub repository",
      "issues": "Issue tracker",
      "docs": "Project documentation",
      "credits": "Credits: open-source community.",
      "tagline": "Built with care, no telemetry, no tracking, no ads.",
      "rerunOnboarding": "Show onboarding again"
    }
  },
  "onboarding": {
    "step1Title": "Welcome to ClearTool",
    "step1Body": "This tool cleans, optimizes and frees space on Windows 11. It does so transparently: you'll see exactly what gets modified before applying.",
    "step2Title": "Safety model",
    "step2Body": "Before any destructive operation, ClearTool creates a system restore point and logs the operation with reverse instructions.",
    "step3Title": "Limited mode",
    "step3Body": "ClearTool scans as a regular user. To modify the registry, services, or remove bloatware, it needs admin. If you cancel UAC, the app enters \"limited mode\" (read-only).",
    "step4Title": "Ready",
    "step4Body": "Start with the Dashboard to see your PC's status, or with Cache if you want to free space fast. If unsure, enable \"Global Dry-Run\" in Settings → Safety — everything simulates without touching anything.",
    "next": "Next",
    "skip": "Skip",
    "finish": "Get started"
  },
  "updates": {
    "available": "Update available",
    "currentVersion": "ClearTool {{latest}} (you have {{current}}).",
    "installAndRestart": "Install and restart",
    "later": "Later"
  }
}
```

---

## 6. Refactor de componentes a t()

### 6.1 Patrón general

Antes:

```tsx
<h2 className="text-2xl font-bold">Servicios de Windows</h2>
<p className="text-muted-foreground text-sm">
  {services.length} servicios encontrados
</p>
```

Después:

```tsx
import { useTranslation } from "react-i18next";

const { t } = useTranslation();

<h2 className="text-2xl font-bold">{t("services.title")}</h2>
<p className="text-muted-foreground text-sm">
  {t("services.summary", { total: services.length, catalog: 0, tunable: 0 })}
</p>
```

### 6.2 Componente RiskBadge i18n-aware

```tsx
import { useTranslation } from "react-i18next";

export function RiskBadge({ risk }: { risk: string }) {
  const { t } = useTranslation();
  const key = risk.toLowerCase() as "low" | "medium" | "high";
  const variant = key === "high" ? "destructive" : key === "medium" ? "warning" : "success";
  return <Badge variant={variant}>{t(`risk.${key}`)}</Badge>;
}
```

### 6.3 Linter custom (opcional, alta-fricción)

Regla ESLint que prohíbe strings literales en JSX excepto en archivos `*.test.tsx` y `*.stories.tsx`. Para v1.0, optar por **revisión manual** + `grep` checks como gate:

```bash
# scripts/check-i18n.ps1
$violations = Select-String -Path "src\**\*.tsx" -Pattern ">\s*[a-zA-ZáéíóúñÑ]{5,}\s*<" |
  Where-Object { $_.Line -notmatch 'i18n|t\(|Trans|className|data-' }
if ($violations) {
    Write-Warning "Posibles strings sin traducir:"
    $violations | Select-Object -First 20 | ForEach-Object { Write-Host $_.Line }
}
```

---

## 7. Onboarding wizard

### 7.1 Componente

**Archivo:** `src/features/onboarding/onboarding-wizard.tsx` (nuevo)

```tsx
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../components/ui/button";

interface Step {
  titleKey: string;
  bodyKey: string;
}

const STEPS: Step[] = [
  { titleKey: "onboarding.step1Title", bodyKey: "onboarding.step1Body" },
  { titleKey: "onboarding.step2Title", bodyKey: "onboarding.step2Body" },
  { titleKey: "onboarding.step3Title", bodyKey: "onboarding.step3Body" },
  { titleKey: "onboarding.step4Title", bodyKey: "onboarding.step4Body" },
];

export function OnboardingWizard(props: { onClose: () => void }) {
  const { t } = useTranslation();
  const [idx, setIdx] = useState(0);
  const step = STEPS[idx];
  const isLast = idx === STEPS.length - 1;

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
      <div className="bg-card border border-border rounded-lg max-w-lg w-full p-6">
        <div className="flex gap-1 mb-4">
          {STEPS.map((_, i) => (
            <div
              key={i}
              className={`h-1 flex-1 rounded ${i <= idx ? "bg-primary" : "bg-border"}`}
            />
          ))}
        </div>

        <h2 className="text-2xl font-bold mb-3">{t(step.titleKey)}</h2>
        <p className="text-muted-foreground leading-relaxed">{t(step.bodyKey)}</p>

        <div className="flex justify-between items-center mt-6">
          <button
            type="button"
            className="text-sm text-muted-foreground hover:text-foreground"
            onClick={props.onClose}
          >
            {t("onboarding.skip")}
          </button>
          <div className="flex gap-2">
            {idx > 0 && (
              <Button variant="ghost" onClick={() => setIdx((i) => i - 1)}>
                ←
              </Button>
            )}
            <Button onClick={() => (isLast ? props.onClose() : setIdx((i) => i + 1))}>
              {isLast ? t("onboarding.finish") : t("onboarding.next")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
```

### 7.2 Trigger en `app.tsx`

```tsx
import { useEffect, useState } from "react";
import { OnboardingWizard } from "./features/onboarding/onboarding-wizard";
import { useSettings, useUpdateSettings } from "./features/settings/use-settings";

function App() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const [showOnboarding, setShowOnboarding] = useState(false);

  useEffect(() => {
    if (settings && settings.behavior.lastSeenAuditRunId === null && !localStorage.getItem("ct.onboarding-seen")) {
      setShowOnboarding(true);
    }
  }, [settings]);

  const closeOnboarding = () => {
    localStorage.setItem("ct.onboarding-seen", "1");
    setShowOnboarding(false);
  };

  return (
    <>
      {/* resto de la app */}
      {showOnboarding && <OnboardingWizard onClose={closeOnboarding} />}
    </>
  );
}
```

### 7.3 Botón "Ver onboarding de nuevo" en Settings

En el tab "Acerca de":

```tsx
<Button
  variant="outline"
  size="sm"
  onClick={() => {
    localStorage.removeItem("ct.onboarding-seen");
    location.reload();
  }}
>
  {t("settings.about.rerunOnboarding")}
</Button>
```

---

## 8. Accesibilidad

### 8.1 Focus rings

`tailwind.config.js`:

```js
extend: {
  ringColor: {
    DEFAULT: "rgb(var(--ring))",
  },
},
```

`src/styles/globals.css`:

```css
button, [role="button"], a, input, select, textarea {
  outline: none;
}
button:focus-visible, [role="button"]:focus-visible, a:focus-visible {
  outline: 2px solid hsl(var(--ring));
  outline-offset: 2px;
}
```

### 8.2 Skip-to-content

En `app-shell.tsx`, al inicio del JSX:

```tsx
<a
  href="#main-content"
  className="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:bg-primary focus:text-primary-foreground focus:px-3 focus:py-1 focus:rounded z-50"
>
  Saltar al contenido
</a>
{/* ... layout ... */}
<main id="main-content">{children}</main>
```

### 8.3 Aria-labels para botones con icono

```tsx
<Button size="icon" aria-label={t("common.refresh")}>
  <RefreshCw className="h-4 w-4" />
</Button>
```

### 8.4 Respeto a `prefers-reduced-motion`

Framer-motion lo respeta nativamente si se setea `MotionConfig`:

```tsx
import { MotionConfig } from "framer-motion";

<MotionConfig reducedMotion="user">
  {/* app */}
</MotionConfig>
```

### 8.5 Contraste

Verificar pares de color críticos en `globals.css` contra WCAG AA:

- Texto sobre fondo: ≥ 4.5:1.
- Texto grande (≥ 18pt o 14pt bold): ≥ 3:1.

Tool: https://webaim.org/resources/contrastchecker/. Verificar 5 combos clave (background/foreground, card/foreground, primary/primary-foreground, destructive/destructive-foreground, muted/muted-foreground).

### 8.6 Keyboard navigation tests

| Caso | Pasa si |
|---|---|
| Tab desde top | recorre sidebar → header → main → footer |
| Enter en botón | activa la acción |
| Esc en modal | cierra modal |
| Arrow keys en tabla virtualizada | mueve foco entre filas |
| Shift+Tab | navega hacia atrás |

---

## 9. Tests

### 9.1 Test de coverage i18n

```ts
// src/i18n/__tests__/coverage.test.ts
import { describe, it, expect } from "vitest";
import es from "../locales/es.json";
import en from "../locales/en.json";

function flatten(obj: any, prefix = ""): string[] {
  const out: string[] = [];
  for (const [k, v] of Object.entries(obj)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (typeof v === "object") out.push(...flatten(v, key));
    else out.push(key);
  }
  return out;
}

describe("i18n coverage", () => {
  it("es y en tienen las mismas keys", () => {
    const esKeys = new Set(flatten(es));
    const enKeys = new Set(flatten(en));
    const onlyEs = [...esKeys].filter((k) => !enKeys.has(k));
    const onlyEn = [...enKeys].filter((k) => !esKeys.has(k));
    expect(onlyEs, "keys en ES sin traducción EN").toEqual([]);
    expect(onlyEn, "keys en EN sin traducción ES").toEqual([]);
  });

  it("ningún valor está vacío", () => {
    const flat = (o: any, p = ""): Array<[string, any]> => {
      const out: Array<[string, any]> = [];
      for (const [k, v] of Object.entries(o)) {
        const key = p ? `${p}.${k}` : k;
        if (typeof v === "object") out.push(...flat(v, key));
        else out.push([key, v]);
      }
      return out;
    };
    for (const [k, v] of flat(es)) {
      expect(typeof v === "string" && v.length > 0, `ES key vacía: ${k}`).toBe(true);
    }
    for (const [k, v] of flat(en)) {
      expect(typeof v === "string" && v.length > 0, `EN key vacía: ${k}`).toBe(true);
    }
  });
});
```

### 9.2 Test manual de cambio de idioma

1. Lanzar app en EN.
2. Settings → Apariencia → cambiar a ES.
3. **Esperado:** todas las pantallas re-renderizan al instante en ES.
4. Cerrar app, relanzar.
5. **Esperado:** sigue en ES.

---

## 10. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Strings olvidados en componentes | Test de coverage + grep check pre-commit |
| Pluralización mal aplicada (ES usa "1 archivo" / "2 archivos") | Keys `*_plural` en JSON |
| Cambio de idioma no re-renderiza al instante | useTranslation provee key reactiva |
| Onboarding aparece a usuarios viejos al hacer update | Flag `ct.onboarding-seen` en localStorage persiste |
| Lectores de pantalla mal manejados (sr-only abusado) | Verificación con NVDA en VM |
| Idioma del SO no detectable | Fallback explícito a "es" |

---

## 11. Definition of Done

- [ ] `i18next` + `react-i18next` + `i18next-browser-languagedetector` instalados.
- [ ] `src/i18n/index.ts` con detección + persistencia.
- [ ] `es.json` y `en.json` con todas las keys del inventario.
- [ ] Componentes refactorizados a `t()`.
- [ ] Sincronización con `settings.appearance.language`.
- [ ] `OnboardingWizard` implementado con 4 pasos.
- [ ] Trigger first-run via localStorage flag.
- [ ] Botón "Ver onboarding de nuevo" en Settings.
- [ ] Focus rings visibles en todos los interactivos.
- [ ] Skip-to-content link.
- [ ] Aria-labels en botones icon-only.
- [ ] `MotionConfig` con `reducedMotion="user"`.
- [ ] Contraste verificado para 5 combos clave.
- [ ] Test `i18n coverage` pasa.
- [ ] Test manual NVDA: navegación por toda la app es coherente.
- [ ] Commit `feat(i18n,a11y,onboarding): ES/EN + wizard de bienvenida + accesibilidad`.

---

## 12. Próximo archivo

→ [13-LANZAMIENTO.md](13-LANZAMIENTO.md) — el cierre. Checklist final + comunicación + post-mortem template.
