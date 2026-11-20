<script lang="ts">
  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import type { HabitsStore } from "../../stores/habits.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { Habit } from "../../types";
  import { formatDisplayDate } from "../../dates";
  import { daysLabel, isScheduled, lastDays } from "./habits";

  // «Hábitos» (F22-01): each habit with its streak and the last seven days, which can be ticked
  // one by one (a forgotten day can still be marked, never a future one).
  interface Props {
    store: HabitsStore;
    view: ViewStore;
    onOpenHabit: (habit: Habit) => void;
    onNewHabit: () => void;
  }

  let { store, view, onOpenHabit, onNewHabit }: Props = $props();
  const l = t.habits;

  // A window left open overnight must not keep yesterday as today.
  // svelte-ignore state_referenced_locally
  let today = $state(view.today());

  const days = $derived(lastDays(today));
  $effect(() => {
    void store.show(days[0], today);
  });

  const weekdayShort = new Intl.DateTimeFormat("es-ES", { weekday: "narrow" });
  const asDate = (iso: string) => new Date(`${iso}T12:00:00`);
  const heads = $derived(
    days.map((date) => ({ date, label: weekdayShort.format(asDate(date)) })),
  );
</script>

<svelte:window onfocus={() => (today = view.today())} />

<div class="habits view">
  <header class="view-head">
    <div class="view-heading">
      <h2 class="view-title">{l.title}</h2>
      <p class="view-sub">{l.last7}</p>
    </div>
    <div class="view-actions">
      <button type="button" class="btn primary" onclick={onNewHabit}>
        {l.newHabit}
      </button>
    </div>
  </header>

  {#if store.active.length + store.archived.length === 0}
    <EmptyState title={l.emptyTitle} description={l.emptyDescription} />
  {:else}
    {@render group(store.active)}
    {#if store.archived.length > 0}
      <h3 class="group-label">{l.archived}</h3>
      {@render group(store.archived)}
    {/if}
  {/if}
</div>

{#snippet group(habits: Habit[])}
  {#if habits.length > 0}
    <ul class="card">
      <li class="legend" aria-hidden="true">
        <span class="spacer"></span>
        <span class="cells">
          {#each heads as head (head.date)}
            <span class="cell-head">{head.label}</span>
          {/each}
        </span>
      </li>
      {#each habits as habit (habit.id)}
        <li
          class="row"
          class:archived={habit.isArchived}
          data-habit-id={habit.id}
        >
          <button
            type="button"
            class="open"
            title={l.open(habit.title)}
            onclick={() => onOpenHabit(habit)}
          >
            <span class="title">{habit.title}</span>
            <span class="meta">
              {daysLabel(habit.days)}{habit.time ? ` · ${habit.time}` : ""}
            </span>
            <span class="streak">
              {l.streak(habit.currentStreak)} · {l.best(habit.bestStreak)}
            </span>
          </button>
          <span class="cells">
            {#each days as date (date)}
              {@const done = store.isDone(habit.id, date)}
              {@const scheduled = isScheduled(habit.days, date)}
              <button
                type="button"
                class="dot"
                class:done
                class:off={!scheduled && !done}
                role="checkbox"
                aria-checked={done}
                aria-label={done
                  ? l.unmarkDay(habit.title, formatDisplayDate(date))
                  : l.markDay(habit.title, formatDisplayDate(date))}
                title={scheduled || done ? undefined : l.notScheduled}
                onclick={() => void store.setDone(habit, date, !done)}
              ></button>
            {/each}
          </span>
        </li>
      {/each}
    </ul>
  {/if}
{/snippet}

<style>
  .card {
    margin: 0;
    padding: 0;
    overflow: hidden;
    border-radius: var(--radius-card);
    background: var(--bg-card);
    list-style: none;
  }

  .legend,
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-3);
  }

  .legend {
    padding-top: var(--space-2);
  }

  .row {
    position: relative;
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

  .row.archived {
    opacity: 0.75;
  }

  .spacer {
    flex: 1;
  }

  .cells {
    display: flex;
    flex: none;
    gap: 6px;
  }

  .cell-head {
    width: 22px;
    color: var(--text-secondary);
    font-size: 10px;
    text-align: center;
    text-transform: uppercase;
  }

  .open {
    display: grid;
    flex: 1;
    gap: 2px;
    min-width: 0;
    padding: 8px 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    border-radius: var(--radius-control);
  }

  .title {
    overflow: hidden;
    font-size: 13px;
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta,
  .streak {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .dot {
    width: 22px;
    height: 22px;
    box-sizing: border-box;
    padding: 0;
    border: 1.5px solid var(--text-secondary);
    border-radius: 50%;
    background: transparent;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .dot:hover {
    background: var(--bg-hover);
  }

  .dot:active {
    transform: scale(0.9);
  }

  .dot.done {
    border-color: var(--ink);
    background: var(--ink);
  }

  .dot.off {
    border-style: dashed;
    opacity: 0.5;
  }

  .dot:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
