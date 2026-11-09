<script lang="ts">
  import { isSettledDate } from "../../dates";
  import { t } from "../../i18n";
  import { draftReminders, pending, toInput } from "../../reminders";
  import {
    DURATION_PRESETS,
    durationBetween,
    durationLabel,
    endTimeOf,
    maxDuration,
  } from "../../schedule";
  import Modal from "../../shell/Modal.svelte";
  import { settingsStore } from "../../stores/settings.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ReminderInput, Task } from "../../types";
  import { textToDoc } from "../notes/content";
  import TagPicker from "../tags/TagPicker.svelte";
  import ReminderEditor from "./ReminderEditor.svelte";

  // Creates a task, or edits `task` when it is given (the notes are edited in the task's own
  // view, because a plain text box would flatten their formatting).
  interface Props {
    store: TasksStore;
    task?: Task;
    /** Pre-filled day, e.g. the one the calendar is showing. */
    defaultDate?: string | null;
    now?: () => Date;
    onClose: () => void;
  }

  let {
    store,
    task,
    defaultDate = null,
    now = () => new Date(),
    onClose,
  }: Props = $props();

  // The form starts from the task and then owns its state.
  // svelte-ignore state_referenced_locally
  let title = $state(task?.title ?? "");
  // svelte-ignore state_referenced_locally
  let dueDate = $state(task ? (task.dueDate ?? "") : (defaultDate ?? ""));
  // svelte-ignore state_referenced_locally
  let dueTime = $state(task?.dueTime ?? "");
  // svelte-ignore state_referenced_locally
  let duration = $state<number | null>(task?.durationMinutes ?? null);
  // svelte-ignore state_referenced_locally
  let tagId = $state<string | null>(task?.tagIds[0] ?? null);
  let description = $state("");
  // svelte-ignore state_referenced_locally
  let inputs = $state.raw<ReminderInput[]>(
    task ? pending(task.reminders).map(toInput) : [],
  );
  let remindersTouched = $state(false);
  let attempted = $state(false);
  let saving = $state(false);
  let endError = $state(false);

  const editing = $derived(task !== undefined);
  const isDone = $derived(task?.isCompleted ?? false);
  // Reminders already saved are not "new": only the ones the user adds can be refused as past.
  // svelte-ignore state_referenced_locally
  const known = new Set(
    task ? pending(task.reminders).map((reminder) => reminder.notifyAt) : [],
  );
  const endTime = $derived(
    dueTime && duration !== null ? endTimeOf(dueTime, duration) : "",
  );
  const longest = $derived(dueTime ? maxDuration(dueTime) : 0);

  // A reminder "before the task" needs the task's date and time; without them it is dropped.
  const usable = $derived(
    inputs.filter(
      (input) =>
        input.offsetMinutes === undefined ||
        (dueDate && (dueTime || input.offsetMinutes % 1440 === 0)),
    ),
  );
  const drafts = $derived(
    draftReminders(
      usable,
      dueDate || null,
      dueTime || null,
      settingsStore.notifyTime,
    ),
  );
  const pastReminder = $derived(
    drafts.some(
      (reminder) =>
        !known.has(reminder.notifyAt) && new Date(reminder.notifyAt) <= now(),
    ),
  );
  const titleMissing = $derived(attempted && title.trim() === "");

  function setDate(value: string) {
    dueDate = value;
    if (!value) setStart("");
  }

  /** The duration survives a change of start while it still ends the same day. */
  function setStart(value: string) {
    dueTime = value;
    endError = false;
    if (!value) duration = null;
    else if (duration !== null) {
      duration = Math.min(duration, maxDuration(value));
    }
  }

  function setEnd(value: string) {
    if (!value) {
      duration = null;
      endError = false;
      return;
    }
    const minutes = durationBetween(dueTime, value);
    endError = minutes === null;
    if (minutes !== null) duration = minutes;
  }

  function setPreset(minutes: number | null) {
    duration = minutes;
    endError = false;
  }

  async function submit() {
    attempted = true;
    if (saving || title.trim() === "" || pastReminder || endError) return;
    saving = true;
    const durationMinutes = dueDate && dueTime ? duration : null;
    let saved: boolean;
    if (task) {
      saved = await store.saveDetails(task.id, {
        title,
        dueDate: dueDate || null,
        dueTime: dueDate && dueTime ? dueTime : null,
        durationMinutes,
        tagId,
        ...(remindersTouched && !isDone ? { reminders: usable } : {}),
      });
    } else {
      const contentJson = textToDoc(description);
      saved = await store.createDetailed({
        title,
        ...(dueDate ? { dueDate } : {}),
        ...(dueDate && dueTime ? { dueTime } : {}),
        ...(durationMinutes ? { durationMinutes } : {}),
        ...(tagId ? { tagId } : {}),
        ...(contentJson ? { contentJson } : {}),
        reminders: usable,
      });
    }
    saving = false;
    if (saved) onClose();
  }

  function onFormKeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    const inTitle =
      event.target instanceof HTMLInputElement &&
      event.target.dataset.field === "title";
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey || inTitle)) {
      event.preventDefault();
      void submit();
    }
  }

  // The editor's own time field is hidden here (the dialog has one), so this is never called.
  function setTime(time: string | null) {
    setStart(time ?? "");
  }
