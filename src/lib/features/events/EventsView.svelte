<script lang="ts">
  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import { addDays } from "../calendar/grid";
  import EventRow from "./EventRow.svelte";

  // «Eventos» (F21-01): the events of the next three months, day by day. A repeating event
  // appears on each day it falls on.
  interface Props {
    store: EventsStore;
    view: ViewStore;
    onOpenEvent: (eventId: string) => void;
    onNewEvent: () => void;
  }

  let { store, view, onOpenEvent, onNewEvent }: Props = $props();
  const l = t.events;

  /** Days ahead, today included, that the view lists. */
  const HORIZON_DAYS = 90;

  // A window left open overnight must not keep yesterday as today.
  // svelte-ignore state_referenced_locally
  let today = $state(view.today());

  $effect(() => {
    void store.show(today, addDays(today, HORIZON_DAYS - 1));
  });

  const longDate = new Intl.DateTimeFormat("es-ES", {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
  const asDate = (iso: string) => new Date(`${iso}T12:00:00`);
  const capitalize = (text: string) =>
    text.charAt(0).toUpperCase() + text.slice(1);

  const days = $derived(
    [...store.byDate]
      .toSorted(([a], [b]) => (a < b ? -1 : 1))
      .map(([date, occurrences]) => ({
        date,
        heading:
          date === today
            ? t.upcoming.today
            : date === addDays(today, 1)
              ? t.upcoming.tomorrow
              : capitalize(longDate.format(asDate(date))),
        occurrences,
      })),
  );
</script>

<svelte:window onfocus={() => (today = view.today())} />

<div class="events">
  <header class="head">
    <div>
      <h2>{l.title}</h2>
      <p class="horizon">{l.horizon}</p>
    </div>
    <button type="button" class="btn primary" onclick={onNewEvent}>
      {l.newEvent}
    </button>
  </header>

  {#if days.length === 0}
    <EmptyState title={l.emptyTitle} description={l.emptyDescription} />
  {:else}
    {#each days as day (day.date)}
      <section class="block" aria-labelledby="events-{day.date}">
        <h3 id="events-{day.date}">
          {day.heading}
          <span class="count">{day.occurrences.length}</span>
        </h3>
        <ul class="card">
          {#each day.occurrences as occurrence (occurrence.eventId + occurrence.startDate)}
            <EventRow {occurrence} onOpen={onOpenEvent} />
          {/each}
        </ul>
      </section>
    {/each}
  {/if}
</div>

<style>
  .events {
    display: grid;
    align-content: start;
    gap: var(--space-6);
    box-sizing: border-box;
    width: 100%;
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) var(--space-8);
  }

  .head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--space-4);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 34px;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.025em;
  }

  .horizon {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .block {
    display: grid;
    gap: 6px;
  }

  h3 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    margin: 0;
    padding: 0 2px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .count {
    font-weight: 400;
    font-variant-numeric: tabular-nums;
  }

  .card {
    margin: 0;
    padding: 0;
    overflow: hidden;
    border-radius: var(--radius-card);
    background: var(--bg-control);
    list-style: none;
  }
</style>
