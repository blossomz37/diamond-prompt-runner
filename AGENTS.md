# AGENTS.md

## Purpose
This file defines how agents should operate in this workspace:
- how to orient
- which documents are authoritative
- how to scope work
- how to validate work
- how to keep docs and repo state aligned

## Current Workspace Reality
- This repo is currently spec-first and prototype-first.
- The intended product is Diamond Prompt Runner, a local-first prompt-workflow tool.
- The target app direction is a Tauri desktop app with a Svelte frontend.
- `workshop-parts/` contains reference assets and starter material, not automatic product truth.

## Authority And Reading Order
Read in this order before making meaningful changes:
1. `AGENTS.md`
2. `README.md`
3. `specifications/SPEC_DIAMOND_RUNNER_v1.md`
4. active files under `implementation-plans/`
5. `prototypes/PROTOTYPES_LIST.md`
6. current code and config, if implementation exists
7. `workshop-parts/` only when the task explicitly ports, adapts, or audits reference material

Rules:
- The spec wins over old reference code.
- Implementation plans guide sequencing, not product truth.
- Prototype docs inform UX direction, not architecture.
- If docs conflict, surface the conflict explicitly before inventing behavior.

## Scope Control
Scope discipline is mandatory.

Rules:
- Keep one work unit focused on one requested outcome.
- Treat adjacent issues as follow-up notes, not permission.
- Do not silently broaden scope.
- Ask before changing architecture, file formats, public behavior, or project direction.
- When UI or workflow changes materially, update the authority docs in the same work unit.
- Prefer the smallest change that preserves future extension paths.

## Default Operating Pattern
1. Orient with the current repo state and authority docs.
2. Confirm whether the task is doc-first, prototype-first, or implementation work.
3. Prefer existing local patterns over new abstractions unless the task requires a new structure.
4. For doc or spec tasks, update authority docs before lower-priority notes.
5. For implementation tasks, align changes to the approved IDE-shell direction and the current v2 spec.
6. If the spec is silent on an impactful product decision, stop and ask before inventing behavior.

Engineering defaults for this repo:
- Prefer vertical slices over screen-by-screen implementation.
- Keep the first implementation milestone focused on create or open project plus read-and-navigate shell behavior.
- Keep Diamond file-first. Do not import database-first patterns from sibling repos unless the repo docs explicitly adopt them.
- Treat `workshop-parts/` as source material for adaptation, not as runtime architecture.

## Validation Rules
Validation depends on the task type.

For doc, spec, or prototype-reference work:
- verify affected docs remain consistent with each other
- confirm paths, filenames, and product terms match the current workspace

For implementation work:
- run the relevant tests, type checks, and app validation before closing the task
- verify real behavior when the task changes user-visible workflows

General rules:
- Do not ask for manual UI testing until the relevant UI is actually wired up.
- If a command hangs or fails repeatedly, stop looping and report clearly.
- Use bounded commands or explicit timeouts for long-running tasks.
- Do not silently rerun the same failing command without changing approach.

## Tooling Guidance
Preferred tools for this repo:
- Figma for IDE-shell prototype work, layout iteration, and later Code Connect mapping.
- Playwright for UI smoke checks, navigation tests, and screenshots once the app shell exists.
- Svelte docs and Context7 for framework and Tauri implementation guidance.
- GitHub tools for repo hygiene, PRs, releases, and publish workflows once active development is underway.

Memory MCP:
- Memory MCP is available and currently persists to `memory.jsonl` in this workspace.
- Use it for stable repo facts and cross-session continuity only.
- Do not use it as product/runtime architecture or as a substitute for repo docs.
- Update it sparingly, typically only when an approved decision or durable repo fact would be costly to rediscover later.

Use with caution:
- Notion is useful for planning and search, but tool reliability may vary by path or transport.
- SQLite tooling may help inspect legacy/reference data, but Diamond itself should remain file-first.
- Google Calendar, Gmail, Google Drive, Canva, Dropbox, and Slack are workflow helpers, not core build tools for this repo.

## Git Workflow
- One logical change per commit.
- Keep commit messages precise.
- Update stale authority docs before committing when behavior or workflow changes.
- Repository URL: `https://github.com/blossomz37/diamond-prompt-runner`
- Commit when one logical, validated work unit is complete.
- Commit before a risky refactor if the current state is coherent and worth preserving.
- Do not commit half-working experiments or mixed unrelated changes.
- Do not push automatically after every commit.
- Push when the user explicitly asks to publish, or when the task is explicitly a push, release, or publish workflow.
- If pushing, prefer `origin main` unless the user asked for a branch workflow.

## Documentation Update Targets
Keep these aligned when work changes product direction, implementation assumptions, workflow, repo structure, or selected prototype direction:
- `README.md`
- `specifications/SPEC_DIAMOND_RUNNER_v1.md`
- files under `implementation-plans/`
- `prototypes/PROTOTYPES_LIST.md`
- `TODO.md`

## Handoff Format
If ending a session with unfinished work, leave a short handoff with:
- Done
- Open
- Next
- Notes

## Default Decision Rule
When in doubt:
1. reread the local authority docs
2. prefer the narrower scope
3. validate current repo state
4. ask before making impactful product decisions
5. leave the workspace in a clean, understandable state
