PROMPT: 
Please replace the current plain-text editing surface for asset editing with a real code editor that supports syntax highlighting, while preserving existing draft/save/preview behavior.

Objective

The current editor is a native <textarea>. I want to upgrade it so .tera prompt files can display syntax highlighting for template expressions such as:

{{ doc("Story Dossier.md") }}

I also want markdown files to retain a good editing experience, and I do not want this change to break the existing preview workflow.

Scope

Focus on the asset editor component and any small supporting modules needed for the editor integration.

Please keep the implementation conservative and staged. Prefer a minimal viable editor integration first, then wire language-specific behavior, then verify that the existing app behavior still works.

Requirements

1. Replace the textarea with a real editor

Use a proper embeddable code editor. CodeMirror 6 is the preferred choice because it is lighter-weight and easier to embed than Monaco for this app.

The current editor is a plain textarea, so syntax highlighting is unavailable by design. The new editor should support:
	•	syntax highlighting
	•	cursor/selection editing
	•	keyboard input parity with the current editor where practical
	•	controlled updates back into the app draft state

2. Preserve existing behavior

Keep the current editing model intact:
	•	the editor should still read from tab.draftContent
	•	edits should still flow through the existing draft update path such as onDraftChange(tab.path, value)
	•	markdown preview mode should continue to work
	•	switching tabs/files should still reset preview mode appropriately
	•	do not regress find/close or file switching behavior

3. Language handling

Support language/editor modes by file kind.

At minimum:
	•	markdown → markdown highlighting
	•	tera → Tera-style template highlighting
	•	unknown/plain text kinds → plain text fallback

For .tera files, I want visible highlighting for template syntax including patterns like:
	•	{{ ... }}
	•	{% ... %}
	•	strings inside template calls
	•	identifiers / function names such as doc

A pragmatic first implementation is fine:
	•	either reuse an existing Twig/Jinja-like mode if appropriate
	•	or implement a small custom Tera-style syntax highlighter sufficient for our template patterns

Do not let perfect mixed-language parsing block the first implementation.

4. Mixed content awareness

Some .tera files may contain prose or markdown-like text around template expressions. A full mixed markdown+tera parser is optional for the first pass.

Phase 1 can treat .tera as a template-oriented language as long as template expressions are clearly highlighted.

5. Preview compatibility

The preview button behavior should remain compatible with the existing markdown preview workflow.

This editor upgrade is for the editing surface. It should not break or substantially redesign the preview pipeline unless required for compatibility.

Implementation guidance

Please approach this in stages:

Stage 1 — editor abstraction

Create a reusable editor component, for example CodeEditor.svelte, and swap the existing <textarea> usage to that component.

The component should accept at least:
	•	value
	•	language or kind
	•	onChange callback behavior compatible with the current draft update flow

Stage 2 — language extensions

Wire CodeMirror extensions by file kind:
	•	markdown extension for markdown
	•	tera extension or tera-like extension for template files
	•	plain text fallback otherwise

If there is no clean off-the-shelf Tera mode, implement a lightweight custom mode that highlights:
	•	opening/closing template delimiters
	•	strings
	•	identifiers/function-like names
	•	template keywords if present

Stage 3 — integration verification

Verify that the following still work:
	•	opening a file
	•	editing content
	•	draft changes persist in app state
	•	preview toggle still works for markdown
	•	switching files/tabs does not leave preview state stuck
	•	the editor updates correctly when the active tab changes

Constraints
	•	Keep this implementation modular.
	•	Keep the shell/component boundaries future-friendly.
	•	Avoid introducing broad architectural churn.
	•	Avoid replacing unrelated logic.
	•	Preserve current state flow unless a small adapter is needed.
	•	Prefer a staged implementation plan if the work naturally breaks into multiple commits.

Deliverables

Please provide:
	1.	the implementation
	2.	a short explanation of the approach
	3.	any package dependencies added
	4.	a concise staged test checklist
	5.	any follow-up recommendations for a later second pass, especially if mixed markdown + tera highlighting would benefit from deeper parser work

Success criteria

I should be able to open a .tera prompt file and see syntax highlighting for expressions like:

{{ doc("Story Dossier.md") }}

while still being able to edit markdown files and use the existing preview flow without regressions.