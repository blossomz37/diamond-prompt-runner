export function getMatches(content: string, query: string, caseSensitive: boolean): number[] {
  if (!query) return [];
  const hay = caseSensitive ? content : content.toLowerCase();
  const needle = caseSensitive ? query : query.toLowerCase();
  const positions: number[] = [];
  let idx = 0;
  while ((idx = hay.indexOf(needle, idx)) !== -1) {
    positions.push(idx);
    idx += 1;
  }
  return positions;
}

export function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
