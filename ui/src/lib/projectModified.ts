/** The project listing stores filesystem modification times as Unix seconds. */
export function formatProjectModified(value: string): string {
  if (!value.trim()) return 'Unknown date';
  const date = /^\d+$/.test(value) ? new Date(Number(value) * 1000) : new Date(value);
  if (!Number.isFinite(date.getTime())) return 'Unknown date';
  return (
    date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' }) +
    ' ' +
    date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
  );
}
