# CSS Audit Report — Diamond Prompt Runner

**Date:** 2026-04-05
**Scope:** Project-owned CSS only (app.css + all Svelte `<style>` blocks)
**Status:** Audit complete — no code changes made

---

## CSS Audit Summary

| Metric | Count |
|---|---|
| Total style sources reviewed | 21 (1 `app.css` + 20 Svelte components) |
| Total CSS lines | ~2,665 |
| Duplicated pattern families | 8 major |
| Hard-coded color values (not tokens) | 104+ occurrences |
| Strong consolidation candidates | 6 pattern families |
| Styles that should remain local | ~60% of total lines |
| Cascade risk areas | 2 (badge variants, button resets) |

### Top-Level Observations

1. **The design token layer is thin.** `app.css` defines 13 custom properties — a solid start — but dozens of raw `rgba()` values repeat across components without token names. Some values appear **22 times** (`rgba(157, 180, 255, 0.16)` — the standard border color).
2. **Several component-level classes are identical across 5–7 files.** `.eyebrow`, `.empty`, `.mini-action`, and the variable panel triplet (`.var-row`, `.var-name`, `.var-value`) are copy-pasted verbatim.
3. **No shared component-style layer exists.** There is no `shared.css`, `tokens.css`, or utility file between `app.css` and the 20 scoped blocks. Every shared pattern has been solved by copy-paste.
4. **`app.css` is lean and appropriate** — resets, font stacks, color-scheme, and base element styling. It does not over-reach.

---

## Findings

### 1. `.eyebrow` — Section Label Pattern

- **Styling scope today:** Scoped in 7 components
- **Recommendation:** **Move to App-Level CSS**
- **Why:** 6 of 7 definitions are character-for-character identical. The 7th (`ProjectBrowser`) differs only by `margin-bottom: 0.35rem` and `font-size: 0.74rem` vs `0.72rem`.
- **Reuse level:** ★★★★★ (7 components)
- **Duplication level:** ★★★★★ (near-identical)
- **Cascade risk:** Low — uses a distinctive class name with no element-type collisions
- **Suggested destination:** `app.css` as a global utility class
- **Components:** AssetViewer, InspectorPanel, PipelineEditorTab, ProjectBrowser, PromptBlockEditorTab, ValidationPanel, WorkspaceShell
- **Notes:** ProjectBrowser variant could use a local override for its bottom margin.

### 2. `.empty` — Empty-State Text

- **Styling scope today:** Scoped in 9 components
- **Recommendation:** **Move to App-Level CSS**
- **Why:** 7 of 9 definitions are identical (`margin: 0; color: var(--text-soft); font-size: 0.82rem`). Remaining 2 use slight variants (`font-style: italic`, different color).
- **Reuse level:** ★★★★★ (9 components)
- **Duplication level:** ★★★★☆
- **Cascade risk:** Low — `.empty` is scoped today but the name is generic enough to warrant care
- **Suggested destination:** `app.css` with variants handled via modifier classes
- **Components:** SidebarDocuments, SidebarExports, SidebarHelp, SidebarPipelines, SidebarPromptBlocks, SidebarPrompts, SidebarRuns, PipelineEditorTab, ValidationPanel

### 3. `.mini-action` — Compact Button

- **Styling scope today:** Scoped in 7 components
- **Recommendation:** **Move to Shared Component Styles**
- **Why:** 5 of 7 definitions are identical. Two variants: SidebarPipelines uses smaller sizing; SidebarSettings/SidebarModels add hover/secondary modifiers.
- **Reuse level:** ★★★★★ (7 components)
- **Duplication level:** ★★★★★
- **Cascade risk:** Low
- **Suggested destination:** Shared `buttons.css` or inline in `app.css`
- **Components:** SidebarExports, SidebarGlobalVariables, SidebarModels, SidebarPipelines, SidebarPrompts, SidebarSettings, SidebarWorkspaceVariables
- **Notes:** Also shares DNA with `.action-btn` — could unify under a single button scale system.

### 4. `.action-btn` — Standard Action Button

