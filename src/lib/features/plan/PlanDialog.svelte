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
      <h3>{title}</h3>
      <ul>
        {#each tasks as task (task.id)}
          <li>
            <label>
              <input
                type="checkbox"
                checked={chosen.includes(task.id)}
                onchange={() => toggle(task.id)}
              />
              <span class="name">{task.title || t.reminders.untitled}</span>
              <span class="meta">
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
    <h2>{l.title}</h2>

    {#if candidates.today.length > 0}
      <section class="group" aria-label={l.alreadyToday}>
        <h3>{l.alreadyToday}</h3>
        <ul>
          {#each candidates.today as task (task.id)}
            <li class="fixed">
              <span class="name">{task.title || t.reminders.untitled}</span>
              <span class="meta">
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
      <p class="empty">{l.nothingToAdd}</p>
    {/if}

    <p class="total" class:over={totals.overloaded} aria-live="polite">
      {l.total(totals.tasks, durationLabel(totals.minutes), totals.unestimated)}
    </p>
    {#if totals.overloaded}
      <p class="warn" role="alert">{l.overloaded}</p>
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
    gap: var(--space-3);
    min-width: 0;
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 17px;
  }

  h3 {
    margin: 0 0 var(--space-1);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  ul {
    max-height: 180px;
    margin: 0;
    padding: 0;
    overflow: auto;
    list-style: none;
  }

  label,
  .fixed {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 32px;
    padding: 0 var(--space-2);
    border-radius: var(--radius-control);
    font-size: 13px;
  }

  label {
    cursor: pointer;
  }

  label:hover {
    background: var(--bg-hover);
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .empty,
  .total {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .total {
    color: var(--text-primary);
    font-weight: 600;
  }

  .total.over,
  .warn {
    color: var(--danger);
  }

  .warn {
    margin: 0;
    font-size: 13px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>
