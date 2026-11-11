<script lang="ts">
  import { isSettledDate } from "../../dates";
  import { t } from "../../i18n";
  import { timeRange } from "../../schedule";
  import Checkbox from "./Checkbox.svelte";
  import TagChip from "../tags/TagChip.svelte";
  import TagMenu from "../tags/TagMenu.svelte";
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
    tagId?: string | null;
    tagName?: string | null;
    tagColor?: string | null;
    onTagChange?: (tagId: string | null) => void;
    /** Pressed on the grip: the parent starts dragging the task towards a sidebar tag. */
    onTagDragStart?: (event: PointerEvent) => void;
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
    tagId = null,
    tagName = null,
    tagColor = null,
    onTagChange,
    onTagDragStart,
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

  let tagMenuOpen = $state(false);

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
  {#if !draft}
    <span
      class="grip"
      role="presentation"
      title={t.tags.grab(title)}
      onpointerdown={(event) => onTagDragStart?.(event)}
    >
      <svg viewBox="0 0 8 14" aria-hidden="true" width="8" height="14">
        <circle cx="2" cy="3" r="1" />
        <circle cx="6" cy="3" r="1" />
        <circle cx="2" cy="7" r="1" />
        <circle cx="6" cy="7" r="1" />
        <circle cx="2" cy="11" r="1" />
        <circle cx="6" cy="11" r="1" />
      </svg>
    </span>
  {/if}
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
      onchange={(event) => {
        if (isSettledDate(event.currentTarget.value))
          onDateChange?.(event.currentTarget.value || null);
      }}
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
  {#if !draft}
    <span class="tag-slot">
      <button
        type="button"
        class="tag-button"
        class:empty={!tagName}
        data-tag-toggle
        aria-haspopup="menu"
        aria-expanded={tagMenuOpen}
        aria-label={t.tags.assign(title)}
        title={t.tags.assign(title)}
        tabindex="-1"
        onclick={() => (tagMenuOpen = !tagMenuOpen)}
      >
        {#if tagName && tagColor}
          <TagChip name={tagName} color={tagColor} />
        {:else}
          <svg viewBox="0 0 16 16" aria-hidden="true" class="notes-icon">
            <path d="M2.5 8.5V3h5.5l5.5 5.5-5.5 5.5z" />
            <circle cx="5.5" cy="6" r="0.8" />
          </svg>
        {/if}
      </button>
      {#if tagMenuOpen}
        <TagMenu
          value={tagId}
          onSelect={(id) => onTagChange?.(id)}
          onClose={() => (tagMenuOpen = false)}
        />
      {/if}
    </span>
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
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 34px;
    padding: 3px var(--space-3);
    border-radius: var(--radius-control);
    outline: none;
    transition:
      opacity 220ms var(--ease-apple),
      transform 220ms var(--ease-apple),
      background-color var(--motion-instant) linear;
  }

  .row:hover,
  .row:focus-within {
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

  .grip {
    position: absolute;
    top: 50%;
    left: 1px;
    display: grid;
    place-items: center;
    width: 10px;
    height: 20px;
    margin-top: -10px;
    color: var(--text-secondary);
    opacity: 0;
    cursor: grab;
    touch-action: none;
    transition: opacity var(--motion-fast) var(--ease-out);
  }

  .grip circle {
    fill: currentColor;
  }

  .row:hover .grip,
  .row:focus-within .grip {
    opacity: 0.7;
  }

  .tag-slot {
    position: relative;
    display: inline-flex;
    flex: none;
  }

  .tag-button {
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: transform var(--motion-instant) var(--ease-out);
  }

  .tag-button:active {
    transform: scale(0.94);
  }

  .tag-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  /* Row actions stay out of the way until the row is hovered or focused. */
  .delete,
  .notes-toggle,
  .tag-button.empty {
    display: grid;
    place-items: center;
    flex: none;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    opacity: 0;
    cursor: pointer;
    transition:
      opacity var(--motion-fast) var(--ease-out),
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .delete:hover,
  .notes-toggle:hover,
  .tag-button.empty:hover {
    background: var(--bg-control);
    color: var(--text-primary);
  }

  .delete:active,
  .notes-toggle:active,
  .tag-button.empty:active {
    transform: scale(0.9);
  }

  .delete:hover {
    color: var(--danger);
  }

  .date-input {
    flex: none;
    width: 118px;
    height: 24px;
    padding: 0 var(--space-1);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    opacity: 0;
    cursor: pointer;
    transition:
      opacity var(--motion-fast) var(--ease-out),
      background-color var(--motion-instant) linear;
  }

  .date-input:hover {
    background: var(--bg-control);
  }

  .row:hover .date-input,
  .row:focus-within .date-input,
  .row:hover .delete,
  .row:focus-within .delete,
  .row:hover .notes-toggle,
  .row:focus-within .notes-toggle,
  .row:hover .tag-button.empty,
  .row:focus-within .tag-button.empty,
  .notes-toggle.has-notes,
  .notes-toggle[aria-expanded="true"],
  .tag-button.empty[aria-expanded="true"] {
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
