<script lang="ts">
  import { t } from "../../i18n";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { EventOccurrence } from "../../types";
  import { timeRangeOf } from "./occurrences";

  // One occurrence in a list (the events view, «Hoy» and «Próximas»). Only what it shows is
  // passed in, so a change elsewhere does not redraw it (D-98).
  interface Props {
    occurrence: EventOccurrence;
    /** Name the date as well, for lists that do not group by day. */
    dateLabel?: string;
    onOpen: (eventId: string) => void;
  }

  let { occurrence, dateLabel = "", onOpen }: Props = $props();

  const tag = $derived(tagsStore.tagOf(occurrence));
  const name = $derived(occurrence.title || t.events.untitled);
  const time = $derived(timeRangeOf(occurrence) ?? t.events.allDay);
</script>

<li class="row" data-event-id={occurrence.eventId}>
  <span class="bar" style:--tag={tag?.colorHex} aria-hidden="true"></span>
  <button
    type="button"
    class="open"
    title={t.events.open(name)}
    onclick={() => onOpen(occurrence.eventId)}
  >
    <span class="title">{name}</span>
    {#if occurrence.recurring}
      <span class="repeat">{t.events.repeats}</span>
    {/if}
    <span class="when">{dateLabel} {time}</span>
  </button>
</li>

<style>
  .row {
    position: relative;
    display: flex;
    align-items: stretch;
    padding-left: var(--space-3);
    transition: background-color var(--motion-instant) linear;
  }

  .row:not(:first-child)::before {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    left: var(--space-3);
    height: 1px;
    background: var(--separator);
  }

  .row:hover {
    background: var(--bg-hover);
  }

  .bar {
    flex: none;
    align-self: center;
    width: 4px;
    height: 20px;
    border-radius: var(--radius-chip);
    background: var(--tag, var(--accent));
  }

  .open {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
    min-height: 40px;
    padding: 6px var(--space-3) 6px 10px;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    border-radius: var(--radius-control);
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .repeat {
    flex: none;
    color: var(--text-secondary);
    font-size: 11px;
  }

  .when {
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
</style>
