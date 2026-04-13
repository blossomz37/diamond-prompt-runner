# Sample Project Patterns

Use these as adaptation templates, not as fixed canon.

## Sample Project 2: Loop Through A List
Directory:
- `projects/project-2/`

Use when:
- a user has a short list of prompts, beats, or options
- one output file should be produced per loop iteration

Core pattern:
- expose the list as a variable
- use `count` to choose the active item
- name outputs with `count`

Relevant files:
- `projects/project-2/project.json`
- `projects/project-2/prompts/flash-fiction.tera`

Copied reference bundle:
- `references/examples/flash-fiction-loop/`

Important manifest note:
- the example bundle includes a `project.json` because the prompt file alone is not the runnable unit
- the `variables` section exposes `prompt_options` and default `count`
- the prompt block connects `flash-fiction.tera` to the runner
- the payload works because the loop sends `count` and both the prompt and `outputFilename` consume that same variable

## Sample Project 3: Rolling Chapter Pipeline
Directory:
- `projects/project-3/`

Use when:
- a user wants planning, drafting, audits, editing, final chapters, and summaries
- the manuscript context must stay coherent without sending everything every turn

Core pattern:
- normalize the source dossier into wrappers
- extract only the needed dossier slices
- use summaries for older chapters
- use full drafts/chapters for recent context
- require stable h2 output headers across intermediate files

Relevant files:
- `projects/project-3/project.json`
- `projects/project-3/prompts/draft-chapter.tera`
- `projects/project-3/prompts/final.tera`
- `projects/project-3/documents/sequence.md`

Copied reference bundle:
- `references/examples/chapter-loop/`

## Sample Project 4: Cascading Dossier Build
Directory:
- `projects/project-4/`

Use when:
- a large worksheet/template must be completed section-by-section
- the output becomes a reusable dossier for later chapter loops

Core pattern:
- normalize the worksheet template into wrappers
- wrap to the deepest level needed downstream, including nested h3 units when required
- build one section at a time with a reusable section-builder prompt
- preserve nested wrappers exactly in the output
- keep tiny canon seeds in play if drift risk is high
- choose `append_document` or rebuild strategy deliberately

Relevant files:
- `projects/project-4/project.json`
- `projects/project-4/prompts/build-dossier-section.tera`
- `projects/project-4/documents/story-dossier-worksheet-template.sections.md`
- `projects/project-4/documents/cascade-worksheet.md`

Copied reference bundle:
- `references/examples/dossier-cascade/`
