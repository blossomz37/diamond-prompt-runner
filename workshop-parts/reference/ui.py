#!/usr/bin/env python3
# ──────────────────────────────────────────────
# File:        ui.py
# Description: Minimal local web UI for the YFD runner (read-only browser)
# Version:     1.0.0
# Created:     2026-03-31
# Modified:    2026-03-31
# Author:      carlo
# ──────────────────────────────────────────────
from __future__ import annotations

import json
from pathlib import Path

from flask import Flask, abort, render_template_string, request

import state as state_module

BASE_DIR = Path(__file__).resolve().parent
STATE_DIR = BASE_DIR / "state"
OUTPUT_DIR = BASE_DIR / "output"

CHAPTER_STEPS = ["plan", "draft", "repetition_audit", "style", "craft", "final", "summary"]

app = Flask(__name__)

# ── helpers ───────────────────────────────────

def list_runs() -> list[str]:
    return sorted(
        p.stem for p in STATE_DIR.glob("*.json") if not p.stem.startswith(".")
    )

def load(run_id: str) -> dict:
    try:
        return state_module.load_state(run_id)
    except state_module.StateError:
        abort(404, f"Run not found: {run_id}")

# ── templates ─────────────────────────────────

BASE_HTML = """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>YFD Runner{% if title %} — {{ title }}{% endif %}</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; }
    body { font-family: system-ui, sans-serif; margin: 0; background: #f8f8f8; color: #222; }
    header { background: #1e1e2e; color: #cdd6f4; padding: 0.75rem 1.5rem; display: flex; align-items: center; gap: 1rem; }
    header h1 { margin: 0; font-size: 1.1rem; font-weight: 600; }
    header a { color: #cba6f7; text-decoration: none; font-size: 0.9rem; }
    header a:hover { text-decoration: underline; }
    .container { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
    h2 { font-size: 1rem; font-weight: 600; margin-top: 2rem; margin-bottom: 0.5rem; border-bottom: 1px solid #ddd; padding-bottom: 0.25rem; }
    table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
    th, td { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 1px solid #e0e0e0; }
    th { background: #f0f0f0; font-weight: 600; }
    a { color: #1a6fb8; }
    a:hover { text-decoration: underline; }
    .badge { display: inline-block; padding: 0.15em 0.5em; border-radius: 3px; font-size: 0.8em; font-weight: 600; }
    .badge-ok { background: #d4edda; color: #155724; }
    .badge-pending { background: #fff3cd; color: #856404; }
    .badge-missing { background: #f8d7da; color: #721c24; }
    .prose { background: #fff; border: 1px solid #ddd; padding: 1rem 1.25rem; border-radius: 4px; font-size: 0.9rem; line-height: 1.65; white-space: pre-wrap; word-break: break-word; max-height: 70vh; overflow-y: auto; }
    .mono { font-family: ui-monospace, monospace; font-size: 0.85rem; }
    .muted { color: #666; font-size: 0.85rem; }
    .nav-pills { display: flex; flex-wrap: wrap; gap: 0.4rem; margin-bottom: 1rem; }
    .nav-pills a { padding: 0.3em 0.75em; border-radius: 4px; background: #e8e8e8; color: #333; text-decoration: none; font-size: 0.85rem; }
    .nav-pills a.active, .nav-pills a:hover { background: #1a6fb8; color: #fff; }
    .run-list { display: flex; flex-direction: column; gap: 0.5rem; }
    .run-card { background: #fff; border: 1px solid #ddd; border-radius: 4px; padding: 0.75rem 1rem; }
    .run-card strong { font-size: 0.95rem; }
    .run-card .muted { margin-left: 0.5rem; }
  </style>
</head>
<body>
  <header>
    <h1>YFD Runner</h1>
    <a href="/">← All runs</a>
    {% if run_id %}<span style="color:#a6e3a1">{{ run_id }}</span>{% endif %}
  </header>
  <div class="container">
    {% block content %}{% endblock %}
  </div>
</body>
</html>
"""

INDEX_HTML = BASE_HTML.replace(
    "{% block content %}{% endblock %}",
    """
<h2>Runs</h2>
{% if runs %}
<div class="run-list">
  {% for run in runs %}
  <div class="run-card">
    <strong><a href="/run/{{ run.id }}">{{ run.id }}</a></strong>
    <span class="muted">{{ run.project }}</span>
    <span class="muted">·</span>
    <span class="muted badge badge-ok">{{ run.chapter_count }} chapter(s)</span>
    <span class="muted">· updated {{ run.updated_at }}</span>
    {% if run.manuscript %}
    <span class="muted">· <a href="/manuscript/{{ run.id }}">manuscript</a></span>
    {% endif %}
  </div>
  {% endfor %}
</div>
{% else %}
<p class="muted">No runs found in state/.</p>
{% endif %}
"""
)

