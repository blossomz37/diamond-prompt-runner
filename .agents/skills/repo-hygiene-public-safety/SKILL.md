# SKILL: Repo Hygiene and Public Safety Cleanup

## Purpose
A repeatable workflow for cleaning up a codebase before making it public or sharing with collaborators. Ensures no secrets, personal info, or unsafe patterns remain, and that documentation is accurate and public-ready.

## When to Use
- Before open-sourcing a repo
- Prior to sharing code with external collaborators
- As a periodic hygiene pass for active projects
- After major refactors or onboarding new contributors

## Steps

1. **Move Ad Hoc Scripts and Utilities**
   - Relocate one-off scripts (e.g., migration, test, or debug scripts) to a dedicated `scripts/` or `scripts/dev/` directory.
   - Update or create `scripts/README.md` to document their purpose.

2. **Tighten Ignore Rules**
   - Update `.gitignore` to exclude:
     - Local config files (e.g., `.env`, `.DS_Store`, editor/project settings)
     - Prompt scratch files, temp output, and local-only artifacts
   - Validate with `git status` that no local config or secrets are tracked.

3. **Normalize Personal Paths in Docs**
   - Replace absolute or user-specific paths (e.g., `/Users/username/...`) with repo-relative or generic placeholders in all documentation and planning files.
   - Use search/replace tools (e.g., `grep`, `rg`, VS Code search) to find and update all instances.

4. **Scan for Hardcoded Secrets and Sensitive Info**
   - Search for API keys, passwords, and sensitive tokens in all tracked files:
     - Use `git grep` or `rg` for patterns like `password`, `API_KEY`, `SECRET`, and known example values.
   - Remove or replace with placeholders (e.g., `YOUR_API_KEY_HERE`).
   - Ensure `.env` and similar files are ignored and not tracked.

5. **Review and Polish Documentation**
   - Rewrite `README.md` for scanability and accuracy:
     - Add a screenshot if possible.
     - Use collapsible sections for long docs.
     - Ensure all setup, build, and usage instructions are current.
   - Scrub any public doc or workflow of real or example secrets (e.g., passwords, signing keys).
   - Replace explicit secrets with generic/placeholder references.

6. **Validate and Commit**
   - Use `git status` and `git diff` to confirm only intended changes.
   - Run a final search for secrets and personal info.
   - Commit with a clear message (e.g., `Hygiene: repo cleanup and public safety pass`).
   - Push changes if ready for public or shared use.

## Notes
- Even doc comments and example workflows can leak secrets—always use placeholders.
- Prefer generic, non-identifying language in all public-facing docs.
- For sensitive workflows (e.g., release signing), never publish real passwords or keys—use environment variables and document the process generically.

## Example Commit Message
```
Hygiene: repo cleanup and public safety pass
- Move ad hoc scripts to scripts/dev/
- Tighten .gitignore for local config and scratch files
- Normalize personal paths in docs
- Remove all hardcoded secrets and example passwords
- Polish README and docs for public safety
```

---

**Crystallized from: Diamond Runner repo hygiene and public safety cleanup, April 2026.**
