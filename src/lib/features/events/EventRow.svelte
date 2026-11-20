<script lang="ts">
  import { t } from "../../i18n";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { EventOccurrence } from "../../types";
  import { timeRangeOf } from "./occurrences";

  // One occurrence in a list (the events view, «Hoy» and «Próximas»). Only what it shows is
  // passed in, so a change elsewhere does not redraw it (D-98). Drawn as in the reference: a
  // coloured dot before the title and the hour under it; the list decides whether it is a card.
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
  <button
    type="button"
    class="open"
    title={t.events.open(name)}
    onclick={() => onOpen(occurrence.eventId)}
  >
    <span class="dot" style:--tag={tag?.colorHex} aria-hidden="true"></span>
    <span class="text">
      <span class="title">{name}</span>
      <span class="when">
        {dateLabel}
        {time}
        {#if occurrence.recurring}
          <span class="repeat">{t.events.repeats}</span>
        {/if}
      </span>
    </span>
  </button>
</li>

<style>
  .row {
    display: flex;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .row:hover {
    background: var(--bg-hover);
  }

  .open {
    display: flex;
    flex: 1;
    align-items: flex-start;
    gap: 10px;
    min-width: 0;
    min-height: 36px;
    padding: 10px 12px;
    border: 0;
    border-radius: inherit;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-body);
    text-align: left;
    cursor: pointer;
  }

  .open:active {
    background: var(--bg-active);
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* Same box as a checkbox, so titles of events and tasks line up in a mixed list. */
  .dot {
    flex: none;
    display: grid;
    place-items: center;
    width: 18px;
    height: 20px;
  }

  .dot::before {
    content: "";
    width: 7px;
    height: 7px;
    border-radius: var(--radius-chip);
    background: var(--tag, var(--danger));
  }

  .text {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .title {
    overflow: hidden;
    font-weight: 600;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .repeat::before {
    content: "· ";
  }

  .when {
    color: var(--text-secondary);
    font-size: var(--text-caption);
    font-variant-numeric: tabular-nums;
    line-height: 18px;
  }
</style>
