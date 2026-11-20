<script lang="ts">
  import { formatDisplayDate } from "../../dates";
  import { t } from "../../i18n";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Task } from "../../types";
  import { taskRefIds } from "./content";

  // "Tareas de esta nota" (F10-04): the tasks the note points at plus the ones created from
  // it, with how many are done. A note without tasks shows no panel at all.
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

{#if own.length > 0}
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
      <span class="progress-text">
        {t.notesApp.tasksProgress(done, own.length)}
      </span>
    </button>

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

    {#if !collapsed}
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
  </section>
{/if}

<style>
  /* A footer of the note, set apart by a hairline instead of a box. */
  .panel {
    flex: none;
    margin-top: var(--space-4);
    padding-top: var(--space-2);
    border-top: 1px solid var(--separator);
    animation: fade-in var(--motion-base) var(--ease-out);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 4px 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
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
    font-weight: 700;
  }

  .progress-text {
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .bar {
    height: 3px;
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
    min-height: 28px;
    margin: 0 calc(var(--space-2) * -1);
    padding: 0 var(--space-2);
    border-radius: var(--radius-small);
    transition: background-color var(--motion-instant) linear;
  }

  .row:hover {
    background: var(--bg-hover);
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
    border-color: var(--ink);
    background: var(--ink);
  }

  .done .check::after {
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

  .name {
    align-self: stretch;
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