- **Styling scope today:** Scoped in 3 components
- **Recommendation:** **Merge/Deduplicate** with `.mini-action` into a button system
- **Why:** Nearly identical to `.mini-action` but with slightly larger sizing (`min-height: 2.2rem` vs `2rem`). The SidebarRuns variant diverges more (dark bg, smaller padding).
- **Reuse level:** ★★★☆☆ (3 components, but with modifiers)
- **Duplication level:** ★★★★☆
- **Cascade risk:** Low
- **Suggested destination:** Shared button system alongside `.mini-action`
- **Components:** PipelineEditorTab, PromptBlockEditorTab, SidebarRuns
- **Notes:** Modifiers `.danger`, `.primary`, `.run`, `.sm` should travel with the base class.

### 5. `.primary` Button Gradient

- **Styling scope today:** Scoped in 6 components
- **Recommendation:** **Move to Shared Component Styles**
- **Why:** Two distinct gradient families repeat:
  - **Accent (blue):** `linear-gradient(135deg, rgba(132, 173, 255, 0.28), rgba(85, 113, 204, 0.36))` — 2 files
  - **Success (green):** `linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28))` — 4 files
- **Reuse level:** ★★★★☆
- **Duplication level:** ★★★★★ (exact same gradient strings)
- **Cascade risk:** Medium — `.primary` is a common class name; must ensure it doesn't leak
- **Suggested destination:** Design tokens for the gradients + shared button modifier
- **Components:** AssetViewer, ProjectBrowser, PipelineEditorTab, ValidationPanel, SidebarExports, SidebarPrompts

### 6. Variable Panel Triplet (`.var-row`, `.var-name`, `.var-value`, `.var-remove`, `.vars-add-form`)

- **Styling scope today:** Scoped in 2 components
- **Recommendation:** **Move to Shared Component Styles**
- **Why:** `SidebarWorkspaceVariables` and `SidebarGlobalVariables` have **100% identical** styling — the entire style block is copy-pasted. The only difference is that Global adds `.var-row.overridden { opacity: 0.4 }`.
- **Reuse level:** ★★★★★ (identical twin pair)
- **Duplication level:** ★★★★★ (verbatim copy)
- **Cascade risk:** Low — class names are specific
- **Suggested destination:** Shared partial or a single extracted component
- **Components:** SidebarWorkspaceVariables, SidebarGlobalVariables

### 7. `.badge` Pattern

- **Styling scope today:** Scoped in 2 components
- **Recommendation:** **Move to Shared Component Styles**
- **Why:** Both `ProjectBrowser` and `ValidationPanel` define `.badge` with `.good`, `.warn`, `.bad` modifiers. The definitions differ slightly (font-size, border style, color hex vs var).
- **Reuse level:** ★★★☆☆
- **Duplication level:** ★★★☆☆ (similar but not identical)
- **Cascade risk:** Medium — `.badge` is a common name
- **Suggested destination:** Shared status-badge component style
- **Components:** ProjectBrowser, ValidationPanel
- **Notes:** Should normalize to use design tokens for the status color families.

### 8. Input Styling Pattern

- **Styling scope today:** Scoped in 10 components
- **Recommendation:** **Convert to Design Tokens + Move to App-Level CSS**
- **Why:** The same input recipe repeats everywhere:
  ```css
  min-height: 2rem;
  border-radius: 8px;
  border: 1px solid rgba(157, 180, 255, 0.16);
  background: rgba(7, 11, 20, 0.82);
  color: var(--text);
  padding: 0.3rem 0.5rem;
  font-size: 11.5px;
  outline: none;
  ```
  Focus state (`border-color: rgba(139, 177, 255, 0.35)`) and placeholder styling are also duplicated.
- **Reuse level:** ★★★★★ (10 components)
- **Duplication level:** ★★★★★
- **Cascade risk:** Low — input styling is inherently element-scoped
- **Suggested destination:** `app.css` input/select/textarea base styling
- **Components:** All sidebar components + ProjectBrowser + FindBar + editor tabs

---

## Design Token Candidates

These raw values appear frequently enough to warrant named tokens in `:root`:

