---
created: 2026-04-03 9:12 PM
modified: 2026-04-03 9:12 PM
---
# Plan To Reach The Online Research Slice

## Summary

The local-first execution path is now stable through prompt runs, pipelines, run history, pipeline authoring, and export bundles. The next narrow extension slice is scoped online research for prompts that explicitly opt into live web-backed execution.

This slice should prove the first online-enabled prompt workflow without turning online mode into a global app toggle:
- enable live web-backed execution only for prompt runs that explicitly opt into online behavior
- validate the OpenRouter online transport path with the standalone `npm run probe:online` script before app runtime behavior is hardcoded
- preserve Diamond's local-first default by keeping non-online prompt execution unchanged
- surface enough run metadata to confirm that web search actually occurred
- keep the first UI pass narrow and avoid broad provider-settings sprawl

This slice does not include broad advanced provider controls, multi-provider abstractions, pipeline-level online orchestration policies, or help-document polish beyond the minimum notes needed for the implemented behavior.

## Scope

Included in this slice:
- define the first scoped rule for when a prompt is considered online-enabled
- prove the OpenRouter online transport path first through the standalone OpenAI SDK probe against real requests
- wire the validated online request shape into app execution only after the probe path succeeds
- keep standard prompt execution unchanged for prompts that do not opt in
- persist enough run metadata to distinguish online-enabled executions from normal runs
- add backend and frontend tests covering the scoped online behavior
- document the narrow rules and constraints for the first online pass

Explicitly out of scope for this slice:
- global online mode toggles across the whole app
- broad OpenRouter controls UI for engine selection, domain filtering, context size, healing, or structured output tuning
- arbitrary server-tool orchestration beyond web search
- pipeline-specific online policies beyond reusing the existing per-block execution path
- replacing the entire local execution system with a Node-side runtime

## Proposed First-Pass Behavior

- Treat online research as an opt-in execution mode for selected prompts, not a workspace-wide setting.
- Keep the first trigger simple and explicit. The current candidate is prompt-level opt-in rather than hidden heuristics.
- Use OpenRouter's online path in a constrained way:
  - default model suffix or plugin configuration that triggers web search
  - conservative defaults for search breadth
  - no advanced UI controls in the first pass
- Persist run metadata that makes the online path auditable, such as whether the run used online mode and whether web-search requests or citations were returned.

## Probe-First Rule

Before app runtime behavior is hardcoded:
- prove the target request shape through `npm run probe:online`
- use a real prompt with web-backed output and citations
- confirm that the response includes evidence of online execution, such as citations or `web_search_requests`
- only then port the validated request contract into the app execution path

This rule exists because OpenRouter online behavior is provider-specific enough that guessing at the payload in app code is higher risk than testing it first.

## Candidate Technical Direction

- Keep the app's real execution boundary in Rust.
- Use the standalone Node/OpenAI SDK probe as the contract-validation harness, not as the long-term app runtime.
- Port only the proven online request shape into the backend execution layer after the probe is passing.
- Reuse the existing run artifact and execution-result patterns rather than inventing a second online-only artifact type.
- Keep the first UI touchpoint narrow, likely attached to prompt execution state rather than a new settings surface.

## Follow-Up Checklist

- [ ] Lock the first online opt-in rule for prompt execution.
- [ ] Keep `npm run probe:online` passing with a real OpenRouter online request.
- [ ] Add backend execution support for the scoped online request path.
- [ ] Persist online-specific run metadata needed for auditability.
- [ ] Add frontend affordance for the scoped online-enabled prompt flow.
- [ ] Add backend and frontend automated coverage for online-enabled execution.
- [ ] Add minimum README, roadmap, and Help notes for the implemented behavior.

## Verification

- Non-online prompt execution must remain unchanged.
- Online-enabled prompt runs must only activate web-backed behavior when the prompt explicitly opts in.
- The online request path must be proven first through `npm run probe:online` before the app hardcodes the behavior.
- Persisted run metadata must make online runs distinguishable from standard runs.
- This slice must pass:
  - `npm run probe:online`
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- The app stays local-first by default; online research is an explicit extension path.
- The standalone OpenAI SDK probe is the proof harness for the request shape, not the final authority for app architecture.
- The first online slice should prefer the narrowest implementation that yields a real end-to-end result.