import { parse } from 'yaml';

export interface YamlPromptPaths {
  alias: string;
  branches: string[];
  leaves: string[];
  error?: string;
}

export function buildYamlPromptPaths(path: string, content: string): YamlPromptPaths {
  const alias = aliasFromPath(path);

  try {
    const parsed = parse(content) as unknown;
    const branches: string[] = [];
    const leaves: string[] = [];

    visitYamlNode(parsed, alias, [], branches, leaves);

    return {
      alias,
      branches,
      leaves
    };
  } catch (error) {
    return {
      alias,
      branches: [],
      leaves: [],
      error: error instanceof Error ? error.message : 'YAML could not be parsed.'
    };
  }
}

function aliasFromPath(path: string): string {
  const filename = path.split('/').pop() ?? path;
  const basename = filename.replace(/\.[^.]+$/, '');
  const normalized = basename
    .replace(/[^A-Za-z0-9_]+/g, '_')
    .replace(/^_+|_+$/g, '');

  if (!normalized) {
    return 'document';
  }

  return /^[A-Za-z_]/.test(normalized) ? normalized : `doc_${normalized}`;
}

function visitYamlNode(
  value: unknown,
  alias: string,
  segments: string[],
  branches: string[],
  leaves: string[]
): void {
  if (isPlainObject(value)) {
    if (segments.length > 0) {
      branches.push(buildReference(alias, segments));
    }

    for (const [key, child] of Object.entries(value)) {
      visitYamlNode(child, alias, [...segments, key], branches, leaves);
    }
    return;
  }

  if (segments.length === 0) {
    leaves.push(`{{ ${alias} }}`);
    return;
  }

  leaves.push(buildReference(alias, segments));
}

function buildReference(alias: string, segments: string[]): string {
  return `{{ ${alias}${segments.map(formatSegment).join('')} }}`;
}

function formatSegment(segment: string): string {
  if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(segment)) {
    return `.${segment}`;
  }

  const escaped = segment.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
  return `["${escaped}"]`;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
