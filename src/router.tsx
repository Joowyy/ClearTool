import { createBrowserRouter } from "react-router-dom";
import { Suspense } from "react";
import { AppShell } from "./components/layout/app-shell";
import { ErrorBoundary } from "./components/error-boundary";
import { TableSkeleton } from "./components/ui/table-skeleton";
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
import { DiskPage } from "./features/disk/disk-page";
import { PrivacyPage } from "./features/privacy/privacy-page";
import { ROUTES } from "./lib/routes";

function LazyPage({ children }: { children: React.ReactNode }) {
  return (
    <Suspense fallback={<TableSkeleton rows={8} columns={4} className="p-4" />}>
      {children}
    </Suspense>
  );
}

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    errorElement: <ErrorBoundary />,
    children: [
      { index: true, element: <LazyPage><HomePage /></LazyPage> },
      { path: ROUTES.EXPLORER, element: <LazyPage><ExplorerPage /></LazyPage> },
      { path: ROUTES.CACHE, element: <LazyPage><CachePage /></LazyPage> },
      { path: ROUTES.DEBLOAT, element: <LazyPage><DebloatPage /></LazyPage> },
      { path: ROUTES.SERVICES, element: <LazyPage><ServicesPage /></LazyPage> },
      { path: ROUTES.REGISTRY, element: <LazyPage><RegistryPage /></LazyPage> },
      { path: ROUTES.RESTORE, element: <LazyPage><RestorePage /></LazyPage> },
      { path: ROUTES.AUDIT, element: <LazyPage><AuditPage /></LazyPage> },
      { path: ROUTES.PROCESSES, element: <LazyPage><ProcessesPage /></LazyPage> },
      { path: ROUTES.STARTUP, element: <LazyPage><StartupPage /></LazyPage> },
      { path: ROUTES.DISK, element: <LazyPage><DiskPage /></LazyPage> },
      { path: ROUTES.PRIVACY, element: <LazyPage><PrivacyPage /></LazyPage> },
      { path: ROUTES.SETTINGS, element: <LazyPage><SettingsPage /></LazyPage> },
    ],
  },
]);