| Current Value | Occurrences | Suggested Token |
|---|---|---|
| `rgba(157, 180, 255, 0.16)` | 22 | `--border-subtle` |
| `rgba(255, 255, 255, 0.04)` | 18 | `--bg-hover` or `--bg-ghost` |
| `rgba(7, 11, 20, 0.82)` | 14 | `--input-bg` |
| `rgba(139, 177, 255, 0.35)` | 9 | `--border-focus` |
| `rgba(139, 177, 255, 0.14)` | 7 | `--bg-active` |
| `rgba(255, 255, 255, 0.08)` | ~8 | `--bg-hover-strong` |
| `rgba(157, 180, 255, 0.12)` | ~12 | `--border-faint` |
| `rgba(255, 141, 161, 0.12)` | ~4 | `--danger-bg` |
| `rgba(153, 227, 190, 0.12)` | ~4 | `--success-bg` |
| `rgba(153, 227, 190, 0.3)` | ~5 | `--success-border` |
| `linear-gradient(135deg, rgba(132, 173, 255...) ` | 2 | `--gradient-accent` |
| `linear-gradient(135deg, rgba(153, 227, 190...)` | 4 | `--gradient-success` |

---

## Keep Local — Components With Correctly Scoped CSS

These components have styling that is tightly coupled to their specific DOM structure and should **not** be centralized:

| Component | Lines | Rationale |
|---|---|---|
| **WorkspaceShell** | 298 | Shell grid layout, responsive breakpoints, sidebar mechanics — all structural |
| **ExplorerTree** | 133 | Depth-based indentation, rename rows, tree-specific node styling |
| **AssetViewer** | 236 | Markdown preview pipeline, textarea sizing, viewer-specific toggle states |
| **ValidationPanel** | 202 | Panel grid, message boxes, run-action cards — highly structural |
| **FindBar** | 79 | Compact find/replace layout — single-use |
| **InspectorPanel** | 49 | Definition-list layout — tightly coupled |
| **ProjectBrowser** | 297 | Welcome screen, recent items, credential row — unique layout |

> [!NOTE]
> "Keep local" means the **structural** portion stays. These components still contain duplicated utility patterns (eyebrow, empty, inputs) that should be extracted.

---

## Monitor Only

| Component | Notes |
|---|---|
| **SidebarHelp** (12 lines) | Too small to matter. Only `.sidebar-help` + `.empty`. |
| **SidebarDocuments** (12 lines) | Same — trivially small. |

---

## Consolidation Plan

### Stage 1: Design Tokens (lowest risk, highest leverage)

**Purpose:** Eliminate raw `rgba()` repetition by naming the 12 most-used color values as CSS custom properties.

**Exact CSS to move:**
- Add ~12 new `--*` tokens to `:root` in `app.css`
- Find-and-replace raw `rgba()` values in all component `<style>` blocks with `var(--token-name)` references

**What stays local:** Everything — this stage only changes values, not selectors or ownership.

**Validation:**
- [ ] `npm run build` succeeds
- [ ] `npm run lint`
- [ ] `npm run typecheck`
- [ ] Visual smoke test: open a project, verify sidebar, editor, and inspector render unchanged
- [ ] Responsive check: resize to below 1160px and 820px breakpoints
- [ ] Interaction: hover states, focus rings, disabled buttons all render correctly
- [ ] Regression: ProjectBrowser welcome screen, ValidationPanel badges, FindBar

### Stage 2: App-Level Utility Classes (`.eyebrow`, `.empty`, base inputs)

**Purpose:** Extract the 3 highest-duplication patterns into `app.css` and remove them from individual component `<style>` blocks.

**Exact CSS to move:**
```css
/* app.css additions */
.eyebrow {
  margin: 0;
  color: var(--accent);
  font-size: 0.72rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.empty-state {
  margin: 0;
  color: var(--text-soft);
  font-size: 0.82rem;
}

input:not([type="checkbox"]):not([type="radio"]),
select,
textarea {
  min-height: 2rem;
  border-radius: 8px;
  border: 1px solid var(--border-subtle);
  background: var(--input-bg);
  color: var(--text);
  padding: 0.3rem 0.5rem;
  outline: none;
}

input:focus, select:focus, textarea:focus {
  border-color: var(--border-focus);
}

input::placeholder, textarea::placeholder {
  color: var(--text-soft);
}
```

