# Tera Template Syntax

Diamond Runner uses the Tera template engine (similar to Jinja2) for prompt files.
This guide covers everything available inside `.tera` prompt templates.

---

## Delimiters

| Syntax       | Purpose                          |
|--------------|----------------------------------|
| `{{ }}`      | Output a value (expression)      |
| `{% %}`      | Control flow (if, for, set, etc.)|
| `{# #}`      | Comment (ignored at render time) |

---

## Available Context Variables

These are injected automatically when Diamond renders a template:

| Variable                 | Type   | Description                                |
|--------------------------|--------|--------------------------------------------|
| `project.name`           | string | The project name from project.json         |
| `project.id`             | string | The project UUID                           |
| `project.default_model_preset` | string | Path to default model preset        |
| `model_id`               | string | The model ID that will run this block      |
| `current_date`           | string | Today's date as `YYYY-MM-DD`               |
| `now_iso`                | string | Current UTC timestamp in ISO 8601          |

### User-Defined Variables

Any variable you set in **Variables** (global or project) becomes a top-level template variable.

```
{# If you set genre = "fantasy" in Variables: #}
Genre: {{ genre }}
```

Global variables are available via `global_variables.name`, and project variables via `variables.name`. But both are also promoted to top-level names, so `{{ genre }}` works directly. Project variables override globals of the same name.

---

## Expressions: `{{ }}`

Output any variable or computed value:

```
Project: {{ project.name }}
Date: {{ current_date }}
Model: {{ model_id }}
```

### Dot Notation and Brackets

```
{{ project.name }}
{{ project["name"] }}
```

### String Concatenation

Use `~` to join strings (not `+`):

```
{{ "Chapter " ~ chapter_number ~ ": " ~ chapter_title }}
```

---

## Inlining Documents: `doc()`

The `doc()` function reads a file from your project's `documents/` folder and inlines its full content:

```
{{ doc("worldbuilding.md") }}
```

This resolves to `documents/worldbuilding.md` in your project. Use it to inject reference material, character sheets, outlines, or any supporting document into a prompt.

### Subdirectories

Organize documents in subfolders and reference them with paths:

```
{{ doc("characters/protagonist.md") }}
{{ doc("worldbuilding/magic-system.md") }}
{{ doc("outline/act-1.md") }}
```

### Multiple Documents

Combine several documents in a single prompt:

```
## World Context
{{ doc("worldbuilding.md") }}

## Character Sheet
{{ doc("characters/elena.md") }}

## Scene Outline
{{ doc("outline/chapter-5.md") }}

Now write Chapter 5 based on the above context.
```

### What Happens When a Document Is Missing

- **Validation (preview):** Shows a warning but still renders: `[Missing document: filename.md]`
- **Execution (run):** Fails with an error — all `doc()` references must resolve

---

## Filters

Transform values with the `|` pipe. Filters chain left to right:

```
{{ name | upper }}
{{ biography | truncate(length=200) }}
{{ title | lower | replace(from=" ", to="-") }}
```

### Useful Filters

| Filter      | Example                                      | Result                |
|-------------|----------------------------------------------|-----------------------|
| `upper`     | `{{ "hello" \| upper }}`                     | `HELLO`               |
| `lower`     | `{{ "HELLO" \| lower }}`                     | `hello`               |
| `title`     | `{{ "dark forest" \| title }}`               | `Dark Forest`         |
| `trim`      | `{{ value \| trim }}`                        | strips whitespace     |
| `length`    | `{{ items \| length }}`                      | count of items        |
| `default`   | `{{ tone \| default(value="neutral") }}`     | fallback if undefined |
| `replace`   | `{{ text \| replace(from="X", to="Y") }}`   | find and replace      |
| `truncate`  | `{{ text \| truncate(length=100) }}`         | cut with ellipsis     |
| `wordcount` | `{{ text \| wordcount }}`                    | number of words       |
| `join`      | `{{ tags \| join(sep=", ") }}`               | array to string       |

---

## Control Flow: `{% %}`

### If / Elif / Else

```
{% if genre == "fantasy" %}
Include magic system rules.
{% elif genre == "thriller" %}
Include pacing guidelines.
{% else %}
Use general fiction defaults.
{% endif %}
```

### Checking if a Variable Exists

```
{% if character_name is defined %}
Focus character: {{ character_name }}
{% endif %}
```

### For Loops

Useful for iterating over lists if you set array-valued variables:

```
{% for chapter in chapters %}
{{ loop.index }}. {{ chapter }}
{% endfor %}
```

Loop variables:

| Variable       | Description                     |
|----------------|---------------------------------|
| `loop.index`   | Current iteration (1-based)     |
| `loop.index0`  | Current iteration (0-based)     |
| `loop.first`   | `true` on first iteration       |
| `loop.last`    | `true` on last iteration        |

Empty fallback:

```
{% for item in items %}
  {{ item }}
{% else %}
  No items provided.
{% endfor %}
```

---

## Assignments

Set local variables inside a template:

```
{% set word_target = 2000 %}
{% set scene_label = "Chapter " ~ chapter_number ~ ", Scene " ~ scene_number %}

Write {{ word_target }} words for {{ scene_label }}.
```

---

## Comments

Comments are stripped from the rendered output:

```
{# This note is for the author, not sent to the LLM #}
{# TODO: add character backstory doc reference #}
```

---

## Whitespace Control

Add `-` inside delimiters to trim surrounding whitespace:

```
{% set x = 42 -%}
Value: {{ x }}
```

Without `-%}`, there would be a blank line before `Value:`.

---

## Raw Blocks

Prevent Tera from parsing content (useful if you need literal `{{ }}` in output):

```
{% raw %}
Use {{ variable }} syntax in your templates.
{% endraw %}
```

---

## Typical Prompt Pattern

A complete prompt template pulling together variables, documents, and control flow:

```
{# Scene drafting prompt — Chapter {{ chapter_number }} #}
Project: {{ project.name }}
Date: {{ current_date }}
Target: {{ word_count | default(value="2000") }} words

## World Context
{{ doc("worldbuilding.md") }}

## Character Sheet
{{ doc("characters/protagonist.md") }}

{% if outline is defined %}
## Scene Outline
{{ outline }}
{% endif %}

## Instructions
Write Chapter {{ chapter_number }}, Scene {{ scene_number }}.
Genre: {{ genre | default(value="literary fiction") }}
POV: {{ pov | default(value="third person limited") }}
Tone: {{ tone | default(value="measured, observational") }}
```
