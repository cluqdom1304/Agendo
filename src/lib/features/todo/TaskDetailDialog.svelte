<script lang="ts">
  import { formatDisplayDate } from "../../dates";
  import { t } from "../../i18n";
  import {
    describeRepeat,
    formatMoment,
    offsetLabel,
    pending,
    sent,
  } from "../../reminders";
  import { durationLabel, timeRange } from "../../schedule";
  import Modal from "../../shell/Modal.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Reminder, Task } from "../../types";
  import NoteEditor from "../notes/NoteEditor.svelte";

  // The view of one task: everything about it, notes included, with the actions that matter
  // (done, edit, delete). Editing opens the same dialog that creates tasks.
  interface Props {
    store: TasksStore;
    taskId: string;
    onEdit: (task: Task) => void;
    /** Leaves this view and shows the task in the list. */
    onShowInList: (task: Task) => void;
    onClose: () => void;
  }

  let { store, taskId, onEdit, onShowInList, onClose }: Props = $props();

  const task = $derived(store.find(taskId));
  const busy = $derived(store.settling.has(taskId));
  // While the exit animation plays the button already shows what it is about to do.
  const done = $derived(task ? task.isCompleted !== busy : false);
  const waiting = $derived(task ? pending(task.reminders) : []);
  const history = $derived(task ? sent(task.reminders) : []);

  // The task disappears from the store when it is deleted (here or by undo timers): close.
  $effect(() => {
    if (!task) onClose();
  });

  function describe(reminder: Reminder): string {
    const moment = formatMoment(reminder.notifyAt);
    const parts =
      reminder.offsetMinutes === null
        ? [moment]
        : [offsetLabel(reminder.offsetMinutes), moment];
    if (reminder.repeat) parts.push(describeRepeat(reminder.repeat));
    return parts.join(" · ");
  }

  async function toggleDone() {
    if (!task || busy) return;
    await store.setCompleted(task.id, !task.isCompleted);
  }

  function remove() {
    if (!task) return;
    void store.remove(task.id);
    onClose();
  }

  function isEditable(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLElement &&
      (target.isContentEditable ||
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement)
    );
  }

  // Supr deletes like in the list, unless the user is typing.
  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Delete" || isEditable(event.target)) return;
    event.preventDefault();
    remove();
  }
</script>

{#if task}
  <Modal label={t.detail.label} closeLabel={t.detail.close} {onClose}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="detail" onkeydown={onKeydown}>
      <h2 class:done>{task.title || t.calendar.untitled}</h2>

      <dl class="facts">
        <div>
          <dt>{t.detail.status}</dt>
          <dd class="status" class:done>
            {done ? t.detail.statusDone : t.detail.statusOpen}
          </dd>
        </div>
        <div>
          <dt>{t.newTask.dateLabel}</dt>
          <dd>
            {task.dueDate ? formatDisplayDate(task.dueDate) : t.detail.noDate}
          </dd>
        </div>
        {#if task.dueTime}
          <div>
            <dt>{t.detail.schedule}</dt>
            <dd>
              {timeRange(task.dueTime, task.durationMinutes)}
              {#if task.durationMinutes !== null}
                <span class="muted">
                  ({durationLabel(task.durationMinutes)})
                </span>
              {/if}
            </dd>
          </div>
        {/if}
      </dl>

      <section class="section" aria-label={t.newTask.remindersHeading}>
        <h3>{t.newTask.remindersHeading}</h3>
        {#if task.isCompleted}
          <p class="muted">{t.newTask.doneNoReminders}</p>
        {:else if waiting.length === 0}
          <p class="muted">{t.detail.noReminders}</p>
        {:else}
          <ul>
            {#each waiting as reminder (reminder.id)}
              <li>{describe(reminder)}</li>
            {/each}
          </ul>
        {/if}
        {#if history.length > 0}
          <ul class="history">
            {#each history.slice(-5) as reminder (reminder.id)}
              <li>{describe(reminder)}</li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="section" aria-label={t.detail.notes}>
        <h3>{t.detail.notes}</h3>
        <NoteEditor
          contentJson={task.contentJson}
          save={(json) => store.saveNotes(task.id, json)}
        />
      </section>

      <div class="actions">
        <button type="button" class="danger" onclick={remove}>
          {t.detail.remove}
        </button>
        <button type="button" class="link" onclick={() => onShowInList(task)}>
          {t.detail.showInList}
        </button>
        <span class="spacer"></span>
        <button type="button" onclick={() => onEdit(task)}>
          {t.detail.edit}
        </button>
        <button
          type="button"
          class="primary"
          disabled={busy}
          onclick={() => void toggleDone()}
        >
          {done ? t.detail.reopen : t.detail.markDone}
        </button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  .detail {
    display: grid;
    gap: var(--space-4);
  }

  h2 {
    margin: 0;
    padding-right: var(--space-6);
    font-size: 18px;
    overflow-wrap: anywhere;
  }

  h2.done {
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  h3 {
    margin: 0 0 var(--space-1);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .facts {
    display: grid;
    gap: var(--space-2);
    margin: 0;
  }

  .facts div {
    display: grid;
    grid-template-columns: 90px 1fr;
    gap: var(--space-3);
  }

  dt {
    color: var(--text-secondary);
    font-size: 12px;
  }

  dd {
    margin: 0;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }

  .status {
    font-weight: 600;
  }

  .status.done,
  .muted {
    color: var(--text-secondary);
    font-weight: 400;
  }

  .section p,
  .section ul {
    margin: 0;
    font-size: 13px;
  }

  ul {
    padding-left: var(--space-4);
  }

  .history {
    margin-top: var(--space-1);
    color: var(--text-secondary);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .spacer {
    flex: 1;
  }

  button {
    padding: 6px 14px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  button.primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 600;
  }

  button.danger {
    color: var(--danger);
  }

  button.link {
    padding: 0 var(--space-2);
    border: 0;
    background: transparent;
    color: var(--accent);
    font-size: 12px;
  }
</style>