RUN_HTML = BASE_HTML.replace(
    "{% block content %}{% endblock %}",
    """
<h2>Run: {{ run_id }}</h2>
<table>
  <tr><th>Project</th><td>{{ data.project }}</td></tr>
  <tr><th>Model config</th><td>{{ data.model_config or '(default)' }}</td></tr>
  <tr><th>Created</th><td>{{ data.created_at }}</td></tr>
  <tr><th>Updated</th><td>{{ data.updated_at }}</td></tr>
  <tr><th>Tokens in</th><td>{{ data.metrics.total_tokens_in }}</td></tr>
  <tr><th>Tokens out</th><td>{{ data.metrics.total_tokens_out }}</td></tr>
  <tr><th>Words</th><td>{{ data.metrics.total_word_count }}</td></tr>
  <tr><th>Cost</th><td>${{ "%.4f"|format(data.metrics.total_cost_usd) }}</td></tr>
</table>

<h2>Cascade Status</h2>
<table>
  <tr><th>Section</th><th>Status</th></tr>
  <tr><td>section_1_required_data_layer</td><td><span class="badge badge-ok">author-filled</span></td></tr>
  {% for key, status in cascade.items() %}
  <tr>
    <td><a href="/cascade/{{ run_id }}/{{ key }}">{{ key }}</a></td>
    <td>
      {% if status == 'complete' %}
        <span class="badge badge-ok">complete</span>
      {% else %}
        <span class="badge badge-pending">pending</span>
      {% endif %}
    </td>
  </tr>
  {% endfor %}
</table>

<h2>Chapters</h2>
{% if chapters %}
<table>
  <tr><th>Chapter</th>{% for s in steps %}<th>{{ s }}</th>{% endfor %}</tr>
  {% for ch in chapters %}
  <tr>
    <td><a href="/chapter/{{ run_id }}/{{ ch.number }}"><strong>Ch {{ ch.number }}</strong></a></td>
    {% for s in steps %}
    <td>
      {% if ch.steps[s] %}
        <a href="/step/{{ run_id }}/{{ ch.number }}/{{ s }}"><span class="badge badge-ok">✓</span></a>
      {% else %}
        <span class="badge badge-missing">–</span>
      {% endif %}
    </td>
    {% endfor %}
  </tr>
  {% endfor %}
</table>
{% else %}
<p class="muted">No chapters yet.</p>
{% endif %}

{% if manuscript_exists %}
<h2>Manuscript</h2>
<p><a href="/manuscript/{{ run_id }}">View combined manuscript →</a></p>
{% endif %}
"""
)

STEP_HTML = BASE_HTML.replace(
    "{% block content %}{% endblock %}",
    """
<div class="nav-pills">
  <a href="/run/{{ run_id }}">← Run</a>
  <a href="/chapter/{{ run_id }}/{{ chapter }}">← Ch {{ chapter }}</a>
  {% for s in steps %}
  <a href="/step/{{ run_id }}/{{ chapter }}/{{ s }}" class="{{ 'active' if s == step else '' }}">{{ s }}</a>
  {% endfor %}
</div>
<h2>Ch {{ chapter }} / {{ step }}</h2>
<div class="prose mono">{{ content }}</div>
"""
)

CHAPTER_HTML = BASE_HTML.replace(
    "{% block content %}{% endblock %}",
    """
<div class="nav-pills">
  <a href="/run/{{ run_id }}">← Run</a>
  {% for s in steps %}
  {% if chapter_data[s] %}
  <a href="/step/{{ run_id }}/{{ chapter }}/{{ s }}">{{ s }}</a>
  {% endif %}
  {% endfor %}
</div>
<h2>Chapter {{ chapter }}</h2>
<table>
  <tr><th>Step</th><th>Words</th><th>Status</th></tr>
  {% for s in steps %}
  <tr>
    <td>
      {% if chapter_data[s] %}
        <a href="/step/{{ run_id }}/{{ chapter }}/{{ s }}">{{ s }}</a>
      {% else %}
        {{ s }}
      {% endif %}
    </td>
    <td>{{ word_counts[s] if chapter_data[s] else '–' }}</td>
    <td>
      {% if chapter_data[s] %}
        <span class="badge badge-ok">done</span>
      {% else %}
        <span class="badge badge-missing">missing</span>
      {% endif %}
    </td>
  </tr>
  {% endfor %}
</table>
"""
)

