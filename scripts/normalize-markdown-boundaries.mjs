/**
 * Script: normalize-markdown-boundaries.mjs
 * Purpose: Convert heading-based markdown into nested bracket-boundary sections.
 * Usage:
 *   node scripts/normalize-markdown-boundaries.mjs input.md [output.md] [--exact-level=N] [--levels=2,3] [--preserve-heading]
 */
import fs from 'fs';
import path from 'path';

function normalizeHeadingToTag(rawHeading) {
  return rawHeading
    .trim()
    .toLowerCase()
    .replace(/[`*~]/g, '')
    .replace(/[\s-]+/g, '_')
    .replace(/[^a-z0-9_]+/g, '')
    .replace(/^_+|_+$/g, '')
    .replace(/_+/g, '_');
}

function shouldWrapLevel(level, exactLevel, levelSet) {
  if (levelSet) {
    return levelSet.has(level);
  }

  if (exactLevel !== null) {
    return level === exactLevel;
  }

  return true;
}

function renderBoundaries(markdown, options = {}) {
  const { exactLevel = null, levelSet = null, preserveHeading = false } = options;
  const lines = markdown.split(/\r?\n/);
  const output = [];
  const stack = [];

  for (const line of lines) {
    const headingMatch = line.match(/^(#{1,6})\s+(.+?)\s*$/);
    if (!headingMatch) {
      output.push(line);
      continue;
    }

    const level = headingMatch[1].length;
    if (!shouldWrapLevel(level, exactLevel, levelSet)) {
      output.push(line);
      continue;
    }

    const rawHeading = headingMatch[2];
    const tag = normalizeHeadingToTag(rawHeading);

    if (!tag) {
      continue;
    }

    while (stack.length > 0 && stack[stack.length - 1].level >= level) {
      const closing = stack.pop();
      output.push(`[/${closing.tag}]`);
      output.push('');
    }

    output.push(`[${tag}]`);
    output.push('');
    if (preserveHeading) {
      output.push(line);
      output.push('');
    }
    stack.push({ level, tag });
  }

  while (stack.length > 0) {
    const closing = stack.pop();
    output.push(`[/${closing.tag}]`);
    if (stack.length > 0) {
      output.push('');
    }
  }

  return `${output.join('\n').replace(/\n{3,}/g, '\n\n')}\n`;
}

function main() {
  const positional = [];
  let exactLevel = null;
  let levelSet = null;
  let preserveHeading = false;

  for (const arg of process.argv.slice(2)) {
    if (arg.startsWith('--exact-level=')) {
      const parsed = Number.parseInt(arg.split('=')[1], 10);
      if (!Number.isInteger(parsed) || parsed < 1 || parsed > 6) {
        console.error('Error: --exact-level must be an integer from 1 to 6.');
        process.exit(1);
      }
      exactLevel = parsed;
      continue;
    }

    if (arg.startsWith('--levels=')) {
      const parsedLevels = arg
        .split('=')[1]
        .split(',')
        .map((value) => Number.parseInt(value.trim(), 10));

      if (
        parsedLevels.length === 0
        || parsedLevels.some((value) => !Number.isInteger(value) || value < 1 || value > 6)
      ) {
        console.error('Error: --levels must be a comma-separated list of integers from 1 to 6.');
        process.exit(1);
      }

      levelSet = new Set(parsedLevels);
      continue;
    }

    if (arg === '--preserve-heading') {
      preserveHeading = true;
      continue;
    }

    positional.push(arg);
  }

  const inputPath = positional[0];
  const outputPath = positional[1];

  if (!inputPath) {
    console.error('Usage: node scripts/normalize-markdown-boundaries.mjs input.md [output.md] [--exact-level=N] [--levels=2,3] [--preserve-heading]');
    process.exit(1);
  }

  if (exactLevel !== null && levelSet !== null) {
    console.error('Error: use either --exact-level or --levels, not both.');
    process.exit(1);
  }

  const absoluteInputPath = path.resolve(process.cwd(), inputPath);
  const absoluteOutputPath = path.resolve(
    process.cwd(),
    outputPath || inputPath.replace(/\.md$/i, '.boundaries.md')
  );

  const source = fs.readFileSync(absoluteInputPath, 'utf8');
  const normalized = renderBoundaries(source, { exactLevel, levelSet, preserveHeading });
  fs.writeFileSync(absoluteOutputPath, normalized);

  console.log(`Wrote ${path.relative(process.cwd(), absoluteOutputPath)}`);
}

main();
