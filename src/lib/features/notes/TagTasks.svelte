<script lang="ts">
  import { formatDisplayDate } from "../../dates";
  import { t } from "../../i18n";
  import type { Task } from "../../types";

  interface Props {
    /** The tasks carrying the tag, in the order to show them. */
    tasks: Task[];
    onOpen: (task: Task) => void;
  }

  let { tasks, onOpen }: Props = $props();

  const rows = $derived(
    tasks.map((task) => ({
      task,
      title: task.title || t.notesApp.untitled,
      meta: task.isCompleted
        ? t.notesApp.done
        : task.dueDate
          ? formatDisplayDate(task.dueDate)
          : t.detail.noDate,
    })),
  );
</script>

{#if rows.length === 0}
  <p class="empty">{t.notesApp.noTasks}</p>
{:else}
  <ul class="tasks">
    {#each rows as row (row.task.id)}
      <li>
        <button
          type="button"
          class="task"
          class:done={row.task.isCompleted}
          aria-label={t.notesApp.openTask(row.title)}
          onclick={() => onOpen(row.task)}
        >
          <span class="mark" aria-hidden="true"></span>
          <span class="text">
            <span class="title">{row.title}</span>
            <span class="meta">{row.meta}</span>
          </span>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  /* Same rhythm as the note list it swaps with: rounded rows, two lines of text. */
  .tasks {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 2px;
    min-height: 0;
    margin: 0;
    padding: 2px var(--space-2) var(--space-2);
    overflow-y: auto;
    list-style: none;
  }

  .task {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: 8px var(--space-3);
    border: 0;
    border-radius: var(--radius-card);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .task:hover {
    background: var(--bg-hover);
  }

  .task:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* The round checkbox of the agenda, here only as a sign of the state. */
  .mark {
    position: relative;
    box-sizing: border-box;
    flex: none;
    width: 16px;
    height: 16px;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
  }

  .done .mark {
    border-color: var(--ink);
    background: var(--ink);
  }

  .done .mark::after {
    content: "";
    position: absolute;
    top: 1.5px;
    left: 4.5px;
    width: 4px;
    height: 7px;
    border: solid var(--on-ink);
    border-width: 0 1.5px 1.5px 0;
    transform: rotate(45deg);
  }

  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .title {
    overflow: hidden;
    font-size: 13px;
    font-weight: 600;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .done .title {
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  .meta {
    overflow: hidden;
    color: var(--text-secondary);
    font-size: var(--text-caption);
    font-variant-numeric: tabular-nums;
    line-height: 17px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    margin: 0;
    padding: var(--space-8) var(--space-4);
    color: var(--text-secondary);
    font-size: 13px;
    text-align: center;
  }
</style>
