---
created: 2026-04-03 2:30 PM
modified: 2026-04-03 2:30 PM
---
# Diamond Runner Agent Tooling And Workflow Plan

## Summary

This document records which MCP tools, connector surfaces, plugins, and repo workflows are most useful for Diamond Prompt Runner. It distinguishes tools that were directly tested in this workspace from tools that are configured but lower priority. It also captures commit and push triggers plus recommended operator-side settings for Personality, Custom Instructions, Plugins, and Skills.

## Tested Tool Audit

### Working and useful now

- `Figma MCP`
  - confirmed working through identity and prior design work in the Diamond mockup file
  - best for prototype iteration, IDE-shell layout work, and later design-to-code mapping
- `Playwright MCP`
  - confirmed working with browser tab and snapshot commands
  - best for UI smoke tests, navigation checks, screenshots, and regression coverage once the app shell exists
- `Svelte MCP`
  - confirmed working through section discovery and documentation fetch
  - best for Svelte 5 patterns, component architecture, and testing guidance
- `Context7`
  - confirmed working for both Tauri and Svelte documentation lookups
  - best for official implementation guidance while building Tauri commands, filesystem access, and validation/test setup
- `Memory MCP`
  - confirmed working
  - currently persists to `memory.jsonl` in this workspace
  - best for cross-session recall and reference memory, not as product/runtime architecture
- `Notion app search`
  - confirmed working through the app connector search path
  - best for finding project notes, specs, or planning material if they live in Notion
- `Google Calendar app`
  - confirmed working through profile lookup
  - best for planning and scheduling workflows, not for the Diamond runtime
- `SQLite MCP`
  - confirmed working against an existing local SQLite database
  - useful for legacy/reference inspection only; Diamond should stay file-first

### Working but lower priority for this repo

- `GitHub` app connector surface is available and should be treated as useful once implementation and PR flows are active
- `Google Drive`, `Gmail`, `Canva`, `Dropbox`, and `Slack` are workflow helpers, not core Diamond engineering tools

### Mixed or flaky

- direct low-level `Notion MCP` calls were not fully reliable in this session
- app-level Notion search worked, so prefer the app-search path over lower-level direct calls when possible

## Recommended Best Use Cases For Diamond

### Core build tools

- `Figma`
  - prototype iteration
  - future design-system work
  - validating shell density and layout decisions
- `Playwright`
  - shell smoke tests
  - file-open navigation checks
  - screenshot-based validation of milestone slices
- `Svelte MCP` and `Context7`
  - Svelte 5 implementation guidance
  - Tauri command and filesystem API guidance
  - testing setup and framework-specific validation rules
- `GitHub`
  - branch, PR, release, and publish workflows
  - commit and repo-state visibility

### Project workflow helpers

- `Notion`
  - planning and cross-document search if project docs live there
- `Google Drive`
  - reference docs or external supporting material if needed
- `Google Calendar`
  - planning and scheduling work sessions

### Not recommended as product architecture inputs

- `SQLite MCP`
  - do not use this to justify database-first design for Diamond
- `Memory MCP`
  - useful for the agent, not the product runtime

## Memory MCP Usage Policy

### Current persistence setup

- Memory MCP is currently configured to persist to `memory.jsonl` in this workspace.
- This is agent-side continuity storage, not application data for Diamond Prompt Runner.

### What to store

Use Memory MCP for stable facts such as:
- approved product or UX direction
- authority-doc ordering and repo operating rules
- durable milestone definitions
- durable repository metadata that future sessions should recall quickly

### What not to store

Do not use Memory MCP for:
- temporary task state
- draft ideas or speculative design notes
- noisy implementation details that are easy to recover from the repo
- runtime or user data for the Diamond product

### Update frequency

Use Memory MCP sparingly.

Practical rule:
- update it only when a fact has been approved or stabilized and would be annoying to rediscover later
- most sessions should add zero to three memory updates, not continuous writes

## Commit And Push Triggers

### Commit triggers

Commit when:
- one logical, validated work unit is complete
- authority docs and behavior are back in sync
- a coherent checkpoint is worth preserving before a risky refactor
- a session ends with a clean, understandable milestone state

Do not commit:
- half-working experiments
- unrelated mixed changes
- temporary exploratory edits that have not converged into a coherent unit

### Push triggers

Push only when:
- the user explicitly asks to publish or push
- the task is explicitly a push, backup, share, PR, or release workflow

Default push target:
- prefer `origin main` unless the user asked for a branch workflow

## Recommended Operator Settings

### Personality and custom-instruction themes

Recommended instruction themes for this repo:
- treat `AGENTS.md`, `README.md`, and `specifications/SPEC_DIAMOND_RUNNER_v1.md` as authority
- Diamond is file-first and should not drift into database-first architecture
- prefer vertical slices over screen-by-screen implementation
- the first implementation milestone is create or open project plus read-and-navigate shell behavior
- `workshop-parts/` is reference material, not runtime truth
- update docs when behavior or workflow changes
- keep commits local and milestone-based; keep pushes explicit
- stay terse, factual, implementation-first, and skeptical of scope creep

### Plugins to prioritize

Keep enabled:
- `Figma`
- `GitHub`
- `Notion`

Helpful but optional:
- `Google Drive`

Lower priority for this repo:
- `Gmail`
- `Google Calendar`
- `Canva`
- `Dropbox`
- `Slack`

### Skills to prioritize

High-value existing skills:
- `figma-use`
- `figma-generate-design`
- `playwright`
- `smart-commit`
- `openai-docs`

Candidate repo-specific custom skills:
- `diamond-runner-implementation`
  - repo architecture, phases, file contract, and IDE-shell conventions
- `diamond-project-fixture`
  - creating and validating `fixtures/sample-project/`
- `diamond-prompt-pack-port`
  - adapting prompt packs from `workshop-parts/`

## Documentation Placement

This topic should be documented in three layers:

- `AGENTS.md`
  - short policy for tool choice, commit triggers, and push behavior
- `README.md`
  - short operator-facing setup guidance
- this plan document
  - detailed audit, best-use-case map, and settings recommendations

`TODO.md` should keep only actionable follow-ups rather than policy text.
