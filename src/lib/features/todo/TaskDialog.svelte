<script lang="ts">
  import { isSettledDate, toLocalIsoDate } from "../../dates";
  import { t } from "../../i18n";
  import { draftReminders, pending, toInput } from "../../reminders";
  import {
    buildRule,
    clampCount,
    clampInterval,
    DAY_CODES,
    FREQS,
    parseRule,
    type DayCode,
    type Freq,
  } from "../../recurrence";
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
  import { CaptureField, captureOptions } from "../capture/capture.svelte";
  import CapturePreview from "../capture/CapturePreview.svelte";
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
  let tagIds = $state<string[]>(task?.tagIds ?? []);
  // svelte-ignore state_referenced_locally
  let priority = $state(task?.priority ?? 0);
  // svelte-ignore state_referenced_locally
  const startRepeat = parseRule(task?.rrule);
  let repeatFreq = $state<Freq | "">(startRepeat?.freq ?? "");
  let repeatInterval = $state<number | null>(startRepeat?.interval ?? 1);
  let repeatUntil = $state(startRepeat?.until ?? "");
  let repeatDays = $state<DayCode[]>(startRepeat?.days ?? []);
  let repeatCount = $state<number | null>(startRepeat?.count ?? null);

  function toggleDay(code: DayCode) {
    repeatDays = repeatDays.includes(code)
      ? repeatDays.filter((day) => day !== code)
      : [...repeatDays, code];
  }
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

  // Only a new task reads its title (F23-02): an existing title is never reinterpreted.
  // svelte-ignore state_referenced_locally
  const field = task
    ? null
    : new CaptureField(() =>
        captureOptions(toLocalIsoDate(now()), dueDate || undefined),
      );

  /** Copies what the title asks for into the fields; what is typed wins over them. */
  function applyCapture() {
    const capture = field?.capture;
    if (!field || !capture) return;
    title = capture.title;
    field.settle(title);
    if (capture.date) setDate(capture.date);
    if (capture.time) setStart(capture.time);
    if (capture.duration) setPreset(capture.duration);
    for (const tag of capture.tags) {
      if (!tagIds.includes(tag.id)) tagIds = [...tagIds, tag.id];
    }
    if (capture.priority > 0) priority = capture.priority;
    const repeat = parseRule(capture.rrule);
    if (repeat) {
      repeatFreq = repeat.freq;
      repeatInterval = repeat.interval;
      repeatDays = repeat.days;
      repeatUntil = "";
      repeatCount = null;
    }
  }

  function onTitleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && field?.ignore()) {
      event.preventDefault();
      event.stopPropagation();
    }
  }

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

  // A repetition needs a date: clearing the date ends it.
  $effect(() => {
    if (!dueDate) repeatFreq = "";
  });
  const untilBefore = $derived(
    repeatFreq !== "" &&
      repeatUntil !== "" &&
      dueDate !== "" &&
      repeatUntil < dueDate,
  );
  const rule = $derived(
    dueDate && repeatFreq
      ? buildRule({
          freq: repeatFreq,
          interval: clampInterval(repeatInterval ?? 1),
          until: repeatUntil,
          days: repeatDays,
          count: repeatCount === null ? null : clampCount(repeatCount),
        })
      : null,
  );

  async function submit() {
    applyCapture();
    attempted = true;
    if (
      saving ||
      title.trim() === "" ||
      pastReminder ||
      endError ||
      untilBefore
    )
      return;
    saving = true;
    const durationMinutes = dueDate && dueTime ? duration : null;
    let saved: boolean;
    if (task) {
      saved = await store.saveDetails(task.id, {
        title,
        dueDate: dueDate || null,
        dueTime: dueDate && dueTime ? dueTime : null,
        durationMinutes,
        tagIds,
        rrule: rule,
        priority,
        ...(remindersTouched && !isDone ? { reminders: usable } : {}),
      });
    } else {
      const contentJson = textToDoc(description);
      saved = await store.createDetailed({
        title,
        ...(dueDate ? { dueDate } : {}),
        ...(dueDate && dueTime ? { dueTime } : {}),
        ...(durationMinutes ? { durationMinutes } : {}),
        ...(tagIds.length > 0 ? { tagIds } : {}),
        ...(contentJson ? { contentJson } : {}),
        ...(rule ? { rrule: rule } : {}),
        ...(priority > 0 ? { priority } : {}),
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
        oninput={(event) => field?.set(event.currentTarget.value)}
        onkeydown={onTitleKeydown}
        onblur={applyCapture}
      />
    </label>
    {#if field}
      <CapturePreview capture={field.capture} today={field.today()} />
    {/if}
    {#if titleMissing}
      <p class="error" role="alert">{t.newTask.needsTitle}</p>
    {/if}

    <div class="block">
      <span class="label">{t.tags.taskLabel}</span>
      <TagPicker
        tags={tagsStore.tags}
        values={tagIds}
        onChange={(ids) => (tagIds = ids)}
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
            <button
              type="button"
              class="btn small quiet"
              onclick={() => setDate("")}
            >
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
            class="toggle-chip"
            aria-pressed={duration === null}
            onclick={() => setPreset(null)}
          >
            {t.newTask.noDuration}
          </button>
          {#each DURATION_PRESETS as minutes (minutes)}
            <button
              type="button"
              class="toggle-chip"
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

    <section class="block" aria-label={t.priority.label}>
      <span class="label">{t.priority.label}</span>
      <div class="durations">
        {#each t.priority.levels as name, level (level)}
          <button
            type="button"
            class="toggle-chip priority"
            aria-pressed={priority === level}
            onclick={() => (priority = level)}
          >
            <span class="dot" style:--level="var(--priority-{level})"></span>
            {name}
          </button>
        {/each}
      </div>
    </section>

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

    <div class="block">
      <span class="label">{t.recurrence.label}</span>
      <div class="repeat">
        <select
          class="text"
          aria-label={t.recurrence.taskLabel}
          disabled={!dueDate}
          bind:value={repeatFreq}
        >
          <option value="">{t.recurrence.never}</option>
          {#each FREQS as freq (freq)}
            <option value={freq}>{t.recurrence.units[freq].one}</option>
          {/each}
        </select>
        {#if repeatFreq}
          <label class="block">
            <span class="label">
              {t.recurrence.intervalLabel}
              ({t.recurrence.units[repeatFreq].many})
            </span>
            <input
              class="text"
              type="number"
              min="1"
              max="365"
              step="1"
              bind:value={repeatInterval}
            />
          </label>
          {#if repeatFreq === "WEEKLY"}
            <div class="days" role="group" aria-label={t.recurrence.daysLabel}>
              {#each DAY_CODES as code (code)}
                <button
                  type="button"
                  class="day"
                  aria-pressed={repeatDays.includes(code)}
                  aria-label={t.recurrence.dayNames[code]}
                  title={t.recurrence.dayNames[code]}
                  onclick={() => toggleDay(code)}
                >
                  {t.recurrence.dayInitials[code]}
                </button>
              {/each}
            </div>
          {/if}
          <label class="block">
            <span class="label">{t.recurrence.untilLabel}</span>
            <input
              class="text"
              type="date"
              bind:value={repeatUntil}
              oninput={() => (repeatCount = null)}
            />
          </label>
          <label class="block">
            <span class="label">{t.recurrence.countLabel}</span>
            <input
              class="text"
              type="number"
              min="1"
              max="999"
              step="1"
              bind:value={repeatCount}
              oninput={() => (repeatUntil = "")}
            />
          </label>
        {/if}
      </div>
      {#if !dueDate}
        <p class="hint">{t.recurrence.needsDate}</p>
      {:else if untilBefore}
        <p class="hint error" role="alert">{t.recurrence.untilBefore}</p>
      {/if}
    </div>

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
      <button type="button" class="btn" onclick={onClose}>
        {t.newTask.cancel}
      </button>
      <button type="submit" class="btn primary" disabled={saving}>
        {editing ? t.newTask.save : t.newTask.create}
      </button>
    </div>
  </form>
</Modal>

<style>
  .form {
    display: grid;
    gap: var(--space-4);
  }

  .block {
    display: grid;
    gap: 6px;
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
    font-weight: 500;
  }

  .text {
    width: 100%;
  }

  textarea {
    padding: 6px var(--space-2);
    resize: vertical;
  }

  .reminders {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-card);
    background: var(--bg-control);
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

  .repeat {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3);
  }

  .days {
    display: flex;
    gap: var(--space-2);
  }

  .day {
    width: 32px;
    height: 32px;
    border: 0;
    background: var(--bg-control);
    border-radius: 50%;
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .day:hover {
    background: var(--bg-hover);
  }

  .day[aria-pressed="true"] {
    border-color: var(--ink);
    background: var(--ink);
    color: var(--on-ink);
  }

  .durations {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .priority {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  /* The ring of the checkbox, small, so the level reads the same here and in the list. */
  .dot {
    width: 9px;
    height: 9px;
    box-sizing: border-box;
    border: 1.5px solid var(--level);
    border-radius: var(--radius-chip);
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
    margin-top: var(--space-1);
  }
</style>
