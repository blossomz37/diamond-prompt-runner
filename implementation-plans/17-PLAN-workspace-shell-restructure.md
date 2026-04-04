# Plan: Workspace Shell Restructure — Sidebar-Driven Workflow

## Context

The Inspector (right sidebar) currently hosts both read-only summaries *and* interactive authoring UI (Pipelines, Variables, Exports). This makes the Inspector too crowded and puts "doing work" in a narrow column that should only show summaries.

**Principle from the user:**
- Left sidebar = navigation + access to work surfaces
- Center pane = where you do the work
- Right sidebar (Inspector) = read-only summary information only

---

## Current Inspector Sections → Proposed Location

| Section | Current Location | Type | Proposed Location |
|---------|-----------------|------|-------------------|
| **Project Summary** | Inspector | Read-only | Inspector ✓ (stays) |
| **Usage Summary** | Inspector | Read-only | Inspector ✓ (stays) |
| **File Metadata** | Inspector | Read-only | Inspector ✓ (stays) |
| **Run History** | Inspector | Read-only browse | Inspector ✓ (stays) |
| **Variables (edit)** | Inspector | Interactive work | Left sidebar → center pane |
| **Pipelines (author/edit)** | Inspector | Interactive work | Left sidebar → center pane |
| **Exports (select/export)** | Inspector | Interactive work | Left sidebar → center pane |

---

## Proposed Changes

### Left Sidebar: Activity Bar + Switchable Views

Add a VS Code-style **activity bar** (icon strip) at the left edge of the sidebar. Clicking an icon switches the sidebar content between views:

```
┌──┬──────────────────┐
│📁│ Explorer          │  ← File tree (current default)
│🔗│ Pipelines         │  ← Pipeline list + "New Pipeline" entry point
│🔤│ Variables         │  ← Global/Project variable editing
│📦│ Exports           │  ← Export bundle builder
└──┴──────────────────┘
```

- **Explorer** view is the default and works as it does now (file tree + "New Prompt")
- **Pipelines** view shows the list of existing pipelines with summary cards (name, block count, Run button). Clicking "New Pipeline" or "Edit" on a card opens a **Pipeline Editor tab** in the center pane.
- **Variables** view shows the Global + Project variable editing UI (the exact same UI currently in the Inspector, but now in the wider left sidebar)
- **Exports** view shows the export bundle builder UI

### Center Pane: Pipeline Editor as a Tab

Pipeline authoring (create/edit forms with block picker, reorder, save) moves into the center pane as a special tab — similar to how VS Code opens settings or a welcome tab. This gives it the full width of the editor area.

- Tab title: `Pipeline: <name>` (or `New Pipeline` for create)
- Content: the existing create/edit form layout but wider and more spacious
- Saving closes the tab or keeps it open at the pipeline summary

### Inspector: Summary Only

Strip the Inspector down to read-only sections:

1. **Project** — name, root, preset, asset counts
2. **Usage** — runs, tokens, cost, output words
3. **File Metadata** — kind, path, size, modified (for active tab)
4. **Run History** — project-wide run list with filter and "Open artifact"

No forms, no inputs, no authoring controls.

---

## Critical Files

| File | Change |
|------|--------|
| `src/lib/components/WorkspaceShell.svelte` | Add activity bar state + sidebar view switching, render the correct sidebar view |
| `src/lib/components/InspectorPanel.svelte` | Remove Variables, Pipelines, and Exports sections. Keep Project, Usage, File Metadata, Run History |
| `src/lib/components/SidebarPipelines.svelte` | **[NEW]** Pipeline list view for the left sidebar |
| `src/lib/components/SidebarVariables.svelte` | **[NEW]** Variable editing view for the left sidebar (extracted from InspectorPanel) |
| `src/lib/components/SidebarExports.svelte` | **[NEW]** Export builder view for the left sidebar (extracted from InspectorPanel) |
| `src/lib/components/PipelineEditor.svelte` | **[NEW]** Pipeline create/edit form as a center-pane tab |
| `src/App.svelte` | Wire new sidebar view callbacks; add pipeline editor tab support |
| `src/App.test.ts` | Update existing pipeline/variables/export tests to match new locations |
| `src/app.css` | Activity bar styling |

---

## User Review Required

> [!IMPORTANT]
> **Activity bar scope**: Should the activity bar icons be text labels, emoji-style icons, or SVG icons? For v1 I'd default to compact text labels (Explorer / Pipes / Vars / Export) to avoid an icon asset dependency. SVG icons can follow.

> [!IMPORTANT]
> **Variables view**: Should the variable editing UI stay in the left sidebar (it's a compact form that fits well there), or should it also open as a center-pane tab? The current form is small enough for a sidebar — unlike pipeline authoring which genuinely benefits from more space.

> [!IMPORTANT]
> **Exports view**: Same question — the export builder is also fairly compact (checkboxes + name input). Left sidebar vs center pane tab?

---

## Open Questions

1. **Pipeline editor tab lifecycle**: When the user saves a new pipeline, should the tab close automatically, or stay open showing the saved result?
2. **Activity bar persistence**: Should the selected sidebar view persist across sessions, or always reset to Explorer on project open?
3. **Pipeline run from sidebar**: Should the "Run pipeline" button stay on the pipeline card in the sidebar, or should running require opening the pipeline in the center pane?

---

## Verification Plan

### Automated Tests
- `npm run test -- --run` — update existing pipeline/variable/export tests for new DOM locations
- `npm run typecheck` — 0 new errors
- `npm run lint` — clean

### Manual Verification
1. Open a project → Explorer is the default sidebar view
2. Switch to Pipelines view → see existing pipelines, click "New Pipeline" → pipeline editor tab opens in center pane
3. Edit a pipeline → "Edit" opens the pipeline editor tab pre-filled
4. Switch to Variables view → add/remove global and project variables
5. Switch to Exports view → select tabs and export
6. Inspector only shows Project, Usage, File Metadata, Run History
