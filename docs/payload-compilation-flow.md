# Payload Compilation Flow

## Overview Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          USER-OPERATED INPUTS                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
                    ▼                 ▼                 ▼
         ┌──────────────────┐  ┌──────────┐  ┌─────────────────┐
         │   DOCUMENTS/     │  │ MODELS/  │  │   PROMPTS/      │
         │   *.md files     │  │ *.yaml   │  │   *.tera files  │
         └──────────────────┘  └──────────┘  └─────────────────┘
                    │                 │                 │
                    │                 │                 │
         ┌──────────────────┐         │      ┌─────────────────┐
         │   VARIABLES      │         │      │   PIPELINES/    │
         │ (3 sources)      │         │      │   *.json files  │
         └──────────────────┘         │      └─────────────────┘
                    │                 │                 │
                    └─────────────────┼─────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        TEMPLATE CONTEXT PREPARATION                          │
│                     prepare_template_context()                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
                    ▼                 ▼                 ▼
         ┌──────────────────┐  ┌──────────┐  ┌─────────────────┐
         │ Resolve doc()    │  │ Merge    │  │ Add metadata    │
         │ references       │  │ variables│  │ (project, time) │
         │ → bindings       │  │ → context│  │ → context       │
         └──────────────────┘  └──────────┘  └─────────────────┘
                    │                 │                 │
                    └─────────────────┼─────────────────┘
                                      │
                                      ▼
                          ┌───────────────────────┐
                          │   TERA CONTEXT        │
                          │   ─────────────       │
                          │   • project           │
                          │   • model_id          │
                          │   • now_iso           │
                          │   • current_date      │
                          │   • variables         │
                          │   • global_variables  │
                          │   • <var_name>        │
                          │   • diamond_doc_ref_* │
                          └───────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          TEMPLATE RENDERING                                  │
