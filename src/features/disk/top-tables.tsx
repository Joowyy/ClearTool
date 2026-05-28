// top-tables — paneles tabulados con top 20 carpetas, archivos y extensiones.
import * as Tabs from "@radix-ui/react-tabs";
import { FolderTree, FileBarChart, Tag } from "lucide-react";
import { cn, formatBytes } from "../../lib/utils";
import { CATEGORY_COLORS, CATEGORY_LABELS, relativeTime } from "./disk-helpers";
import type { DiskAnalysisReport } from "../../api/types";

interface TopTablesProps {
  report: DiskAnalysisReport | null;
  onSelectPath: (path: string) => void;
}

export function TopTables({ report, onSelectPath }: TopTablesProps) {
  if (!report) return null;

  return (
    <Tabs.Root defaultValue="folders" className="flex flex-col h-full">
      <Tabs.List className="flex items-center gap-1 border-b border-edge-default/10 px-2">
        <TabTrigger value="folders" icon={FolderTree}>
          Top carpetas
        </TabTrigger>
        <TabTrigger value="files" icon={FileBarChart}>
          Top archivos
        </TabTrigger>
        <TabTrigger value="ext" icon={Tag}>
          Extensiones
        </TabTrigger>
      </Tabs.List>

      <div className="flex-1 overflow-auto">
        <Tabs.Content value="folders" className="outline-none">
          <FoldersTable report={report} onSelectPath={onSelectPath} />
        </Tabs.Content>
        <Tabs.Content value="files" className="outline-none">
          <FilesTable report={report} onSelectPath={onSelectPath} />
        </Tabs.Content>
        <Tabs.Content value="ext" className="outline-none">
          <ExtensionsTable report={report} />
        </Tabs.Content>
      </div>
    </Tabs.Root>
  );
}

function TabTrigger({
  value,
  icon: Icon,
  children,
}: {
  value: string;
  icon: typeof FolderTree;
  children: React.ReactNode;
}) {
  return (
    <Tabs.Trigger
      value={value}
      className={cn(
        "inline-flex items-center gap-1.5 px-3 h-8 text-xs rounded-t-md",
        "text-ink-tertiary hover:text-ink-secondary transition-colors",
        "data-[state=active]:text-ink-primary data-[state=active]:bg-signal-cyan/[0.05]",
        "border-b-2 border-transparent data-[state=active]:border-signal-cyan",
      )}
    >
      <Icon className="h-3 w-3" strokeWidth={1.75} />
      {children}
    </Tabs.Trigger>
  );
}

