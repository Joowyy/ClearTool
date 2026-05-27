export { formatBytes } from "./utils";

/** "45 s", "2 min 30 s", "1 h 12 min". Input: seconds. */
export function formatDuration(secs: number): string {
  if (!isFinite(secs) || secs <= 0) return "0 s";
  if (secs < 60) return `${Math.round(secs)} s`;
  if (secs < 3600) {
    const m = Math.floor(secs / 60);
    const s = Math.round(secs % 60);
    return s ? `${m} min ${s} s` : `${m} min`;
  }
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  return m ? `${h} h ${m} min` : `${h} h`;
}
