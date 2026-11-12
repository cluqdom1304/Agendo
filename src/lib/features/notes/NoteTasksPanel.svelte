<script lang="ts">
  import { formatDisplayDate } from "../../dates";
  import { t } from "../../i18n";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Task } from "../../types";
  import { taskRefIds } from "./content";

  // "Tareas de esta nota" (F10-04): the tasks the note points at plus the ones created from
  // it, with how many are done.
  interface Props {
    tasks: TasksStore;
    noteId: string;
    /** The note's document, to find its references. */
    contentJson: string;
    onOpen: (task: Task) => void;
  }

  let { tasks, noteId, contentJson, onOpen }: Props = $props();

  let collapsed = $state(false);

  // References first, in the order of the note; then tasks made from it that it no longer shows.
  const own = $derived.by(() => {
    const referenced = taskRefIds(contentJson).flatMap((id) => {
      const task = tasks.find(id);
      return task ? [task] : [];
    });
    const known = new Set(referenced.map((task) => task.id));
    const created = tasks.tasks.filter(
      (task) => task.sourceNoteId === noteId && !known.has(task.id),
    );
    return [...referenced, ...created];
  });
  const done = $derived(own.filter((task) => task.isCompleted).length);
  const share = $derived(own.length === 0 ? 0 : done / own.length);
</script>

<section class="panel" aria-label={t.notesApp.notesTasks}>
  <button
    type="button"
    class="head"
    aria-expanded={!collapsed}
    onclick={() => (collapsed = !collapsed)}
  >
    <svg
      class="chevron"
      class:turned={!collapsed}
      viewBox="0 0 12 12"
      width="10"
      height="10"
      aria-hidden="true"
    >
      <path
        d="M4 2.5l3.5 3.5L4 9.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <span class="title">{t.notesApp.notesTasks}</span>
    {#if own.length > 0}
      <span class="progress-text">
        {t.notesApp.tasksProgress(done, own.length)}
      </span>
    {/if}
  </button>

  {#if own.length > 0}
    <div
      class="bar"
      role="progressbar"
      aria-valuemin="0"
      aria-valuemax={own.length}
      aria-valuenow={done}
      aria-label={t.notesApp.tasksProgress(done, own.length)}
    >
      <span class="fill" style:transform="scaleX({share})"></span>
    </div>
  {/if}

  {#if !collapsed}
    {#if own.length === 0}
      <p class="empty">{t.notesApp.noNoteTasks}</p>
    {:else}
      <ul class="rows">
        {#each own as task (task.id)}
          <li class="row" class:done={task.isCompleted}>
            <button
              type="button"
              class="check"
              role="checkbox"
              aria-checked={task.isCompleted}
              aria-label={task.isCompleted
                ? t.notesApp.taskReopen(task.title)
                : t.notesApp.taskComplete(task.title)}
              onclick={() =>
                void tasks.setCompleted(task.id, !task.isCompleted)}
            ></button>
            <button
              type="button"
              class="name"
              title={t.notesApp.taskOpen}
              onclick={() => onOpen(task)}
            >
              {task.title || t.notesApp.untitledTask}
            </button>
            {#if task.dueDate}
              <span class="due">{formatDisplayDate(task.dueDate)}</span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</section>

<style>
  .panel {
    flex: none;
    margin-top: var(--space-4);
    padding: var(--space-2) var(--space-3) var(--space-3);
    border-radius: var(--radius-card);
    background: var(--bg-control);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 4px 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .chevron {
    flex: none;
    color: var(--text-secondary);
    transition: transform var(--motion-fast) var(--ease-out);
  }

  .chevron.turned {
    transform: rotate(90deg);
  }

  .title {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
  }

  .progress-text {
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .bar {
    height: 4px;
    margin: 4px 0 var(--space-2);
    overflow: hidden;
    border-radius: var(--radius-chip);
    background: var(--bg-active);
  }

  .fill {
    display: block;
    width: 100%;
    height: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform var(--motion-base) var(--ease-out);
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 3px 0;
  }

  .check {
    position: relative;
    flex: none;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
    background: transparent;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      transform var(--motion-base) var(--ease-bounce);
  }

  .check:active {
    transform: scale(0.82);
  }

  .done .check {
    border-color: var(--accent);
    background: var(--accent);
  }

  .done .check::after {
    content: "";
    position: absolute;
    top: 1.5px;
    left: 4.5px;
    width: 4px;
    height: 7px;
    border: solid var(--on-accent);
    border-width: 0 1.5px 1.5px 0;
    transform: rotate(45deg);
  }

  .name {
    flex: 1;
    min-width: 0;
    padding: 0;
    overflow: hidden;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
  }

  .done .name {
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  .due {
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .empty {
    margin: 0;
    padding: 2px 0 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (prefers-reduced-motion: reduce) {
    .fill,
    .chevron {
      transition: none;
    }
  }
</style>