**What stays local:**
- ProjectBrowser's `.eyebrow` override (margin-bottom, slightly different font-size)
- Component-specific input sizing overrides (e.g., PromptBlockEditorTab's `min-height: 2.4rem`)

**Validation:**
- [ ] Same as Stage 1 checklist
- [ ] Verify `.eyebrow` usage in all 7 components renders identically
- [ ] Verify Svelte scoping: ensure `:global(.eyebrow)` is applied where `<style>` blocks are removed
- [ ] Verify input styling in: sidebar add-forms, FindBar, ProjectBrowser, editor tabs

> [!WARNING]
> Moving from scoped to global `.eyebrow` and `.empty` requires updating component templates to use `:global()` or removing the class from the `<style>` block entirely. Svelte scopes styles by default — global utility classes applied via HTML class attributes work without `:global()` **only** if the class is defined in an unscoped stylesheet like `app.css`.

### Stage 3: Shared Button System (`.mini-action`, `.action-btn`, `.primary`, `.danger`)

**Purpose:** Unify the 3 overlapping button patterns into a single scale system.

**Exact CSS to move:**
- Define `.btn`, `.btn-sm`, `.btn-primary`, `.btn-danger`, `.btn-success` in `app.css` or a new `shared/buttons.css`
- Migrate component markup from `.mini-action` / `.action-btn` to the unified classes
- Remove per-component button definitions

**What stays local:**
- ValidationPanel's `button` reset (which targets bare `<button>` elements)
- FindBar's `.find-btn` (unique sizing)
- Any button variant with truly one-off styling

**Validation:**
- [ ] All checklist items from Stage 1
- [ ] Verify every button in the app: sidebar add-buttons, editor save/delete, pipeline run, validation run
- [ ] Hover/active/disabled states across all screens
- [ ] Focus-visible ring consistency

### Stage 4: Shared Component Styles (variable panels, badge, status chips)

**Purpose:** Extract shared structural patterns used by component pairs.

**Exact CSS to move:**
- Variable panel styles → shared partial (or refactor into a single `VariablePanel` component)
- `.badge` + `.good/.warn/.bad` → shared status utility
- `.online-chip` / pill patterns → shared status chip

**What stays local:**
- SidebarGlobalVariables' `.overridden` modifier
- Component-specific layout containers

**Validation:**
- [ ] All prior checklist items
- [ ] Verify both variable panels render identically
- [ ] Verify badge rendering in ProjectBrowser and ValidationPanel

---

## Final Recommendation

### What to consolidate first
**Stage 1 (Design Tokens)** delivers the highest ROI with zero risk to layout or cascade. It eliminates ~100+ raw color occurrences and makes future theming possible. Start here.

### What should remain local
~60% of the current CSS is structural layout (grid definitions, responsive breakpoints, panel-specific padding) that is appropriately scoped to individual components. Do not centralize this.

### App-level CSS assessment
`app.css` is currently **too thin**. It handles resets and font stacks well but has not taken ownership of shared design patterns that are clearly app-wide (eyebrow labels, empty states, input styling, status badges). The 13 existing tokens are a good foundation but need to grow to ~25 to cover the repeated `rgba()` palette.

### Suggested shared style layers

```
src/
  app.css              ← Global resets, tokens, typography, base element styling
  lib/
    styles/
      tokens.css       ← Design tokens only (colors, spacing, radii, gradients)
      buttons.css      ← Shared button scale (.btn, .btn-sm, .btn-primary, etc.)
      utilities.css    ← .eyebrow, .empty-state, .badge, .truncate, etc.
```

Alternatively, all of the above can live in a single expanded `app.css` — the component count is small enough that a separate directory is optional for now. The key structural change is naming the repeated values and defining the 4–5 utility classes that currently exist as copy-paste clones.

### Token groups to establish

| Group | Tokens |
|---|---|
| **Surface** | `--input-bg`, `--bg-ghost`, `--bg-hover`, `--bg-hover-strong`, `--bg-active` |
| **Border** | `--border-subtle`, `--border-faint`, `--border-focus`, `--border-strong` (exists as `--panel-border-strong`) |
| **Status** | `--danger-bg`, `--danger-border`, `--success-bg`, `--success-border`, `--warning-bg`, `--warning-border` |
| **Gradient** | `--gradient-accent`, `--gradient-success` |
| **Radius** | `--radius-sm` (6px), `--radius-md` (8-10px), `--radius-lg` (12-14px), `--radius-xl` (18-22px), `--radius-full` (999px) |
