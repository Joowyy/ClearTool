// disk-helpers — utilidades compartidas por los componentes del Disk Analyzer.

import type {
  ExtCategory,
  TreemapNode,
} from "../../api/types";

export const CATEGORY_COLORS: Record<ExtCategory | "dir", string> = {
  media: "#a855f7",
  image: "#06b6d4",
  code: "#22c55e",
  docs: "#3b82f6",
  archive: "#f59e0b",
  executable: "#ef4444",
  database: "#f43f5e",
  font: "#ec4899",
  threeD: "#14b8a6",
  other: "#71717a",
  dir: "#1f2937",
};

export const CATEGORY_LABELS: Record<ExtCategory, string> = {
  media: "Media",
  image: "Imagen",
  code: "Código",
  docs: "Docs",
  archive: "Archivo",
  executable: "Ejecutable",
  database: "Base de datos",
  font: "Fuente",
  threeD: "3D",
  other: "Otros",
};

const EXT_CATEGORY_MAP: Record<string, ExtCategory> = {
  mp4: "media", mkv: "media", avi: "media", mov: "media", wmv: "media",
  flv: "media", webm: "media", m4v: "media", mpg: "media", mpeg: "media",
  mp3: "media", wav: "media", flac: "media", aac: "media", ogg: "media",
  m4a: "media", wma: "media", opus: "media",
  jpg: "image", jpeg: "image", png: "image", gif: "image", webp: "image",
  bmp: "image", tiff: "image", tif: "image", svg: "image", ico: "image",
  heic: "image", heif: "image", raw: "image", cr2: "image", nef: "image",
  ts: "code", tsx: "code", js: "code", jsx: "code", rs: "code", py: "code",
  go: "code", java: "code", kt: "code", cpp: "code", cc: "code", c: "code",
  h: "code", hpp: "code", cs: "code", swift: "code", rb: "code", php: "code",
  css: "code", scss: "code", html: "code", vue: "code", svelte: "code",
  lua: "code", sh: "code", ps1: "code", json: "code", toml: "code",
  yaml: "code", yml: "code", xml: "code", md: "code", sql: "code",
  pdf: "docs", docx: "docs", doc: "docs", xlsx: "docs", xls: "docs",
  pptx: "docs", ppt: "docs", odt: "docs", ods: "docs", odp: "docs",
  txt: "docs", rtf: "docs", epub: "docs", mobi: "docs",
  zip: "archive", rar: "archive", "7z": "archive", tar: "archive",
  gz: "archive", bz2: "archive", xz: "archive", iso: "archive", cab: "archive",
  lz: "archive", lzma: "archive", zst: "archive",
  exe: "executable", dll: "executable", msi: "executable", msix: "executable",
  appx: "executable", bat: "executable", cmd: "executable", sys: "executable",
  drv: "executable",
  db: "database", sqlite: "database", sqlite3: "database", mdb: "database",
  accdb: "database", sdf: "database", dat: "database",
  ttf: "font", otf: "font", woff: "font", woff2: "font", fon: "font",
  obj: "threeD", fbx: "threeD", blend: "threeD", stl: "threeD",
  gltf: "threeD", glb: "threeD", dae: "threeD", "3ds": "threeD", max: "threeD",
};

export function categoryOf(ext: string | null | undefined): ExtCategory {
  if (!ext) return "other";
  return EXT_CATEGORY_MAP[ext.toLowerCase()] ?? "other";
}

export function colorForNode(node: TreemapNode): string {
  if (node.kind !== "File") return CATEGORY_COLORS.dir;
  const cat = categoryOf(node.extension);
  return CATEGORY_COLORS[cat];
}

export function lighten(hex: string, amount: number): string {
  return shade(hex, amount);
}

export function darken(hex: string, amount: number): string {
  return shade(hex, -amount);
}

function shade(hex: string, amount: number): string {
  const m = hex.replace("#", "");
  if (m.length !== 6) return hex;
  const r = parseInt(m.slice(0, 2), 16);
  const g = parseInt(m.slice(2, 4), 16);
  const b = parseInt(m.slice(4, 6), 16);
  const adjust = (c: number) => {
    const v = amount >= 0 ? c + (255 - c) * amount : c * (1 + amount);
    return Math.max(0, Math.min(255, Math.round(v)));
  };
  const toHex = (c: number) => c.toString(16).padStart(2, "0");
  return `#${toHex(adjust(r))}${toHex(adjust(g))}${toHex(adjust(b))}`;
}

export function relativeTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    const then = new Date(iso).getTime();
    const diff = Date.now() - then;
    if (diff < 0) return "futuro";
    const s = Math.floor(diff / 1000);
    if (s < 60) return "hace segundos";
    const m = Math.floor(s / 60);
    if (m < 60) return `hace ${m} min`;
    const h = Math.floor(m / 60);
    if (h < 24) return `hace ${h} h`;
    const d = Math.floor(h / 24);
    if (d < 30) return `hace ${d} d`;
    const mo = Math.floor(d / 30);
    if (mo < 12) return `hace ${mo} m`;
    const y = Math.floor(d / 365);
    return `hace ${y} a`;
  } catch {
    return iso;
  }
}

export function findNodeByPath(root: TreemapNode, path: string): TreemapNode | null {
  if (root.path === path) return root;
  for (const c of root.children) {
    const r = findNodeByPath(c, path);
    if (r) return r;
  }
  return null;
}

export function buildBreadcrumbs(rootPath: string, currentPath: string): string[] {
  if (!currentPath.startsWith(rootPath)) return [currentPath];
  const rest = currentPath.slice(rootPath.length);
  const segments = rest.split(/[\\/]/).filter(Boolean);
  return [rootPath, ...segments];
}
