# Paso 03 — IPC + página dedicada `/privacy`

**Área**: 09-privacy-hardening
**Tiempo estimado**: 4-5 horas
**Dependencias**: Paso 02

## Qué hacemos

IPC + página con 3 radio buttons + preview "Ver cambios" + botón aplicar.

## Archivos

- `src-tauri/src/ipc/privacy.rs` (nuevo)
- `src/api/client.ts` (wrappers)
- `src/features/privacy/privacy-page.tsx` (nuevo)
- `src/lib/routes.ts` (añadir `/privacy`)

## Cómo

### IPC

```rust
#[tauri::command]
pub async fn list_privacy_presets() -> AppResult<PrivacyPresetsFile> {
    crate::domain::privacy::load_presets()
}

#[tauri::command]
pub async fn apply_privacy_preset(level: String, dry_run: bool) -> AppResult<PrivacyApplyReport> {
    crate::domain::privacy::apply_preset(&level, dry_run)
}
```

### TS

```ts
export const listPrivacyPresets = () => invoke<PrivacyPresetsFile>("list_privacy_presets");
export const applyPrivacyPreset = (level: string, dryRun: boolean) =>
  invoke<PrivacyApplyReport>("apply_privacy_preset", { level, dryRun });
```

### Página

```tsx
// src/features/privacy/privacy-page.tsx
import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Shield, ShieldCheck, ShieldAlert } from "lucide-react";
import { listPrivacyPresets, applyPrivacyPreset } from "../../api";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { toast } from "../../lib/toast";

const LEVEL_ICONS = {
  balanced: Shield,
  strict: ShieldCheck,
  paranoid: ShieldAlert,
};

const LEVEL_COLORS = {
  balanced: "text-signal-cyan",
  strict: "text-yellow-400",
  paranoid: "text-signal-red",
};

export function PrivacyPage() {
  const [selected, setSelected] = useState<string>("balanced");
  const [previewOpen, setPreviewOpen] = useState(false);

  const { data } = useQuery({
    queryKey: ["privacy-presets"],
    queryFn: listPrivacyPresets,
  });

  const applyMut = useMutation({
    mutationFn: (dryRun: boolean) => applyPrivacyPreset(selected, dryRun),
    onSuccess: (report) => {
      if (report.dryRun) {
        toast.info("Dry-run completado", {
          description: `${report.registryChanged + report.servicesChanged + report.tasksDisabled + report.appxRemoved} cambios se aplicarían.`,
        });
      } else {
        toast.success(`Privacidad aplicada (${selected})`, {
          description: `${report.registryChanged + report.servicesChanged + report.tasksDisabled + report.appxRemoved} cambios. ${report.failed > 0 ? `${report.failed} fallaron.` : ""}`,
        });
      }
    },
    onError: (err) => toast.error("Error aplicando preset", err),
  });

  if (!data) return null;

  const preset = data.presets[selected];

  return (
    <div className="p-6 flex flex-col gap-4 max-w-4xl mx-auto">
      <div>
        <h2 className="text-2xl font-bold">Privacidad</h2>
        <p className="text-muted-foreground text-sm">
          Bundles coordinados de tweaks para reducir telemetría y publicidad.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        {Object.entries(data.presets).map(([level, p]) => {
          const Icon = LEVEL_ICONS[level as keyof typeof LEVEL_ICONS] ?? Shield;
          const color = LEVEL_COLORS[level as keyof typeof LEVEL_COLORS] ?? "";
          return (
            <button
              key={level}
              onClick={() => setSelected(level)}
              className={`p-4 border rounded-lg text-left transition ${
                selected === level ? "border-signal-cyan bg-signal-cyan/10" : "border-border hover:bg-accent/30"
              }`}
            >
              <Icon className={`h-8 w-8 mb-2 ${color}`} />
              <div className="font-medium">{p.displayName}</div>
              <div className="text-xs text-muted-foreground mt-1">{p.description}</div>
              <Badge variant="secondary" className="mt-2">{p.estimatedChanges} cambios</Badge>
            </button>
          );
        })}
      </div>

      <div className="border border-warning/30 bg-warning/5 rounded-lg p-4">
        <h3 className="font-medium mb-2">Preview: {preset.displayName}</h3>
        <p className="text-sm text-muted-foreground whitespace-pre-line">{preset.disclaimer}</p>

        <details className="mt-3">
          <summary className="cursor-pointer text-sm hover:text-foreground">
            Ver cambios detallados ({preset.registryTweaks.length} registry, {preset.services.length} services, {preset.scheduledTasks.length} tasks, {preset.debloatEntries.length} apps)
          </summary>
          <div className="mt-3 grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            <ChangeList title="Registry tweaks" items={preset.registryTweaks} />
            <ChangeList title="Services → Disabled" items={preset.services.map(s => s.name)} />
            <ChangeList title="Scheduled tasks → Disabled" items={preset.scheduledTasks} />
            <ChangeList title="Apps a desinstalar" items={preset.debloatEntries} />
          </div>
        </details>
      </div>

      <div className="flex gap-2 justify-end">
        <Button variant="outline" onClick={() => applyMut.mutate(true)} disabled={applyMut.isPending}>
          Simular (dry-run)
        </Button>
        <Button
          variant={selected === "paranoid" ? "destructive" : "default"}
          onClick={() => {
            if (selected === "paranoid" && !confirm(preset.disclaimer)) return;
            applyMut.mutate(false);
          }}
          disabled={applyMut.isPending}
        >
          {applyMut.isPending ? "Aplicando..." : `Aplicar ${preset.displayName}`}
        </Button>
      </div>
    </div>
  );
}

function ChangeList({ title, items }: { title: string; items: string[] }) {
  return (
    <div>
      <div className="font-medium text-muted-foreground mb-1">{title}</div>
      <ul className="space-y-0.5 max-h-32 overflow-auto">
        {items.map(i => <li key={i} className="font-mono truncate">{i}</li>)}
      </ul>
    </div>
  );
}
```

## Criterio de done

- [ ] Página `/privacy` accesible.
- [ ] 3 cards de presets clickables.
- [ ] "Ver cambios" muestra desglose por categoría.
- [ ] Dry-run reporta count sin tocar.
- [ ] Aplicar Paranoid pide confirm con disclaimer completo.
- [ ] Toast muestra resumen de cambios aplicados.
- [ ] Restore point se crea automáticamente.
