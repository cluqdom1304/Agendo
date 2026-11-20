<script lang="ts">
  import { t } from "../../i18n";
  import { tagsStore } from "../../stores/tags.svelte";
  import TagChip from "../tags/TagChip.svelte";
  import Checkbox from "./Checkbox.svelte";
  import ChecklistProgress from "./ChecklistProgress.svelte";

  // One task in the short lists (Hoy, Próximas, Bandeja), as the reference draws them: the
  // checkbox and title on one line, and under it what places the task — day, hour, tags. Only the
  // fields it shows are passed in, so a change elsewhere does not redraw it (D-98).
  interface Props {
    title: string;
    priority: number;
    completed?: boolean;
    checklistDone?: number;
    checklistTotal?: number;
    /** Already formatted day ("jue 12 mar"), or empty to leave it out. */
    dateLabel?: string;
    time?: string | null;
    /** Past its day: the day and hour turn red. */
    overdue?: boolean;
    tagIds?: string[];
    /** Accessible name of the checkbox. */
    checkLabel: string;
    onToggle: () => void;
    onOpen: () => void;
  }

  let {
    title,
    priority,
    completed = false,
    checklistDone = 0,
    checklistTotal = 0,
    dateLabel = "",
    time = null,
    overdue = false,
    tagIds = [],
    checkLabel,
    onToggle,
    onOpen,
  }: Props = $props();

  const tags = $derived(tagsStore.tagsOf({ tagIds }).slice(0, 2));
  const hasMeta = $derived(
    dateLabel !== "" || !!time || tags.length > 0 || checklistTotal > 0,
  );
</script>

<div class="line" class:done={completed}>
  <Checkbox checked={completed} label={checkLabel} {priority} {onToggle} />
  <button type="button" class="open" onclick={onOpen}>
    <span class="title">{title || t.reminders.untitled}</span>
    {#if hasMeta}
      <span class="meta">
        {#if dateLabel}<span class="when" class:late={overdue}>{dateLabel}</span
          >{/if}
        {#if time}<span class="when time" class:late={overdue}>{time}</span
          >{/if}
        {#if checklistTotal > 0}
          <ChecklistProgress done={checklistDone} total={checklistTotal} />
        {/if}
        {#each tags as tag (tag.id)}
          <TagChip name={tag.name} color={tag.colorHex} />
        {/each}
      </span>
    {/if}
  </button>
</div>

<style>
  .line {
    display: flex;
    flex: 1;
    align-items: flex-start;
    gap: 10px;
    min-width: 0;
  }

  /* Lines the checkbox up with the first line of the title. */
  .line > :global(:first-child) {
    margin-top: 1px;
  }

  .open {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-body);
    text-align: left;
    cursor: pointer;
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: var(--radius-small);
  }

  .title {
    overflow: hidden;
    font-weight: 600;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color var(--motion-base) var(--ease-out);
  }

  .done .title {
    color: var(--text-secondary);
    font-weight: 400;
    text-decoration: line-through;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 8px;
    min-width: 0;
    color: var(--text-secondary);
    font-size: var(--text-caption);
    line-height: 18px;
  }

  .when {
    flex: none;
  }

  .time {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .late {
    color: var(--danger);
  }
</style>
