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
</script>

{#if tasks.length === 0}
  <p class="empty">{t.notesApp.noTasks}</p>
{:else}
  <ul class="tasks">
    {#each tasks as task (task.id)}
      <li>
        <button
          type="button"
          class="task"
          class:done={task.isCompleted}
          aria-label={t.notesApp.openTask(task.title || t.notesApp.untitled)}
          onclick={() => onOpen(task)}
        >
          <span class="mark" aria-hidden="true">
            {#if task.isCompleted}✓{/if}
          </span>
          <span class="text">
            <span class="title">{task.title || t.notesApp.untitled}</span>
            <span class="meta">
              {#if task.isCompleted}{t.notesApp
                  .done}{:else if task.dueDate}{formatDisplayDate(
                  task.dueDate,
                )}{:else}{t.detail.noDate}{/if}
            </span>
          </span>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .tasks {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0 var(--space-2);
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

  .mark {
    display: grid;
    flex: none;
    place-items: center;
    width: 16px;
    height: 16px;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
    color: var(--on-accent);
    font-size: 10px;
    line-height: 1;
  }

  .done .mark {
    border-color: var(--accent);
    background: var(--accent);
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
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .done .title {
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  .meta {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .empty {
    margin: 0;
    padding: var(--space-6) var(--space-4);
    color: var(--text-secondary);
    font-size: 13px;
    text-align: center;
  }
</style>