CASCADE_HTML = BASE_HTML.replace(
    "{% block content %}{% endblock %}",
    """
<div style="margin-bottom:0.75rem"><a href="/run/{{ run_id }}">← Run</a></div>
<h2>Cascade: {{ section_key }}</h2>
<div class="prose">{{ content }}</div>
"""
)

MANUSCRIPT_HTML = BASE_HTML.replace(
    "{% block content %}{% endblock %}",
    """
<div style="margin-bottom:0.75rem"><a href="/run/{{ run_id }}">← Run</a></div>
<h2>Manuscript: {{ run_id }}</h2>
<div class="prose">{{ content }}</div>
"""
)

# ── routes ────────────────────────────────────

@app.route("/")
def index():
    runs = []
    for run_id in list_runs():
        try:
            data = state_module.load_state(run_id)
        except state_module.StateError:
            continue
        manuscript_path = OUTPUT_DIR / f"{run_id}-manuscript.md"
        runs.append({
            "id": run_id,
            "project": data.get("project", "—"),
            "updated_at": data.get("updated_at", "—"),
            "chapter_count": len(data.get("chapters", {})),
            "manuscript": manuscript_path.exists(),
        })
    return render_template_string(INDEX_HTML, runs=runs, title=None, run_id=None)


@app.route("/run/<run_id>")
def run_view(run_id: str):
    data = load(run_id)
    chapters_raw = data.get("chapters", {})
    chapters = []
    for ch_num in sorted(int(k) for k in chapters_raw.keys()):
        ch_data = chapters_raw[str(ch_num)]
        chapters.append({
            "number": ch_num,
            "steps": {s: bool(ch_data.get(s)) for s in CHAPTER_STEPS},
        })
    cascade = state_module.get_cascade_status(run_id)
    manuscript_path = OUTPUT_DIR / f"{run_id}-manuscript.md"
    return render_template_string(
        RUN_HTML,
        run_id=run_id,
        data=data,
        chapters=chapters,
        steps=CHAPTER_STEPS,
        cascade=cascade,
        manuscript_exists=manuscript_path.exists(),
        title=run_id,
    )


@app.route("/chapter/<run_id>/<int:chapter>")
def chapter_view(run_id: str, chapter: int):
    data = load(run_id)
    ch_data = data.get("chapters", {}).get(str(chapter), {})
    import re
    word_pat = re.compile(r"\b\w+(?:['-]\w+)?\b")
    word_counts = {s: len(word_pat.findall(ch_data.get(s) or "")) for s in CHAPTER_STEPS}
    return render_template_string(
        CHAPTER_HTML,
        run_id=run_id,
        chapter=chapter,
        chapter_data=ch_data,
        steps=CHAPTER_STEPS,
        word_counts=word_counts,
        title=f"Ch {chapter}",
    )


@app.route("/step/<run_id>/<int:chapter>/<step>")
def step_view(run_id: str, chapter: int, step: str):
    data = load(run_id)
    ch_data = data.get("chapters", {}).get(str(chapter), {})
    content = ch_data.get(step)
    if content is None:
        abort(404, f"Step '{step}' not found for chapter {chapter}")
    return render_template_string(
        STEP_HTML,
        run_id=run_id,
        chapter=chapter,
        step=step,
        steps=CHAPTER_STEPS,
        content=content,
        title=f"Ch {chapter} / {step}",
    )


@app.route("/cascade/<run_id>/<section_key>")
def cascade_view(run_id: str, section_key: str):
    data = load(run_id)
    sections = state_module.parse_sections(data.get("worksheet", ""))
    section = next((s for s in sections if s["section_key"] == section_key), None)
    if section is None:
        abort(404, f"Section '{section_key}' not found")
    return render_template_string(
        CASCADE_HTML,
        run_id=run_id,
        section_key=section_key,
        content=section["text"],
        title=section_key,
    )


@app.route("/manuscript/<run_id>")
def manuscript_view(run_id: str):
    path = OUTPUT_DIR / f"{run_id}-manuscript.md"
    if not path.exists():
        abort(404, "Manuscript not built yet")
    content = path.read_text(encoding="utf-8")
    return render_template_string(
        MANUSCRIPT_HTML,
        run_id=run_id,
        content=content,
        title=f"{run_id} manuscript",
    )


if __name__ == "__main__":
    print("Starting YFD Runner UI at http://127.0.0.1:5050")
    app.run(debug=False, port=5050)
