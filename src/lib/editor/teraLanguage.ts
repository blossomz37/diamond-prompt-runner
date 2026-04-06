import { StreamLanguage } from '@codemirror/language';
import type { StreamParser } from '@codemirror/language';

interface TeraState {
  inTag: boolean;
  stringQuote: string | null;
}

const teraParser: StreamParser<TeraState> = {
  name: 'tera',
  startState() {
    return { inTag: false, stringQuote: null };
  },
  token(stream, state) {
    if (!state.inTag) {
      if (stream.match('{{') || stream.match('{%') || stream.match('{#')) {
        state.inTag = true;
        return 'bracket';
      }
      stream.next();
      return null; // plain text
    }

    if (state.stringQuote) {
      if (stream.match(state.stringQuote)) {
        state.stringQuote = null;
        return 'string';
      }
      // Eat characters until the next quote or tag boundary
      while (!stream.eol() && stream.peek() !== state.stringQuote && stream.peek() !== '}' && stream.peek() !== '%') {
        stream.next();
      }
      return 'string';
    }

    if (stream.match('}}') || stream.match('%}') || stream.match('#}')) {
      state.inTag = false;
      return 'bracket';
    }

    // Match keywords like if, for, in, endfor, endif, set, etc.
    if (stream.match(/^(?:if|else|elif|endif|for|in|endfor|set|macro|endmacro|include|import|from|doc)\b/)) {
      return 'keyword';
    }

    // Match variable names
    if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*/)) {
      return 'variable';
    }

    // Match strings
    const ch = stream.peek();
    if (ch === '"' || ch === "'") {
      state.stringQuote = ch;
      stream.next();
      return 'string';
    }

    stream.next();
    return null;
  }
};

export const teraLanguage = StreamLanguage.define(teraParser);
