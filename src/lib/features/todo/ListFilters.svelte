<script lang="ts">
  import { t } from "../../i18n";
  import {
    isFiltering,
    NO_FILTER,
    type DateFilter,
    type ListFilter,
  } from "../../stores/listFilter";

  // The filter bar of «Tareas» (F20-05).
  interface Props {
    filter: ListFilter;
    onChange: (filter: ListFilter) => void;
  }

  let { filter, onChange }: Props = $props();
  const l = t.listFilter;

  const DATES: DateFilter[] = ["all", "undated", "overdue", "today", "week"];
  const filtering = $derived(isFiltering(filter));
</script>

<div class="filters" role="group" aria-label={l.label}>
  <div class="dates" role="group" aria-label={l.date}>
    {#each DATES as date (date)}
      <button
        type="button"
        class="toggle-chip"
        aria-pressed={filter.date === date}
        onclick={() => onChange({ ...filter, date })}
      >
        {l.dates[date]}
      </button>
    {/each}
  </div>
  <select
    aria-label={l.priority}
    value={String(filter.minPriority)}
    onchange={(event) =>
      onChange({ ...filter, minPriority: Number(event.currentTarget.value) })}
  >
    {#each l.priorities as name, level (level)}
      <option value={String(level)}>{name}</option>
    {/each}
  </select>
  <button
    type="button"
    class="toggle-chip"
    aria-pressed={filter.showCompleted}
    onclick={() =>
      onChange({ ...filter, showCompleted: !filter.showCompleted })}
  >
    {l.completed}
  </button>
  {#if filtering}
    <button
      type="button"
      class="btn small quiet"
      onclick={() => onChange(NO_FILTER)}
    >
      {l.clear}
    </button>
  {/if}
</div>

<style>
  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3) var(--space-2);
  }

  .dates {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  /* Outside a .ui-form the select is native: it takes the look of the chips beside it. */
  select {
    width: auto;
    height: 26px;
    padding: 0 var(--space-2) 0 var(--space-3);
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-caption);
    font-weight: 500;
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  select:hover {
    background: var(--bg-control-hover);
  }

  select option {
    background: var(--bg-popover);
    color: var(--text-primary);
  }
</style>
