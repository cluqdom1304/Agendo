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

<div class="events view">
  <header class="view-head">
    <div class="view-heading">
      <h2 class="view-title">{l.title}</h2>
      <p class="view-sub">{l.horizon}</p>
    </div>
    <div class="view-actions">
      <button type="button" class="btn primary" onclick={onNewEvent}>
        {l.newEvent}
      </button>
    </div>
  </header>

  {#if days.length === 0}
    <EmptyState title={l.emptyTitle} description={l.emptyDescription} />
  {:else}
    {#each days as day (day.date)}
      <section class="block" aria-labelledby="events-{day.date}">
        <h3 id="events-{day.date}" class="group-label">
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
  .block {
    display: grid;
    gap: 10px;
  }

  /* One card per event, as «Próximas» draws its items. */
  .card {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .card > :global(li) {
    background: var(--bg-card);
  }

  .card > :global(li:hover) {
    background: var(--bg-card-hover);
  }
</style>
