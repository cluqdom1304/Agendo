<script lang="ts">
  import { isSettledDate } from "../../dates";
  import { t } from "../../i18n";
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
  import { offsetLabel, presetOffsets } from "../../reminders";
  import Modal from "../../shell/Modal.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { CalendarEvent } from "../../types";
  import {
    MINUTES_PER_DAY,
    addDays,
    daysBetween,
    minutesToTime,
    timeToMinutes,
  } from "../calendar/grid";
  import TagPicker from "../tags/TagPicker.svelte";

  // Creates an event, or edits `event` (D-94). Repeating events are edited as a series.
  interface Props {
    store: EventsStore;
    event?: CalendarEvent;
    /** First day of a new event, e.g. the one the calendar is showing. */
    defaultDate: string;
    /** Start of a new event; without it the event starts at 09:00. */
    defaultTime?: string | null;
    onClose: () => void;
    /** Turns the event into a task; only offered when editing. */
    onConvert?: (event: CalendarEvent) => void;
  }

  let {
    store,
    event,
    defaultDate,
    defaultTime = null,
    onClose,
    onConvert,
  }: Props = $props();

  const FIRST_START = "09:00";
  const DEFAULT_LENGTH = 60;

  /** An hour after `time`, never past the end of the day. */
  function hourAfter(time: string): string {
    return minutesToTime(
      Math.min(timeToMinutes(time) + DEFAULT_LENGTH, MINUTES_PER_DAY - 1),
    );
  }

  // The form starts from the event and then owns its state.
  // svelte-ignore state_referenced_locally
  let title = $state(event?.title ?? "");
  // svelte-ignore state_referenced_locally
  let notes = $state(event?.notes ?? "");
  // svelte-ignore state_referenced_locally
  let tagIds = $state<string[]>(event?.tagIds ?? []);
  // svelte-ignore state_referenced_locally
  let allDay = $state(event?.allDay ?? false);
  // svelte-ignore state_referenced_locally
  let startDate = $state(event?.startDate ?? defaultDate);
  // svelte-ignore state_referenced_locally
  let startTime = $state(event?.startTime ?? defaultTime ?? FIRST_START);
  // svelte-ignore state_referenced_locally
  let endDate = $state(event?.endDate ?? event?.startDate ?? defaultDate);
  // svelte-ignore state_referenced_locally
  let endTime = $state(event?.endTime ?? hourAfter(startTime));
  // svelte-ignore state_referenced_locally
  let offsets = $state<number[]>(event?.reminderOffsets ?? []);
  // svelte-ignore state_referenced_locally
  const startRepeat = parseRule(event?.rrule);
  let repeatFreq = $state<Freq | "">(startRepeat?.freq ?? "");
  let repeatInterval = $state<number | null>(startRepeat?.interval ?? 1);
  let repeatUntil = $state(startRepeat?.until ?? "");
  let repeatDays = $state<DayCode[]>(startRepeat?.days ?? []);
  let repeatCount = $state<number | null>(startRepeat?.count ?? null);
  let attempted = $state(false);
  let saving = $state(false);

  const editing = $derived(event !== undefined);
  const titleMissing = $derived(attempted && title.trim() === "");
  const endBeforeStart = $derived(
    endDate < startDate ||
      (!allDay && endDate === startDate && endTime <= startTime),
  );
  const untilBefore = $derived(
    repeatFreq !== "" && repeatUntil !== "" && repeatUntil < startDate,
  );
  const rule = $derived(
    repeatFreq
      ? buildRule({
          freq: repeatFreq,
          interval: clampInterval(repeatInterval ?? 1),
          until: repeatUntil,
          days: repeatDays,
          count: repeatCount === null ? null : clampCount(repeatCount),
        })
      : null,
  );
  const presets = $derived(presetOffsets(!allDay));
  const known = $derived(
    [...new Set([...presets, ...offsets])].toSorted((a, b) => a - b),
  );

  function toggleDay(code: DayCode) {
    repeatDays = repeatDays.includes(code)
      ? repeatDays.filter((day) => day !== code)
      : [...repeatDays, code];
  }

  function toggleOffset(minutes: number) {
    offsets = offsets.includes(minutes)
      ? offsets.filter((offset) => offset !== minutes)
      : [...offsets, minutes];
  }

  /** The end moves with the start, so the length of the event stays. */
  function setStartDate(value: string) {
    if (!isSettledDate(value)) return;
    endDate = addDays(endDate, daysBetween(startDate, value));
    startDate = value;
  }

  function setStartTime(value: string) {
    if (!value) return;
    if (endDate === startDate) {
      const length = timeToMinutes(endTime) - timeToMinutes(startTime);
      endTime = minutesToTime(
        Math.min(
          timeToMinutes(value) + Math.max(length, 0),
          MINUTES_PER_DAY - 1,
        ),
      );
    }
    startTime = value;
  }

  async function submit() {
    attempted = true;
    if (saving || title.trim() === "" || endBeforeStart || untilBefore) return;
    saving = true;
    // A whole-day event carries no times, and an offset in minutes needs a time to count from.
    const usable = allDay
      ? offsets.filter((offset) => offset % MINUTES_PER_DAY === 0)
      : offsets;
    const saved = event
      ? await store.update(event.id, {
          title,
          notes,
          startDate,
          endDate,
          startTime: allDay ? null : startTime,
          ...(allDay ? {} : { endTime }),
          rrule: rule,
          tagIds,
          reminderOffsets: usable,
        })
      : await store.create({
          title,
          ...(notes.trim() ? { notes } : {}),
          startDate,
          endDate,
          ...(allDay ? {} : { startTime, endTime }),
          ...(rule ? { rrule: rule } : {}),
          ...(tagIds.length > 0 ? { tagIds } : {}),
          ...(usable.length > 0 ? { reminderOffsets: usable } : {}),
        });
    saving = false;
    if (saved) onClose();
  }

  async function remove() {
    if (!event) return;
    if (await store.remove(event)) onClose();
  }

  function onFormKeydown(keyEvent: KeyboardEvent) {
    if (keyEvent.isComposing) return;
    const inTitle =
      keyEvent.target instanceof HTMLInputElement &&
      keyEvent.target.dataset.field === "title";
    if (
      keyEvent.key === "Enter" &&
      (keyEvent.ctrlKey || keyEvent.metaKey || inTitle)
    ) {
      keyEvent.preventDefault();
      void submit();
    }
  }
