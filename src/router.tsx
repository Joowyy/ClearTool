import { createBrowserRouter } from "react-router-dom";
import { AppShell } from "./components/layout/app-shell";
import { ErrorBoundary } from "./components/error-boundary";
import { HomePage } from "./features/home/home-page";
import { ExplorerPage } from "./features/explorer/explorer-page";
import { CachePage } from "./features/cache-cleaner/cache-page";
import { DebloatPage } from "./features/debloat/debloat-page";
import { ServicesPage } from "./features/services/services-page";
import { RegistryPage } from "./features/registry-tweaks/registry-page";
import { RestorePage } from "./features/restore-points/restore-page";
import { AuditPage } from "./features/audit-log/audit-page";
import { SettingsPage } from "./features/settings/settings-page";
import { ProcessesPage } from "./features/processes/processes-page";
import { StartupPage } from "./features/startup/startup-page";
import { ROUTES } from "./lib/routes";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    errorElement: <ErrorBoundary />,
    children: [
      { index: true, element: <HomePage /> },
      { path: ROUTES.EXPLORER, element: <ExplorerPage /> },
      { path: ROUTES.CACHE, element: <CachePage /> },
      { path: ROUTES.DEBLOAT, element: <DebloatPage /> },
      { path: ROUTES.SERVICES, element: <ServicesPage /> },
      { path: ROUTES.REGISTRY, element: <RegistryPage /> },
      { path: ROUTES.RESTORE, element: <RestorePage /> },
      { path: ROUTES.AUDIT, element: <AuditPage /> },
      { path: ROUTES.PROCESSES, element: <ProcessesPage /> },
      { path: ROUTES.STARTUP, element: <StartupPage /> },
      { path: ROUTES.SETTINGS, element: <SettingsPage /> },
    ],
  },
]);