function FoldersTable({
  report,
  onSelectPath,
}: {
  report: DiskAnalysisReport;
  onSelectPath: (path: string) => void;
}) {
  if (report.largestFolders.length === 0)
    return <EmptyRow label="No hay carpetas para listar" />;
  return (
    <table className="w-full text-xs">
      <thead className="sticky top-0 bg-surface-inset/95 backdrop-blur z-10">
        <tr className="text-left text-ink-muted">
          <Th className="w-12 text-right">%</Th>
          <Th>Carpeta</Th>
          <Th className="w-24 text-right">Tamaño</Th>
          <Th className="w-24 text-right">Archivos</Th>
        </tr>
      </thead>
      <tbody>
        {report.largestFolders.map((f) => (
          <tr
            key={f.path}
            onClick={() => onSelectPath(f.path)}
            className="hover:bg-white/[0.03] cursor-pointer border-b border-edge-default/[0.04]"
          >
            <Td className="text-right font-mono tabular-nums text-signal-cyan">
              {f.percent.toFixed(1)}%
            </Td>
            <Td>
              <div className="flex flex-col">
                <span className="text-ink-primary font-medium truncate">{f.name}</span>
                <span className="text-[10px] font-mono text-ink-muted truncate" title={f.path}>
                  {f.path}
                </span>
              </div>
            </Td>
            <Td className="text-right font-mono tabular-nums text-ink-primary">
              {formatBytes(f.bytes)}
            </Td>
            <Td className="text-right font-mono tabular-nums text-ink-tertiary">
              {f.fileCount.toLocaleString("es-ES")}
            </Td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function FilesTable({
  report,
  onSelectPath,
}: {
  report: DiskAnalysisReport;
  onSelectPath: (path: string) => void;
}) {
  if (report.largestFiles.length === 0)
    return <EmptyRow label="No hay archivos para listar" />;
  return (
    <table className="w-full text-xs">
      <thead className="sticky top-0 bg-surface-inset/95 backdrop-blur z-10">
        <tr className="text-left text-ink-muted">
          <Th>Archivo</Th>
          <Th className="w-24 text-right">Tamaño</Th>
          <Th className="w-20">Ext</Th>
          <Th className="w-32">Modif.</Th>
        </tr>
      </thead>
      <tbody>
        {report.largestFiles.map((f) => (
          <tr
            key={f.path}
            onClick={() => onSelectPath(f.path)}
            className="hover:bg-white/[0.03] cursor-pointer border-b border-edge-default/[0.04]"
          >
            <Td>
              <div className="flex flex-col">
                <span className="text-ink-primary font-medium truncate">{f.name}</span>
                <span className="text-[10px] font-mono text-ink-muted truncate" title={f.path}>
                  {f.path}
                </span>
              </div>
            </Td>
            <Td className="text-right font-mono tabular-nums text-ink-primary">
              {formatBytes(f.bytes)}
            </Td>
            <Td className="text-ink-tertiary font-mono">
              {f.extension ? `.${f.extension}` : "—"}
            </Td>
            <Td className="text-ink-tertiary">{relativeTime(f.lastModified)}</Td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function ExtensionsTable({ report }: { report: DiskAnalysisReport }) {
  if (report.topExtensions.length === 0)
    return <EmptyRow label="No hay extensiones para listar" />;
  return (
    <table className="w-full text-xs">
      <thead className="sticky top-0 bg-surface-inset/95 backdrop-blur z-10">
        <tr className="text-left text-ink-muted">
          <Th className="w-14 text-right">%</Th>
          <Th className="w-20">Ext</Th>
          <Th>Categoría</Th>
          <Th className="w-24 text-right">Tamaño</Th>
          <Th className="w-24 text-right">Archivos</Th>
        </tr>
      </thead>
      <tbody>
        {report.topExtensions.map((e) => (
          <tr
            key={`${e.extension}-${e.category}`}
            className="hover:bg-white/[0.03] border-b border-edge-default/[0.04]"
          >
            <Td className="text-right font-mono tabular-nums text-signal-cyan">
              {e.percent.toFixed(1)}%
            </Td>
            <Td className="text-ink-primary font-mono">
              {e.extension ? `.${e.extension}` : "(sin ext)"}
            </Td>
            <Td>
              <span className="inline-flex items-center gap-1.5">
                <span
                  className="w-2 h-2 rounded-sm"
                  style={{ backgroundColor: CATEGORY_COLORS[e.category] }}
                />
                <span className="text-ink-secondary">{CATEGORY_LABELS[e.category]}</span>
              </span>
            </Td>
            <Td className="text-right font-mono tabular-nums text-ink-primary">
              {formatBytes(e.bytes)}
            </Td>
            <Td className="text-right font-mono tabular-nums text-ink-tertiary">
              {e.fileCount.toLocaleString("es-ES")}
            </Td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function Th({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <th
      className={cn(
        "text-[10px] uppercase tracking-wider font-semibold px-3 py-2",
        className,
      )}
    >
      {children}
    </th>
  );
}

function Td({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return <td className={cn("px-3 py-2", className)}>{children}</td>;
}

function EmptyRow({ label }: { label: string }) {
  return (
    <div className="p-6 text-center text-xs text-ink-muted">{label}</div>
  );
}
