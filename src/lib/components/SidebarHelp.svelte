<script lang="ts">
  interface HelpTopic {
    title: string;
    description: string;
    key: string;
  }

  interface Props {
    onOpenHelp: (key: string) => void;
  }

  let { onOpenHelp }: Props = $props();

  const topics: HelpTopic[] = [
    { key: 'tera-syntax', title: 'Tera Template Syntax', description: 'Variables, filters, control flow, and the doc() function' },
    { key: 'prompt-blocks', title: 'Prompt Blocks', description: 'Registering blocks, model overrides, and output targets' },
    { key: 'pipelines', title: 'Pipelines', description: 'Chaining blocks into multi-step workflows' },
    { key: 'variables', title: 'Variables', description: 'Global and project-level variable assignment' },
    { key: 'model-presets', title: 'Model Presets', description: 'Creating and managing YAML model configurations' },
    { key: 'export-bundles', title: 'Export Bundles', description: 'Packaging selected assets for sharing' },
    { key: 'keyboard-shortcuts', title: 'Keyboard Shortcuts', description: 'Navigation, editing, and execution shortcuts' }
  ];
</script>

<div class="sidebar-help">
  {#each topics as topic (topic.key)}
    <button type="button" class="help-card" onclick={() => onOpenHelp(topic.key)}>
      <strong>{topic.title}</strong>
      <p class="desc">{topic.description}</p>
    </button>
  {/each}
</div>

<style>
  .sidebar-help {
    display: grid;
    gap: 0.35rem;
  }

  .help-card {
    display: grid;
    gap: 0.15rem;
    padding: 0.55rem 0.65rem;
    border-radius: 12px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .help-card:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(157, 180, 255, 0.2);
  }

  .desc {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.78rem;
  }
</style>
