import { describe, it, expect } from "vitest";
import { normalizeError, formatError } from "./errors";

describe("normalizeError", () => {
  it("normaliza AppError de Tauri", () => {
    const err = { kind: "powershell", message: "Get-AppxPackage falló" };
    const n = normalizeError(err);
    expect(n.kind).toBe("powershell");
    expect(n.message).toBe("Get-AppxPackage falló");
    expect(n.isPermission).toBe(false);
  });

  it("detecta permisos", () => {
    const err = { kind: "permission", message: "Access denied" };
    expect(normalizeError(err).isPermission).toBe(true);
  });

  it("normaliza Error standard", () => {
    const err = new Error("boom");
    expect(normalizeError(err).message).toBe("boom");
  });

  it("normaliza string", () => {
    expect(normalizeError("texto").message).toBe("texto");
  });

  it("fallback con objeto extraño", () => {
    expect(normalizeError({ foo: 1 }).message).toBe('{"foo":1}');
  });

  it("formatError nunca devuelve [object Object]", () => {
    const inputs: unknown[] = [
      { kind: "io", message: "x" },
      new Error("y"),
      "z",
      { random: true },
      null,
      undefined,
    ];
    for (const i of inputs) {
      expect(formatError(i)).not.toContain("[object Object]");
    }
  });
});
