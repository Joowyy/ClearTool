# Paso 02 — Frontend component + integration tests

**Área**: 13-testing-ci
**Tiempo estimado**: 4-5 horas
**Dependencias**: Vitest ya instalado (paso `01-error-handling/01`)

## Qué hacemos

Tests unitarios para utilities (errors, toast, formatters) + tests de componente con `@testing-library/react`.

## Archivos

- `package.json` (testing deps)
- `vitest.config.ts` (nuevo o ajustar)
- `src/lib/*.test.ts` (tests de utilities)
- `src/components/__tests__/*.test.tsx` (tests de componentes)

## Cómo

### 1. Instalar deps

```bash
npm i -D vitest @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom
```

### 2. Config Vitest

```ts
// vitest.config.ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
    css: false,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
```

```ts
// src/test-setup.ts
import "@testing-library/jest-dom/vitest";

// Mock de Tauri (no existe en jsdom)
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
```

`package.json`:
```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest run --coverage"
  }
}
```

### 3. Tests de utilities (ya creados en paso 01)

`src/lib/errors.test.ts` (del paso `01-error-handling/01`):
- Verificado que ya cubre `normalizeError`, `formatError`, casos edge.

`src/lib/toast.test.ts` (nuevo, opcional):
```ts
// Mock de sonner para verificar que toast.error mete description correcta
import { vi, describe, it, expect } from "vitest";
import { toast } from "./toast";
import * as sonner from "sonner";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("toast.error", () => {
  it("normaliza AppError en description", () => {
    const err = { kind: "powershell", message: "Get-AppxPackage falló" };
    toast.error("No funcionó", err);
    expect(sonner.toast.error).toHaveBeenCalledWith(
      "No funcionó",
      expect.objectContaining({
        description: "Get-AppxPackage falló",
      })
    );
  });

  it("sin err no incluye description", () => {
    toast.error("Solo title");
    expect(sonner.toast.error).toHaveBeenCalledWith(
      "Solo title",
      expect.objectContaining({ description: undefined }),
    );
  });
});
```

### 4. Tests de componentes — EmptyState

```tsx
// src/components/__tests__/empty-state.test.tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { FileText } from "lucide-react";
import { EmptyState } from "../empty-state";

describe("EmptyState", () => {
  it("renderiza title y description", () => {
    render(
      <EmptyState icon={FileText} title="Sin datos" description="Aún no hay nada." />
    );
    expect(screen.getByText("Sin datos")).toBeInTheDocument();
    expect(screen.getByText("Aún no hay nada.")).toBeInTheDocument();
  });

  it("muestra action cuando se pasa", async () => {
    const onClick = vi.fn();
    render(
      <EmptyState
        icon={FileText}
        title="Sin datos"
        description="..."
        action={{ label: "Empezar", onClick }}
      />
    );
    const btn = screen.getByRole("button", { name: "Empezar" });
    await userEvent.click(btn);
    expect(onClick).toHaveBeenCalled();
  });

  it("no muestra botón sin action", () => {
    render(<EmptyState icon={FileText} title="x" description="y" />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
```

### 5. Tests de hooks (useConfirm)

```tsx
// src/components/__tests__/confirm-provider.test.tsx
import { render, screen, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConfirmProvider, useConfirm } from "../confirm-provider";

function TestComponent({ onResult }: { onResult: (ok: boolean) => void }) {
  const confirm = useConfirm();
  return (
    <button onClick={async () => {
      const ok = await confirm({ title: "Test", description: "..." });
      onResult(ok);
    }}>
      Open
    </button>
  );
}

describe("useConfirm", () => {
  it("resuelve true cuando se confirma", async () => {
    const result = vi.fn();
    render(
      <ConfirmProvider>
        <TestComponent onResult={result} />
      </ConfirmProvider>
    );

    await userEvent.click(screen.getByRole("button", { name: "Open" }));
    await userEvent.click(screen.getByRole("button", { name: "Confirmar" }));

    expect(result).toHaveBeenCalledWith(true);
  });

  it("resuelve false al cancelar", async () => {
    const result = vi.fn();
    render(
      <ConfirmProvider>
        <TestComponent onResult={result} />
      </ConfirmProvider>
    );

    await userEvent.click(screen.getByRole("button", { name: "Open" }));
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));

    expect(result).toHaveBeenCalledWith(false);
  });
});
```

### 6. Tests de integración con React Query

Para mutations + queries:

```tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { vi } from "vitest";
import { CachePage } from "../cache-cleaner/cache-page";

function renderWithQuery(ui: React.ReactElement) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

// Mock de api
vi.mock("../../api", () => ({
  listCacheLocations: vi.fn().mockResolvedValue([
    { id: "tmp", displayName: "Temp", path: "%TEMP%" },
  ]),
}));
```

### 7. Catalog validation script

```bash
npm i -D ajv ajv-cli ajv-formats
```

```json
// package.json
"scripts": {
  "validate-catalogs": "ajv validate -s .claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.schema.json -d .claude/skills/powershell-debloat/RESOURCES/bloatware-catalog.json --spec=draft7 -c ajv-formats && ajv validate -s .claude/skills/cache-scanner/RESOURCES/cache-locations.schema.json -d .claude/skills/cache-scanner/RESOURCES/cache-locations.json --spec=draft7 -c ajv-formats"
}
```

## Criterio de done

- [ ] `npm test` corre suite verde.
- [ ] Tests de errors.ts + toast.ts pasan.
- [ ] Test de EmptyState con action.
- [ ] Test de useConfirm (resolve true/false).
- [ ] `npm run validate-catalogs` valida los 4 JSON contra sus schemas.
- [ ] Coverage de `src/lib/` ≥ 70%.
