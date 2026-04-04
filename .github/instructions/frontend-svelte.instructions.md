---
description: "Use when editing Svelte UI, App.svelte orchestration, WorkspaceShell, ValidationPanel, AssetViewer, ExplorerTree, InspectorPanel, or TypeScript files under src/. Covers frontend state flow, Tauri bridge usage, and UI test expectations."
name: "Frontend Svelte"
applyTo: "src/**"
---
# Frontend Svelte Guidelines

- Keep frontend work in `src/` focused on state orchestration and presentation.
- Route backend calls through `src/lib/tauri.ts`; do not invoke Tauri commands inline from scattered components.
- Treat `src/App.svelte` as the top-level workspace coordinator and `src/lib/components/` as shell UI pieces.
- Keep shared frontend domain types in `src/lib/types/project.ts` instead of redefining ad hoc shapes in components.
- Preserve the current IDE-shell flow: explorer selects assets, tabs manage open files, inspector shows metadata, bottom panel shows validation or execution context.
- Respect current product behavior from `README.md` and the active slice in `implementation-plans/09-PLAN-single-block-execution-slice.md` instead of inventing broader UX.
- `project.json` is read-only in the current UI unless the task explicitly changes that product rule.
- `.tera` preview validation is permissive, but execution UI should reflect stricter backend behavior rather than masking errors.
- Prefer extending existing components over creating parallel UI surfaces for the same feature.
- Add or update Vitest + Testing Library coverage in `src/App.test.ts` when frontend behavior changes across the shell boundary.
- Validate frontend changes with the relevant commands from `AGENTS.md`, especially `npm run typecheck`, `npm run lint`, and `npm run test`.

See `AGENTS.md` for repo-wide operating rules and `README.md` for current shipped slice status.