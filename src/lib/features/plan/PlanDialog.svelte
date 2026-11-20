<script lang="ts">
  import { t } from "../../i18n";
  import { durationLabel } from "../../schedule";
  import Modal from "../../shell/Modal.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Task } from "../../types";
  import { planCandidates, planTotals } from "./plan";

  // «Planificar mi día» (F24-02): pick which overdue and undated tasks go into today and see
  // what the day adds up to before committing.
  interface Props {
    store: TasksStore;
    onClose: () => void;
  }

  let { store, onClose }: Props = $props();
  const l = t.plan;

  // The dialog plans the day it was opened on; the store is fixed for its life.
  // svelte-ignore state_referenced_locally
  const today = store.today();
  // svelte-ignore state_referenced_locally
  const candidates = planCandidates(store.tasks, today);
  // Overdue tasks start chosen (they are the ones that slipped); the inbox's start apart.
  let chosen = $state<string[]>(candidates.overdue.map((task) => task.id));
  let saving = $state(false);

  const day = $derived([
    ...candidates.today,
    ...[...candidates.overdue, ...candidates.undated].filter((task) =>
      chosen.includes(task.id),
    ),
  ]);
  const totals = $derived(planTotals(day));

  function toggle(id: string) {
    chosen = chosen.includes(id)
      ? chosen.filter((other) => other !== id)
      : [...chosen, id];
  }

  async function apply() {
    if (saving) return;
    saving = true;
    for (const id of chosen) await store.setDueDate(id, today);
    saving = false;
    onClose();
  }
</script>

{#snippet group(title: string, tasks: Task[])}
  {#if tasks.length > 0}
    <section class="group" aria-label={title}>
      <h3 class="group-label">
        {title}
        <span class="count">{tasks.length}</span>
      </h3>
      <ul class="choice-list">
        {#each tasks as task (task.id)}
          <li>
            <label>
              <input
                type="checkbox"
                checked={chosen.includes(task.id)}
                onchange={() => toggle(task.id)}
              />
              <span class="name">{task.title || t.reminders.untitled}</span>
              <span class="meta" class:none={task.durationMinutes === null}>
                {task.durationMinutes === null
                  ? l.noEstimate
                  : durationLabel(task.durationMinutes)}
              </span>
            </label>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
{/snippet}

<Modal label={l.title} closeLabel={l.close} {onClose}>
  <div class="plan">
    <header>
      <h2>{l.title}</h2>
      <p class="subtitle">{l.subtitle}</p>
    </header>

    {#if candidates.today.length > 0}
      <section class="group" aria-label={l.alreadyToday}>
        <h3 class="group-label">
          {l.alreadyToday}
          <span class="count">{candidates.today.length}</span>
        </h3>
        <ul class="choice-list">
          {#each candidates.today as task (task.id)}
            <li class="in-today">
              <span class="pin" aria-hidden="true"></span>
              <span class="name">{task.title || t.reminders.untitled}</span>
              <span class="meta" class:none={task.durationMinutes === null}>
                {task.durationMinutes === null
                  ? l.noEstimate
                  : durationLabel(task.durationMinutes)}
              </span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {@render group(l.overdue, candidates.overdue)}
    {@render group(l.undated, candidates.undated)}

    {#if candidates.overdue.length === 0 && candidates.undated.length === 0}
      <p class="callout empty">{l.nothingToAdd}</p>
    {/if}

    <section class="summary" class:over={totals.overloaded}>
      <span class="summary-label">{l.dayLabel}</span>
      <p class="total" aria-live="polite">
        {l.total(
          totals.tasks,
          durationLabel(totals.minutes),
          totals.unestimated,
        )}
      </p>
    </section>
    {#if totals.overloaded}
      <p class="callout warn" role="alert">{l.overloaded}</p>
    {/if}

    <div class="actions">
      <button type="button" class="btn" onclick={onClose}>{l.cancel}</button>
      <button
        type="button"
        class="btn primary"
        disabled={saving}
        onclick={() => void apply()}
      >
        {l.apply(chosen.length)}
      </button>
    </div>
  </div>
</Modal>

<style>
  .plan {
    display: grid;
    gap: 20px;
    min-width: 0;
  }

  .subtitle {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .group {
    display: grid;
    gap: 6px;
  }

  .choice-list {
    max-height: 200px;
  }

  label,
  .in-today {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 40px;
    padding: 0 14px;
    font-size: 14px;
  }

  label {
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  label:hover {
    background: var(--bg-card-hover);
  }

  /* Tasks already in today have no box to tick: a small mark keeps the names aligned. */
  .pin {
    flex: none;
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
  }

  .pin::before {
    content: "";
    width: 6px;
    height: 6px;
    border-radius: var(--radius-chip);
    background: var(--text-secondary);
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    flex: none;
    padding: 1px 9px;
    border-radius: var(--radius-chip);
    background: var(--bg-control);
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    line-height: 18px;
  }

  .meta.none {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .empty {
    color: var(--text-secondary);
    text-align: center;
  }

  .summary {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
    padding: 12px 14px;
    border-radius: var(--radius-card);
    background: var(--bg-card);
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 700;
  }

  .total {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .summary.over .total {
    color: var(--danger);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>
