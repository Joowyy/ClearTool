import { createBrowserRouter } from "react-router-dom";
import { Suspense, lazy } from "react";
import { AppShell } from "./components/layout/app-shell";
import { ErrorBoundary } from "./components/error-boundary";
import { TableSkeleton } from "./components/ui/table-skeleton";
import { ROUTES } from "./lib/routes";

const HomePage = lazy(() => import("./features/home/home-page").then(m => ({ default: m.HomePage })));
const ExplorerPage = lazy(() => import("./features/explorer/explorer-page").then(m => ({ default: m.ExplorerPage })));
const CachePage = lazy(() => import("./features/cache-cleaner/cache-page").then(m => ({ default: m.CachePage })));
const DebloatPage = lazy(() => import("./features/debloat/debloat-page").then(m => ({ default: m.DebloatPage })));
const ServicesPage = lazy(() => import("./features/services/services-page").then(m => ({ default: m.ServicesPage })));
const RegistryPage = lazy(() => import("./features/registry-tweaks/registry-page").then(m => ({ default: m.RegistryPage })));
const RestorePage = lazy(() => import("./features/restore-points/restore-page").then(m => ({ default: m.RestorePage })));
const AuditPage = lazy(() => import("./features/audit-log/audit-page").then(m => ({ default: m.AuditPage })));
const SettingsPage = lazy(() => import("./features/settings/settings-page").then(m => ({ default: m.SettingsPage })));
const ProcessesPage = lazy(() => import("./features/processes/processes-page").then(m => ({ default: m.ProcessesPage })));
const StartupPage = lazy(() => import("./features/startup/startup-page").then(m => ({ default: m.StartupPage })));
const DiskPage = lazy(() => import("./features/disk/disk-page").then(m => ({ default: m.DiskPage })));
const PrivacyPage = lazy(() => import("./features/privacy/privacy-page").then(m => ({ default: m.PrivacyPage })));

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