</script>

<Modal
  label={editing ? t.newTask.editTitle : t.newTask.title}
  closeLabel={t.newTask.close}
  {onClose}
>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <form
    class="form"
    onsubmit={(event) => {
      event.preventDefault();
      void submit();
    }}
    onkeydown={onFormKeydown}
  >
    <h2>{editing ? t.newTask.editTitle : t.newTask.title}</h2>

    <label class="block">
      <span class="label">{t.newTask.titleLabel}</span>
      <input
        class="text"
        type="text"
        data-field="title"
        data-autofocus
        autocomplete="off"
        maxlength="500"
        placeholder={t.newTask.titlePlaceholder}
        aria-invalid={titleMissing}
        bind:value={title}
      />
    </label>
    {#if titleMissing}
      <p class="error" role="alert">{t.newTask.needsTitle}</p>
    {/if}

    <div class="block">
      <span class="label">{t.tags.taskLabel}</span>
      <TagPicker
        tags={tagsStore.tags}
        value={tagId}
        onChange={(id) => (tagId = id)}
      />
    </div>

    <div class="row">
      <label class="block">
        <span class="label">{t.newTask.dateLabel}</span>
        <span class="inline">
          <input
            type="date"
            value={dueDate}
            onchange={(event) => {
              if (isSettledDate(event.currentTarget.value))
                setDate(event.currentTarget.value);
            }}
          />
          {#if dueDate}
            <button type="button" class="link" onclick={() => setDate("")}>
              {t.newTask.clearDate}
            </button>
          {/if}
        </span>
      </label>
      <label class="block">
        <span class="label">{t.newTask.timeLabel}</span>
        <input
          type="time"
          disabled={!dueDate}
          value={dueTime}
          oninput={(event) => setStart(event.currentTarget.value)}
        />
      </label>
    </div>

    {#if dueDate && dueTime}
      <section class="block" aria-label={t.newTask.durationLabel}>
        <span class="label">{t.newTask.durationLabel}</span>
        <div class="durations">
          <button
            type="button"
            class="chip"
            aria-pressed={duration === null}
            onclick={() => setPreset(null)}
          >
            {t.newTask.noDuration}
          </button>
          {#each DURATION_PRESETS as minutes (minutes)}
            <button
              type="button"
              class="chip"
              aria-pressed={duration === minutes}
              disabled={minutes > longest}
              onclick={() => setPreset(minutes)}
            >
              {durationLabel(minutes)}
            </button>
          {/each}
          <label class="end">
            <span>{t.newTask.endLabel}</span>
            <input
              type="time"
              value={endTime}
              aria-invalid={endError}
              oninput={(event) => setEnd(event.currentTarget.value)}
            />
          </label>
        </div>
        {#if endError}
          <p class="error" role="alert">{t.newTask.endBeforeStart}</p>
        {:else if duration !== null}
          <p class="hint">{t.newTask.lasts(durationLabel(duration))}</p>
        {/if}
      </section>
    {/if}

    {#if !editing}
      <label class="block">
        <span class="label">{t.newTask.descriptionLabel}</span>
        <textarea
          class="text"
          rows="4"
          placeholder={t.newTask.descriptionPlaceholder}
          bind:value={description}></textarea>
      </label>
    {/if}

    <section class="block" aria-label={t.newTask.remindersHeading}>
      <span class="label">{t.newTask.remindersHeading}</span>
      {#if isDone}
        <p class="hint">{t.newTask.doneNoReminders}</p>
      {:else}
        <div class="reminders">
          <ReminderEditor
            dueDate={dueDate || null}
            dueTime={dueTime || null}
            reminders={drafts}
            {now}
            showTime={false}
            onSetTime={setTime}
            onChange={(next) => {
              inputs = next;
              remindersTouched = true;
            }}
          />
        </div>
      {/if}
    </section>

    {#if attempted && pastReminder}
      <p class="error" role="alert">{t.newTask.pastReminder}</p>
    {/if}

    <div class="actions">
      <button type="button" onclick={onClose}>{t.newTask.cancel}</button>
      <button type="submit" class="primary" disabled={saving}>
        {editing ? t.newTask.save : t.newTask.create}
      </button>
    </div>
  </form>
</Modal>

<style>
  .form {
    display: grid;
    gap: var(--space-3);
  }

  h2 {
    margin: 0;
    font-size: 16px;
  }

  .block {
    display: grid;
    gap: var(--space-1);
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }

  .inline {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .label {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  input,
  textarea {
    box-sizing: border-box;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    padding: 6px var(--space-2);
    color-scheme: light dark;
  }

  .text {
    width: 100%;
  }

  textarea {
    resize: vertical;
  }

  input:focus-visible,
  textarea:focus-visible,
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  input[aria-invalid="true"] {
    border-color: var(--danger);
  }

  .reminders {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: var(--bg-elevated);
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
  }

  .hint {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .durations {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .chip {
    padding: 4px 12px;
    font-size: 12px;
  }

  .chip[aria-pressed="true"] {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    font-weight: 600;
  }

  .end {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: auto;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
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

  button.primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 600;
  }

  button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .link {
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--accent);
    font-size: 12px;
  }
</style>
