export const ONLINE_PROMPT_DIRECTIVE = '{# diamond:online #}';

export function promptUsesOnlineResearch(content: string): boolean {
  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    return line === ONLINE_PROMPT_DIRECTIVE;
  }

  return false;
}