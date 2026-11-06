<script lang="ts">
  import { t } from "../../i18n";
  import Checkbox from "./Checkbox.svelte";
  import InlineTitle from "./InlineTitle.svelte";

  // Only the fields the row draws (CLAUDE.md §6): a change elsewhere in the task does not
  // reach this component.
  interface Props {
    id: string;
    title: string;
    isCompleted: boolean;
    dueTime: string | null;
    /** The task has notes; the toggle is drawn stronger so they are easy to spot. */
    hasNotes?: boolean;
    notesOpen?: boolean;
    onToggleNotes?: () => void;
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
    hasNotes = false,
    notesOpen = false,
    onToggleNotes,
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
  {#if dueTime}
    <span class="time">
      <svg viewBox="0 0 16 16" aria-hidden="true" class="clock">
        <circle cx="8" cy="8" r="6" />
        <path d="M8 4.5V8l2.5 1.5" />
      </svg>
      {dueTime}
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

  .row:focus-visible {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }

  .row.settling {
    opacity: 0;
    transform: translateY(8px);
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
