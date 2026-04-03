---
title: Specification for Diamond Prompt Runner v1
created: 2024-04-03 10:48 AM
modified: 2024-04-03 10:48 AM
---

# 1. 🗂️ Project \& Workflow Model

- each project have its own set of documents, templates, variables, and workflow graph — like a workspace
- Views of the workflow: a visual node graph (think: Obsidian Canvas or n8n-style), a linear pipeline (sequential steps), and list of runnable prompt blocks you execute manually one at a time or prompt x to prompt y

2. 📄 Documents \& Templates

- The templates on the system side are Tera. They are viewable and editable to the user as a document/form with buttons to reuse variables.
- The user can create new variables tied to document names. They can type in a document name if it does not exist yet as `{{document_name.txt}}` or whatever the Tera syntax is.
- WYSIWYG editor where I can toggle/edit in a markdown view

3. 🔁 Loop / Chapter Generation

- The user can define the N value to start with the number of loops for chapters. So it would be something like "Write the Chapter Plan for Chapter {{n-chapter}} and the output would be something like `Chapter_1_Plan.md".
- Loops should support sequential chaining. For example with generating a story concept, you might start with a brainstorm story ideas prompt, then use that output prompt for a pros and cons of each concept in comparison to a previously generated genre conventions guide, then select a winning idea, then develop that idea into a fleshed out hook, premise, and pitch document.

4. 🤖 AI \& Streaming

- The app support multiple concurrent streams. For example, one could do research on a genre while another does research on terminology for a specific profession.
- Just OpenRouter, with toggle options to support OpenRouter features like Structured Output, Healing, Online, and Reasoning.

5. 🎨 Visual Style

- Obsidian (Dark/light with accent colors; dense, panel-heavy interface)
- VS Code (utilitarian, activity-bar-driven layout)
- n8n (node driven workflows)
- Color preference: Warm dark (like Bear), minimal accent colors
- Name is "Diamond Prompt Runner" and the icon is a diamond

6. 💾 Storage \& Persistence

- Projects stored locally on disk (Tauri's filesystem) as folders with JSON/Markdown files
- Projects should be exportable (e.g., zip of documents + templates)

7. 📦 MVP vs. Full Vision

- For v2: wysiwig editor; node graph;  toggle options to support OpenRouter features like Structured Output, Healing, Online, and Reasoning; toast notifications
- I will have some prompts already designed, so we can start with those and make sure they work and are easily reconfigurable by the user.

Here's the full spec, ready to hand off to any AI for mockup generation.

***

## What's in the Doc

The spec covers 11 sections — here are the highlights worth noting:

**Key decisions I made and why:**

- **Tera instead of Jinja2** — since you're on Tauri/Rust, Tera runs natively in the backend as a Rust crate. It's Jinja2-compatible syntax so your existing prompts transfer directly, and it resolves in the Rust layer before the API call (cleaner, faster, no Python subprocess)
- **Document references as first-class variables** — `{{ Genre_Conventions.md }}` resolves to full file content at runtime, stored as a variable type in `project.json`. Unresolved refs show amber warnings, not crashes
- **Concurrent streams as v1** — since you mentioned research tasks running in parallel, I kept this in MVP rather than pushing to v2
- **`[INCOMPLETE]` header on interrupted streams** — partial outputs are still saved; you don't lose work if a stream drops

**What's deferred to v2 (per your guidance):**

- WYSIWYG editor
- Node graph view
- OpenRouter feature toggles (Structured Output, Healing, Online, Reasoning)
- Toast notifications

**The 7 priority mockup screens** at the end of the doc are formatted specifically to give an AI renderer a clear sequence — most complex/representative screens first.

***

**Next natural steps when you're ready:**

1. Hand this spec to an AI (Claude, Cursor, etc.) to generate Figma-style HTML mockups of the 7 screens
2. Define your starter template prompts so they can be bundled as preloads
3. Scaffold the Tauri + Svelte project structure
