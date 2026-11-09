<script lang="ts">
  import { t } from "../../i18n";
  import type { Tag } from "../../types";
  import TagChip from "./TagChip.svelte";

  interface Props {
    tags: Tag[];
    value: string | null;
    onChange: (tagId: string | null) => void;
  }

  let { tags, value, onChange }: Props = $props();
</script>

<div class="picker" role="radiogroup" aria-label={t.tags.taskLabel}>
  <button
    type="button"
    role="radio"
    class="option"
    class:selected={value === null}
    aria-checked={value === null}
    onclick={() => onChange(null)}
  >
    {t.tags.noneOption}
  </button>
  {#each tags as tag (tag.id)}
    <button
      type="button"
      role="radio"
      class="option"
      class:selected={value === tag.id}
      aria-checked={value === tag.id}
      style:--tag={tag.colorHex}
      onclick={() => onChange(tag.id)}
    >
      <TagChip name={tag.name} color={tag.colorHex} />
    </button>
  {/each}
</div>
{#if tags.length === 0}
  <p class="hint">{t.tags.createHint}</p>
{/if}

<style>
  .picker {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .option {
    display: inline-flex;
    align-items: center;
    padding: 2px 6px;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 11px;
    line-height: 16px;
    cursor: pointer;
  }

  .option:hover {
    background: var(--bg-hover);
  }

  .option.selected {
    border-color: var(--tag, var(--accent));
    box-shadow: 0 0 0 1px var(--tag, var(--accent));
    color: var(--text-primary);
  }

  .option:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .hint {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 11px;
  }
</style>
