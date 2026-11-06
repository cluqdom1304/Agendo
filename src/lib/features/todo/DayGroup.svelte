<script lang="ts">
  import { formatDisplayDate } from "../../dates";
  import { t } from "../../i18n";
  import type { TaskBlock, TasksStore } from "../../stores/tasks.svelte";
  import TaskRow from "./TaskRow.svelte";

  // One block of tasks: a day of AGENDA, or POR HACER when `block.date` is null.
  interface Props {
    block: TaskBlock;
    store: TasksStore;
  }

  let { block, store }: Props = $props();

  const target = $derived(
    block.date === null
      ? ({ section: "todo" } as const)
      : ({ section: "agenda", date: block.date } as const),
  );
  const draft = $derived(store.isDraftFor(target) ? store.draft : null);
  const heading = $derived(
    block.date === null ? t.tasks.sectionTodo : formatDisplayDate(block.date),
  );
</script>

{#snippet taskRow(task: TaskBlock["open"][number])}
  <TaskRow
    id={task.id}
    title={task.title}
    isCompleted={task.isCompleted}
    dueTime={task.dueTime}
    settling={store.settling.has(task.id)}
    onToggle={() => store.setCompleted(task.id, !task.isCompleted)}
    onSelect={() => (store.selectedId = task.id)}
    onCommit={(text, trigger) => {
      void store.rename(task.id, text);
      if (trigger === "enter") store.startDraft(target);
    }}
  />
{/snippet}

<section class="block" aria-label={heading}>
  {#if block.date !== null}
    <h3 class="day">{heading}</h3>
  {/if}

  {#each block.open as task (task.id)}
    {@render taskRow(task)}
  {/each}

  <!-- Keyed by draft: each draft gets a fresh input, and the handlers keep the key of the
       draft they were created for even when a late blur arrives after it was closed. -->
  {#each draft ? [draft] : [] as current (current.key)}
    <TaskRow
      id=""
      title=""
      isCompleted={false}
      dueTime={null}
      draft
      onCommit={(text, trigger) =>
        void store.commitDraft(current.key, text, trigger === "enter")}
      onCancel={() => store.cancelDraft(current.key)}
    />
  {/each}

  {#if block.completed.length > 0}
    <h4 class="completed-heading">{t.tasks.completed}</h4>
    {#each block.completed as task (task.id)}
      {@render taskRow(task)}
    {/each}
  {/if}
</section>

<style>
  .block {
    padding-top: var(--space-2);
  }

  .day {
    margin: 0;
    padding: var(--space-2) var(--space-3) var(--space-1);
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .completed-heading {
    margin: var(--space-2) 0 0;
    padding: var(--space-1) var(--space-3);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
</style>