│                   render_template_for_execution()                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
                          ┌───────────────────────┐
                          │   RENDERED PROMPT     │
                          │   (plain text)        │
                          └───────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
                    ▼                 ▼                 ▼
         ┌──────────────────┐  ┌──────────┐  ┌─────────────────┐
         │ Check for        │  │ Load     │  │ Determine       │
         │ {# diamond:      │  │ model    │  │ model_id        │
         │ online #}        │  │ preset   │  │ (+ :online?)    │
         └──────────────────┘  └──────────┘  └─────────────────┘
                    │                 │                 │
                    └─────────────────┼─────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PAYLOAD COMPILATION                                  │
│                     build_openrouter_payload()                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                          ┌───────────┴───────────┐
                          │                       │
                          ▼                       ▼
              ┌─────────────────────┐   ┌─────────────────────┐
              │ BASE PAYLOAD        │   │ CONDITIONAL         │
              │ (from model preset) │   │ (if online_enabled) │
              │                     │   │                     │
              │ 1. Insert "model"   │   │ 4. Insert "plugins" │
              │ 2. Insert "messages"│   │ 5. Insert           │
              │    [{ role, content}]   │    "web_search_     │
              │ 3. Keep preset      │   │    options"         │
              │    fields (temp,    │   │                     │
              │    max_tokens, etc) │   │                     │
              └─────────────────────┘   └─────────────────────┘
                          │                       │
                          └───────────┬───────────┘
                                      │
                                      ▼
                          ┌───────────────────────┐
                          │   FINAL API PAYLOAD   │
                          │   (JSON Value)        │
                          └───────────────────────┘
                                      │
                                      ▼
                          ┌───────────────────────┐
                          │   POST to OpenRouter  │
                          │   /chat/completions   │
                          └───────────────────────┘
```

## Variable Priority Cascade

```
┌─────────────────────────────────────────────────────────────────┐
│                    VARIABLE RESOLUTION ORDER                     │
│                    (Higher Priority Wins)                        │
└─────────────────────────────────────────────────────────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
                ▼               ▼               ▼
    ┌──────────────────┐ ┌─────────────┐ ┌──────────────┐
    │ 1. GLOBAL        │ │ 2. PROJECT  │ │ 3. PAYLOAD   │
    │    VARIABLES     │ │    VARIABLES│ │    VARIABLES │
    │                  │ │             │ │              │
    │ Source:          │ │ Source:     │ │ Source:      │
    │ app_data_dir/    │ │ variables.  │ │ Runtime      │
    │ global_          │ │ yaml OR     │ │ BTreeMap     │
    │ variables.json   │ │ project.json│ │ parameter    │
    │                  │ │ .variables  │ │              │
    │ Scope: App-wide  │ │ Scope:      │ │ Scope:       │
    │                  │ │ Project     │ │ Execution    │
    └──────────────────┘ └─────────────┘ └──────────────┘
                │               │               │
                └───────────────┼───────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │   MERGED INTO TERA    │
                    │   CONTEXT             │
                    │                       │
                    │   Same key?           │
                    │   → Higher priority   │
                    │     overwrites        │
                    └───────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │   RESOLVED IN         │
                    │   TEMPLATE RENDERING  │
                    │                       │
                    │   {{ variable_name }} │
                    │   becomes value       │
                    └───────────────────────┘
```

## Document Reference Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROMPT TEMPLATE (.tera)                       │
│                                                                  │
│   Here is the story context:                                    │
│   {{ doc("character-profiles.md") }}                            │
│                                                                  │
│   And the outline:                                              │
│   {{ doc("chapter-outline.md") }}                               │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                ┌───────────────────────────────┐
                │ preprocess_doc_references()   │
                │                               │
                │ Regex: doc\(\s*"([^"]+)"\s*\) │
                └───────────────────────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
                ▼               ▼               ▼
    ┌──────────────────┐ ┌─────────────┐ ┌──────────────┐
    │ Read:            │ │ Read:       │ │ Generate     │
    │ documents/       │ │ documents/  │ │ binding      │
    │ character-       │ │ chapter-    │ │ names:       │
    │ profiles.md      │ │ outline.md  │ │              │
    │                  │ │             │ │ diamond_doc_ │
    │ → content        │ │ → content   │ │ ref_0        │
    │                  │ │             │ │ diamond_doc_ │
    │                  │ │             │ │ ref_1        │
    └──────────────────┘ └─────────────┘ └──────────────┘
                │               │               │
                └───────────────┼───────────────┘
                                │
                                ▼
                ┌───────────────────────────────┐
                │ PREPROCESSED TEMPLATE:        │
                │                               │
                │ Here is the story context:    │
                │ {{ diamond_doc_ref_0 }}       │
                │                               │
                │ And the outline:              │
                │ {{ diamond_doc_ref_1 }}       │
                │                               │
                │ + Bindings added to context   │
                └───────────────────────────────┘
                                │
                                ▼
                ┌───────────────────────────────┐
                │ RENDERED PROMPT:              │
                │                               │
                │ Here is the story context:    │
                │ [full character profile text] │
                │                               │
                │ And the outline:              │
                │ [full chapter outline text]   │
                └───────────────────────────────┘
```

## Pipeline Orchestration

```
┌─────────────────────────────────────────────────────────────────┐
│                    PIPELINE EXECUTION                            │
│                    (pipelines/my-pipeline.json)                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                ┌───────────────────────────────┐
                │ ordered_blocks:               │
                │ ["block-1", "block-2", ...]   │
                └───────────────────────────────┘
                                │
                                ▼
                ┌───────────────────────────────┐
                │ Sequential Execution Loop     │
                └───────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌──────────────┐        ┌──────────────┐        ┌──────────────┐
│ BLOCK 1      │        │ BLOCK 2      │        │ BLOCK 3      │
│              │        │              │        │              │
│ Full payload │        │ Full payload │        │ Full payload │
│ compilation  │        │ compilation  │        │ compilation  │
│ (as above)   │        │ (as above)   │        │ (as above)   │
│              │        │              │        │              │
│ ↓            │        │ ↓            │        │ ↓            │
│ API call     │   →    │ API call     │   →    │ API call     │
│ ↓            │        │ ↓            │        │ ↓            │
│ Persist run  │        │ Persist run  │        │ Persist run  │
└──────────────┘        └──────────────┘        └──────────────┘
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                │
                                ▼
                ┌───────────────────────────────┐
                │ PipelineExecutionResult       │
                │                               │
                │ • steps: [result1, result2..] │
                │ • status: Success/Failed      │
                │ • aggregate metrics           │
                └───────────────────────────────┘
```

## Final Payload Structure

```json
{
  // From model preset YAML (base config)
  "temperature": 0.7,
  "max_tokens": 2000,
  "top_p": 1.0,
  "frequency_penalty": 0.0,
  "presence_penalty": 0.0,
  
  // Inserted by build_openrouter_payload (step 1)
  "model": "anthropic/claude-3.5-sonnet:online",
  
  // Inserted by build_openrouter_payload (step 2)
  "messages": [
    {
      "role": "user",
      "content": "<fully rendered prompt with variables and documents resolved>"
    }
  ],
  
  // Inserted conditionally if online_enabled (steps 4-5)
  "plugins": [
    {
      "id": "web",
      "max_results": 3
    }
  ],
  "web_search_options": {
    "search_context_size": "medium"
  }
}
```

## Key Insights

1. **Documents** and **Variables** are compiled into the prompt text **before** payload assembly
2. **Models** provide the payload structure (all preset fields carry through)
3. **Prompts** are the content (rendered to `messages[0].content`)
4. **Pipelines** orchestrate multiple independent payloads (not payload modification)
5. Variable priority: Payload > Project > Global (higher overwrites lower)
6. Online mode is detected from prompt directive and modifies both model ID and payload structure
