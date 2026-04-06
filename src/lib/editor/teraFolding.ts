import type { EditorState } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
import {
  codeFolding,
  foldEffect,
  foldGutter,
  foldService,
  foldedRanges,
  unfoldAll
} from '@codemirror/language';

const TERA_ONLY_LINE = /^\s*(?:\{#.*#\}|\{%.*%\}|\{\{.*\}\})\s*$/;
const BLANK_LINE = /^\s*$/;
const FOLD_MARKER = /^\s*\{#\s*fold(?:\s*:\s*([^#]+?))?\s*#\}\s*$/i;

function isFoldableTeraLine(text: string): boolean {
  return TERA_ONLY_LINE.test(text);
}

function isBlankLine(text: string): boolean {
  return BLANK_LINE.test(text);
}

function getFoldMarkerLabel(text: string): string | null {
  const match = text.match(FOLD_MARKER);
  return match?.[1]?.trim() ?? (match ? 'Folded Tera block' : null);
}

function markerFoldRange(state: EditorState, lineStart: number) {
  const startLine = state.doc.lineAt(lineStart);
  const label = getFoldMarkerLabel(startLine.text);
  if (!label) {
    return null;
  }

  let lastLine = startLine;
  let nextNumber = startLine.number + 1;

  while (nextNumber <= state.doc.lines) {
    const nextLine = state.doc.line(nextNumber);
    if (getFoldMarkerLabel(nextLine.text)) {
      break;
    }

    lastLine = nextLine;
    nextNumber += 1;
  }

  if (lastLine.number === startLine.number) {
    return null;
  }

  return {
    from: startLine.from,
    to: lastLine.to,
    placeholder: label
  };
}

function fallbackFoldRange(state: EditorState, lineStart: number) {
  const startLine = state.doc.lineAt(lineStart);
  if (!isFoldableTeraLine(startLine.text) || getFoldMarkerLabel(startLine.text)) {
    return null;
  }

  let lastLine = startLine;
  let nextNumber = startLine.number + 1;

  while (nextNumber <= state.doc.lines) {
    const nextLine = state.doc.line(nextNumber);
    if (!isFoldableTeraLine(nextLine.text) || getFoldMarkerLabel(nextLine.text)) {
      break;
    }

    lastLine = nextLine;
    nextNumber += 1;
  }

  if (lastLine.number === startLine.number) {
    return null;
  }

  return {
    from: startLine.to,
    to: lastLine.to,
    placeholder: 'Tera block'
  };
}

function teraFoldRange(state: EditorState, lineStart: number) {
  return markerFoldRange(state, lineStart) ?? fallbackFoldRange(state, lineStart);
}

export const teraFolding = [
  codeFolding({
    placeholderDOM(view, onclick, prepared) {
      const span = document.createElement('span');
      span.textContent = `${String(prepared ?? 'Tera block')}…`;
      span.className = 'cm-teraFoldPlaceholder';
      span.setAttribute('title', 'Expand folded Tera block');
      span.onclick = onclick;
      return span;
    },
    preparePlaceholder(state, range) {
      return teraFoldRange(state, range.from)?.placeholder ?? 'Tera block';
    }
  }),
  foldGutter({ openText: '⌄', closedText: '›' }),
  foldService.of((state, lineStart) => teraFoldRange(state, lineStart))
];

export function setTeraBlocksFolded(view: EditorView, folded: boolean): void {
  if (!folded) {
    unfoldAll(view);
    return;
  }

  const effects = [];
  const existing = foldedRanges(view.state);

  for (let lineNumber = 1; lineNumber <= view.state.doc.lines; ) {
    const line = view.state.doc.line(lineNumber);
    const range = teraFoldRange(view.state, line.from);

    if (range) {
      let alreadyFolded = false;
      existing.between(range.from, range.to, (from, to) => {
        if (from === range.from && to === range.to) {
          alreadyFolded = true;
        }
      });

      if (!alreadyFolded) {
        effects.push(foldEffect.of(range));
      }

      lineNumber = view.state.doc.lineAt(range.to).number + 1;
    } else {
      lineNumber += 1;
    }
  }

  if (effects.length > 0) {
    view.dispatch({ effects });
  }
}