<script lang="ts">
  import { t } from "../../i18n";
  import { timeRange } from "../../schedule";
  import Checkbox from "./Checkbox.svelte";
  import TagChip from "../tags/TagChip.svelte";
  import InlineTitle from "./InlineTitle.svelte";

  // Only the fields the row draws (CLAUDE.md §6): a change elsewhere in the task does not
  // reach this component.
  interface Props {
    id: string;
    title: string;
    isCompleted: boolean;
    dueTime: string | null;
    durationMinutes?: number | null;
    dueDate?: string | null;
    tagName?: string | null;
    tagColor?: string | null;
    onDateChange?: (date: string | null) => void;
    onDelete?: () => void;
    /** The task has notes; the toggle is drawn stronger so they are easy to spot. */
    hasNotes?: boolean;
    notesOpen?: boolean;
    onToggleNotes?: () => void;
    /** Reminders that have not fired yet; the bell stays visible while there are any. */
    reminderCount?: number;
    remindersOpen?: boolean;
    onToggleReminders?: () => void;
    /** Playing the exit animation of SPEC §8.4 while the backend confirms. */
    settling?: boolean;
    /** An unsaved row waiting for its title. */
    draft?: boolean;
    onToggle?: () => void;
    onSelect?: () => void;
    onCommit: (text: string, trigger: "enter" | "blur") => void;
    onCancel?: () => void;
  }

  let {
    id,
    title,
    isCompleted,
    dueTime,
    durationMinutes = null,
    dueDate = null,
    tagName = null,
    tagColor = null,
    onDateChange,
    onDelete,
    hasNotes = false,
    notesOpen = false,
    onToggleNotes,
    reminderCount = 0,
    remindersOpen = false,
    onToggleReminders,
    settling = false,
    draft = false,
    onToggle,
    onSelect,
    onCommit,
    onCancel,
  }: Props = $props();

  // While settling, the checkbox already shows the state the task is moving to.
  const checked = $derived(settling ? !isCompleted : isCompleted);
  const checkboxLabel = $derived(
    checked ? t.tasks.markOpen(title) : t.tasks.markDone(title),
  );
</script>

<div
  class="row"
  class:done={checked}
  class:settling
  role="row"
  tabindex="0"
  data-task-row
  data-task-id={draft ? undefined : id}
  onfocus={onSelect}
>
  <Checkbox
    {checked}
    label={checkboxLabel}
    disabled={draft}
    onToggle={() => onToggle?.()}
  />
  <InlineTitle
    value={title}
    label={draft ? t.tasks.newTaskLabel : t.tasks.titleLabel}
    placeholder={draft ? t.tasks.newTaskPlaceholder : ""}
    completed={checked}
    focusOnMount={draft}
    {onCommit}
    {onCancel}
  />
  {#if !draft}
    <input
      type="date"
      class="date-input"
      value={dueDate ?? ""}
      aria-label={t.tasks.dateLabel(title)}
      title={t.tasks.dateLabel(title)}
      tabindex="-1"
      onchange={(event) => onDateChange?.(event.currentTarget.value || null)}
    />
    <button
      type="button"
      class="delete"
      aria-label={t.tasks.deleteLabel(title)}
      title={t.tasks.deleteLabel(title)}
      tabindex="-1"
      onclick={() => onDelete?.()}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true" class="notes-icon">
        <path d="M4 4l8 8M12 4l-8 8" />
      </svg>
    </button>
  {/if}
  {#if !draft && !checked}
    <button
      type="button"
      class="notes-toggle"
      data-reminders-toggle
      class:has-notes={reminderCount > 0}
      aria-expanded={remindersOpen}
      aria-label={reminderCount > 0
        ? `${t.reminders.open(title)} (${t.reminders.count(reminderCount)})`
        : t.reminders.open(title)}
      title={t.reminders.open(title)}
      tabindex="-1"
      onclick={() => onToggleReminders?.()}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true" class="notes-icon">
        <path d="M4 11V7a4 4 0 0 1 8 0v4l1 1.5H3zM6.5 14h3" />
      </svg>
    </button>
    <button
      type="button"
      class="notes-toggle"
      class:has-notes={hasNotes}
      aria-expanded={notesOpen}
      aria-label={notesOpen ? t.notes.close(title) : t.notes.open(title)}
      title={notesOpen ? t.notes.close(title) : t.notes.open(title)}
      tabindex="-1"
      onclick={() => onToggleNotes?.()}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true" class="notes-icon">
        <path d="M3 4.5h10M3 8h10M3 11.5h6" />
      </svg>
    </button>
  {/if}
  {#if tagName && tagColor}
    <TagChip name={tagName} color={tagColor} />
  {/if}
  {#if dueTime}
    <span class="time">
      <svg viewBox="0 0 16 16" aria-hidden="true" class="clock">
        <circle cx="8" cy="8" r="6" />
        <path d="M8 4.5V8l2.5 1.5" />
      </svg>
      {timeRange(dueTime, durationMinutes)}
    </span>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 3px var(--space-3);
    border-radius: var(--radius-control);
    outline: none;
    transition:
      opacity 220ms var(--ease-apple),
      transform 220ms var(--ease-apple),
      background-color var(--motion-fast) var(--ease-apple);
  }

  .row:hover {
    background: var(--bg-hover);
  }

  /* Done tasks recede so the pending ones of the day stand out. */
  .row.done {
    opacity: 0.6;
  }

  .row.done:hover,
  .row.done:focus-within {
    opacity: 1;
  }

  .row:focus-visible {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }

  .row.settling {
    opacity: 0;
    transform: translateY(8px);
  }

  .date-input {
    flex: none;
    width: 118px;
    height: 22px;
    padding: 0 var(--space-1);
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    opacity: 0;
    cursor: pointer;
    transition: opacity var(--motion-fast) var(--ease-apple);
  }

  .delete {
    display: grid;
    place-items: center;
    flex: none;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    opacity: 0;
    cursor: pointer;
    transition: opacity var(--motion-fast) var(--ease-apple);
  }

  .row:hover .date-input,
  .row:focus-within .date-input,
  .row:hover .delete,
  .row:focus-within .delete {
    opacity: 1;
  }

  .delete:hover {
    color: var(--danger);
  }

  .notes-toggle {
    display: grid;
    place-items: center;
    flex: none;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    opacity: 0;
    cursor: pointer;
    transition: opacity var(--motion-fast) var(--ease-apple);
  }

  .row:hover .notes-toggle,
  .row:focus-within .notes-toggle,
  .notes-toggle.has-notes,
  .notes-toggle[aria-expanded="true"] {
    opacity: 1;
  }

  .notes-toggle.has-notes {
    color: var(--accent);
  }

  .notes-icon {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
  }

  .time {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .clock {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
  }
</style>
