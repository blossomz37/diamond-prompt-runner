# Add Code Editor (CodeMirror 6)

This plan upgrades the plain-text `<textarea>` in `AssetViewer` to a real code editor, utilizing **CodeMirror 6**. This will fulfill the requirement to support syntax highlighting for `.tera` prompt files while preserving existing workflow and preview capabilities.

## User Review Required

> [!IMPORTANT]
> **FindBar Compatibility:** Currently, `FindBar.svelte` tightly couples with the native `HTMLTextAreaElement` by calling `.focus()` and `.setSelectionRange()`. 
> 
> Instead of fully replacing `FindBar` with CodeMirror's native search plugin (which would change the UI), I propose creating an **EditorAdapter Interface** inside `CodeEditor.svelte`. This interface will expose `.focus()` and `.setSelectionRange()` methods to mirror the textarea API, meaning we can preserve the exact same `FindBar` behavior without regressions. Let me know if you prefer to use CodeMirror's native search plugin instead.

## Proposed Changes

### Dependencies
#### [NEW] `package.json`
- Install `codemirror`
- Install `@codemirror/language`, `@codemirror/state`, `@codemirror/view`, `@codemirror/commands`
- Install `@codemirror/lang-markdown` for markdown files.

### Components

#### [NEW] `src/lib/components/CodeEditor.svelte`
- Create a Svelte 5 component wrapping a `EditorView`.
- Accept props: `value`, `kind` ('markdown' | 'tera' | 'plain'), `onChange`.
- Export an `api` object (or methods) returning `{ focus, setSelectionRange }` to replicate the native textarea's API for `FindBar`.
- Sync the `value` prop efficiently by dispatching `onChange` when the view updates, but avoiding reset cycles when typing.

#### [MODIFY] `src/lib/components/FindBar.svelte`
- Broaden the `editorEl` type definition from `HTMLTextAreaElement` to accept duck-typed `CodeEditor` instances that provide `.focus()` and `.setSelectionRange(_start, _end)`.

#### [MODIFY] `src/lib/components/AssetViewer.svelte`
- Replace `<textarea>` with `<CodeEditor bind:api={editorEl} />`.
- Hook up `tab.draftContent` and `onDraftChange` identical to how the textarea worked.
- Ensure the native `keydown` bindings (like `Cmd+F` and `Cmd+H` to trigger `FindBar`) are preserved by either binding them to the `<CodeEditor>` or intercepting globally within the component.

### Language Implementations
#### [NEW] `src/lib/editor/teraLanguage.ts`
- Implement a minimal custom `StreamLanguage` or regular expression highlighter for `.tera` that correctly styles `{{`, `}}`, `{%`, `%}`, internal strings, and identifiers (like `doc("...")`).
- We will lean on `@codemirror/language` to provide a simple parsing mode that doesn't need to perfectly parse mixed markdown for this first phase (as specified).

## Verification Plan

### Automated Tests
- Run `npm run typecheck` to verify strict Typing holds up.
- Run `npm run lint`.
- Verify Svelte tests with `npm run test`. CodeMirror can sometimes cause issues in `jsdom` environments, so we will need to mock its view correctly if failing.

### Manual Verification
1. **Asset File Loading:** Open a `.md` document, open a `.tera` prompt, verify contents display correctly.
2. **Editing:** Edit a `.tera` file, confirm the "Unsaved changes" status updates. Save the file.
3. **Syntax Highlighting:**
   - Markdown files show bold, list, headers correctly.
   - Tera templates highlight `{{ doc("...") }}` accurately.
4. **FindBar functionality:** Press `Cmd+F`, verify input focuses. Searching for words should jump cursor in CodeMirror correctly and mark matches (if needed, though standard selection range works just like textarea).
5. **Switching tabs:** Verify preview mode toggles stay in correct bounds when switching from marked up text back to another tab.
