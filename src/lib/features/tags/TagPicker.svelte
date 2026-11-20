<script lang="ts">
  import { t } from "../../i18n";
  import type { Tag } from "../../types";
  import TagChip from "./TagChip.svelte";

  interface Props {
    tags: Tag[];
    /** The chosen tags; any number, including none. */
    values: string[];
    onChange: (tagIds: string[]) => void;
  }

  let { tags, values, onChange }: Props = $props();

  function toggle(id: string) {
    onChange(
      values.includes(id) ? values.filter((v) => v !== id) : [...values, id],
    );
  }
</script>

<div class="picker" role="group" aria-label={t.tags.taskLabel}>
  <button
    type="button"
    class="option"
    class:selected={values.length === 0}
    aria-pressed={values.length === 0}
    onclick={() => onChange([])}
  >
    {t.tags.noneOption}
  </button>
  {#each tags as tag (tag.id)}
    <button
      type="button"
      class="option"
      class:selected={values.includes(tag.id)}
      aria-pressed={values.includes(tag.id)}
      aria-label={tag.name}
      style:--tag={tag.colorHex}
      onclick={() => toggle(tag.id)}
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
    min-height: 24px;
    padding: 0 4px;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    box-shadow: inset 0 0 0 1px var(--border-subtle);
    color: var(--text-secondary);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      box-shadow var(--motion-fast) var(--ease-out),
      transform var(--motion-instant) var(--ease-out);
  }

  .option:first-child {
    padding: 0 10px;
  }

  .option:hover {
    background: var(--bg-hover);
  }

  .option:active {
    transform: scale(0.95);
  }

  .option.selected {
    box-shadow: inset 0 0 0 2px var(--tag, var(--accent));
    color: var(--text-primary);
  }

  .option:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .hint {
    margin: 4px 0 0;
    color: var(--text-secondary);
    font-size: 12px;
  }
</style>
