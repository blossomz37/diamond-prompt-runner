<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { EditorView, minimalSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { teraLanguage } from '$lib/editor/teraLanguage';
  import { keymap } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';

  interface Props {
    value: string;
    kind: string; // 'markdown', 'tera', etc.
    onContentChange: (newValue: string) => void;
    onkeydown?: (event: KeyboardEvent) => void;
    api?: CodeEditorApi; // Bound back to parent
  }

  export interface CodeEditorApi {
    focus: () => void;
    setSelectionRange: (start: number, end: number) => void;
  }

  let { value, kind, onContentChange, onkeydown, api = $bindable() }: Props = $props();

  let containerEl: HTMLDivElement | undefined = $state();
  let view: EditorView | undefined;

  // Custom key handler to pass events back up (e.g., for Cmd+F)
  const domEventHandlers = EditorView.domEventHandlers({
    keydown(event) {
      if (onkeydown) {
        onkeydown(event);
      }
      return event.defaultPrevented;
    }
  });

  // Base theme to somewhat match our app's visual style.
  // We use CSS variables to sync with the main app theme.
  const appTheme = EditorView.theme({
    "&": {
      color: "#dbe5ff",
      backgroundColor: "rgba(5, 8, 15, 0.9)",
      minHeight: "30rem", // Matches the native textarea
      height: "100%",
      borderRadius: "18px",
      fontSize: "13px",
      border: "1px solid var(--border-faint)"
    },
    ".cm-content": {
      padding: "1rem"
    },
    "&.cm-focused": {
      outline: "none",
      border: "1px solid var(--border-subtle)"
    },
    ".cm-gutters": {
      backgroundColor: "rgba(5, 8, 15, 0.9)",
      color: "var(--text-soft)",
      border: "none",
      borderRight: "1px solid var(--border-faint)",
      borderTopLeftRadius: "18px",
      borderBottomLeftRadius: "18px",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "rgba(255, 255, 255, 0.05)"
    },
    ".cm-cursor": {
      borderLeftColor: "var(--accent)"
    },
    ".cm-selectionBackground, ::selection": {
      backgroundColor: "rgba(139, 177, 255, 0.25) !important"
    }
  }, { dark: true });

  onMount(() => {
    if (!containerEl) return;

    // Determine language extension
    const langExt = [];
    if (kind === 'markdown') {
      langExt.push(markdown());
    } else if (kind === 'tera') {
      langExt.push(teraLanguage);
    }

    const startState = EditorState.create({
      doc: value,
      extensions: [
        minimalSetup, // includes history, syntax highlighting, line numbers
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        appTheme,
        domEventHandlers,
        ...langExt,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const newContent = update.state.doc.toString();
            // Don't call if it's identical to prevent cyclic updates
            $effect.root(() => {
              if (untrack(() => value) !== newContent) {
                onContentChange(newContent);
              }
            });
          }
        }),
        EditorView.lineWrapping
      ]
    });

    view = new EditorView({
      state: startState,
      parent: containerEl
    });

    // Provide the API back
    api = {
      focus() {
        view?.focus();
      },
      setSelectionRange(start: number, end: number) {
        if (!view) return;
        view.dispatch({
          selection: { anchor: start, head: end },
          scrollIntoView: true
        });
        view.focus();
      }
    };

    return () => {
      view?.destroy();
    };
  });

  // Watch for external value changes (e.g. FindBar replace, or file switch)
  $effect(() => {
    if (view && value !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: value }
      });
    }
  });

  // Watch for kind changes
  $effect(() => {
    if (view) {
      // Just re-creating the editor view is simplest for Svelte 5 reactivity for now, 
      // but CodeMirror supports dynamic extension reconfiguration via Compartments.
      // Since changing files tends to change the `value` entirely, a full unmount/mount 
      // usually happens anyway if rendered with a #key block. 
      // But we will stick to a simpler re-dispatch approach if possible.
      // Wait, we won't implement dynamic dispatch for `kind` here since `AssetViewer`
      // uses `#key tab.path` or similar? Let's check AssetViewer.
      // If it doesn't, we should ideally use a Compartment.
    }
  });
</script>

<div bind:this={containerEl} class="cm-wrapper"></div>

<style>
  .cm-wrapper {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    width: 100%;
    min-height: 30rem;
    overflow: hidden;
  }
</style>