</script>

<Modal
  label={editing ? t.events.dialogEdit : t.events.dialogNew}
  closeLabel={t.events.close}
  {onClose}
>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <form
    class="form"
    onsubmit={(submitEvent) => {
      submitEvent.preventDefault();
      void submit();
    }}
    onkeydown={onFormKeydown}
  >
    <h2>{editing ? t.events.dialogEdit : t.events.dialogNew}</h2>

    <label class="block">
      <span class="label">{t.events.titleLabel}</span>
      <input
        class="text"
        type="text"
        data-field="title"
        data-autofocus
        autocomplete="off"
        maxlength="500"
        placeholder={t.events.titlePlaceholder}
        aria-invalid={titleMissing}
        bind:value={title}
      />
    </label>
    {#if titleMissing}
      <p class="error" role="alert">{t.events.needsTitle}</p>
    {/if}

    <div class="block">
      <span class="label">{t.tags.taskLabel}</span>
      <TagPicker
        tags={tagsStore.tags}
        values={tagIds}
        onChange={(ids) => (tagIds = ids)}
      />
    </div>

    <label class="check">
      <input type="checkbox" bind:checked={allDay} />
      {t.events.allDayLabel}
    </label>

    <div class="row">
      <label class="block">
        <span class="label">{t.events.startLabel}</span>
        <input
          type="date"
          value={startDate}
          onchange={(changeEvent) =>
            setStartDate(changeEvent.currentTarget.value)}
        />
      </label>
      {#if !allDay}
        <label class="block">
          <span class="label"
            >{t.events.startLabel} · {t.newTask.timeLabel}</span
          >
          <input
            type="time"
            value={startTime}
            oninput={(inputEvent) =>
              setStartTime(inputEvent.currentTarget.value)}
          />
        </label>
      {/if}
    </div>
    <div class="row">
      <label class="block">
        <span class="label">{t.events.endLabel}</span>
        <input
          type="date"
          value={endDate}
          onchange={(changeEvent) => {
            if (isSettledDate(changeEvent.currentTarget.value))
              endDate = changeEvent.currentTarget.value;
          }}
        />
      </label>
      {#if !allDay}
        <label class="block">
          <span class="label">{t.events.endLabel} · {t.newTask.timeLabel}</span>
          <input
            type="time"
            value={endTime}
            oninput={(inputEvent) => {
              if (inputEvent.currentTarget.value)
                endTime = inputEvent.currentTarget.value;
            }}
          />
        </label>
      {/if}
    </div>
    {#if attempted && endBeforeStart}
      <p class="error" role="alert">{t.events.endBeforeStart}</p>
    {/if}

    <div class="block">
      <span class="label">{t.recurrence.label}</span>
      <div class="repeat">
        <select
          class="text"
          aria-label={t.recurrence.taskLabel}
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
      {#if untilBefore}
        <p class="error" role="alert">{t.recurrence.untilBefore}</p>
      {:else if editing && event?.rrule}
        <p class="hint">{t.events.seriesHint}</p>
      {/if}
    </div>

    <section class="block" aria-label={t.events.remindersLabel}>
      <span class="label">{t.events.remindersLabel}</span>
      <div class="chips">
        {#each known as minutes (minutes)}
          <button
            type="button"
            class="toggle-chip"
            aria-pressed={offsets.includes(minutes)}
            onclick={() => toggleOffset(minutes)}
          >
            {offsetLabel(minutes, allDay)}
          </button>
        {/each}
      </div>
      {#if offsets.length === 0}
        <p class="hint">{t.events.noReminders}</p>
      {/if}
    </section>

    <label class="block">
      <span class="label">{t.events.notesLabel}</span>
      <textarea
        class="text"
        rows="3"
        placeholder={t.events.notesPlaceholder}
        bind:value={notes}></textarea>
    </label>

    <div class="actions">
      {#if event}
        <button type="button" class="btn danger" onclick={() => void remove()}>
          {t.events.delete}
        </button>
        {#if onConvert}
          <button type="button" class="btn" onclick={() => onConvert(event)}>
            {t.events.toTask}
          </button>
        {/if}
      {/if}
      <span class="spacer"></span>
      <button type="button" class="btn" onclick={onClose}>
        {t.events.cancel}
      </button>
      <button type="submit" class="btn primary" disabled={saving}>
        {editing ? t.events.save : t.events.create}
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
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .check {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 13px;
  }

  .label {
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
  }

  .text {
    width: 100%;
  }

  textarea {
    padding: 6px var(--space-2);
    resize: vertical;
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: var(--text-caption);
  }

  .hint {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-caption);
  }

  .repeat {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3);
  }

  .days {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .day {
    flex: none;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-caption);
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      color var(--motion-fast) var(--ease-out),
      transform var(--motion-fast) var(--ease-spring);
  }

  .day:hover {
    background: var(--bg-control-hover);
  }

  .day:active {
    transform: scale(0.92);
    transition-duration: var(--motion-instant);
  }

  .day[aria-pressed="true"] {
    border-color: var(--ink);
    background: var(--ink);
    color: var(--on-ink);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .spacer {
    flex: 1;
  }
</style>
