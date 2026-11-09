<script lang="ts">
  import { toLocalIsoDate } from "../../dates";
  import { t } from "../../i18n";
  import {
    PRESET_OFFSETS,
    customToInstant,
    describeRepeat,
    dueInstant,
    formatMoment,
    offsetIsAhead,
    offsetLabel,
    pending,
    sent,
    togglePreset,
    withCustom,
    withoutReminder,
  } from "../../reminders";
  import type {
    Reminder,
    ReminderInput,
    Repeat,
    RepeatUnit,
  } from "../../types";

  // F-12 and D-32: "En el momento", "15 min antes", "1 hora antes", "1 día antes", or a custom
  // time that may repeat. The same editor serves saved tasks (the popover of a row) and the
  // dialog that creates one, where the reminders are drafts.
  interface Props {
    dueDate: string | null;
    dueTime: string | null;
    reminders: Reminder[];
    now?: () => Date;
    /** Hide the task-time field when the caller already shows one. */
    showTime?: boolean;
    onSetTime: (time: string | null) => void;
    onChange: (reminders: ReminderInput[]) => void;
  }

  /** Sounded reminders kept on screen; older ones are summarised. */
  const HISTORY_SHOWN = 5;

  type RepeatMode = "none" | "daily" | "weekly" | "monthly" | "custom";
  type EndKind = "never" | "count" | "until";

  let {
    dueDate,
    dueTime,
    reminders,
    now = () => new Date(),
    showTime = true,
    onSetTime,
    onChange,
  }: Props = $props();

  let custom = $state("");
  let mode = $state<RepeatMode>("none");
  let every = $state(1);
  let unit = $state<RepeatUnit>("day");
  let endKind = $state<EndKind>("never");
  let count = $state(5);
  let until = $state("");

  const UNITS: RepeatUnit[] = ["hour", "day", "week", "month"];
  const MODE_OPTIONS: { value: RepeatMode; label: string }[] = [
    { value: "none", label: t.reminders.repeatNone },
    { value: "daily", label: t.reminders.repeatDaily },
    { value: "weekly", label: t.reminders.repeatWeekly },
    { value: "monthly", label: t.reminders.repeatMonthly },
    { value: "custom", label: t.reminders.repeatCustom },
  ];

  const active = $derived(pending(reminders));
  const history = $derived(sent(reminders));
  const hiddenHistory = $derived(Math.max(0, history.length - HISTORY_SHOWN));
  const shownHistory = $derived(history.slice(hiddenHistory));
  const due = $derived(dueInstant(dueDate, dueTime));
  const customInstant = $derived(customToInstant(custom));
  const customAhead = $derived(
    customInstant !== null && new Date(customInstant) > now(),
  );
  const hint = $derived(
    dueDate === null
      ? t.reminders.needsSchedule
      : dueTime === null
        ? t.reminders.needsTime
        : null,
  );

  const interval = $derived(
    mode === "daily"
      ? { every: 1, unit: "day" as RepeatUnit }
      : mode === "weekly"
        ? { every: 1, unit: "week" as RepeatUnit }
        : mode === "monthly"
          ? { every: 1, unit: "month" as RepeatUnit }
          : { every, unit },
  );
  const repeat = $derived<Omit<Repeat, "occurrence"> | undefined>(
    mode === "none"
      ? undefined
      : {
          ...interval,
          count: endKind === "count" ? count : null,
          until: endKind === "until" ? until : null,
        },
  );
  const repeatValid = $derived(
    repeat === undefined ||
      (Number.isInteger(repeat.every) &&
        repeat.every >= 1 &&
        repeat.every <= 365 &&
        (repeat.count === null ||
          (Number.isInteger(repeat.count) &&
            repeat.count >= 2 &&
            repeat.count <= 1000)) &&
        (repeat.until === null ||
          (/^\d{4}-\d{2}-\d{2}$/.test(repeat.until) &&
            custom !== "" &&
            repeat.until >= toLocalIsoDate(new Date(custom))))),
  );
  const canAdd = $derived(customAhead && repeatValid);

  function isOn(minutes: number): boolean {
    return active.some((reminder) => reminder.offsetMinutes === minutes);
  }

  function addCustom() {
    if (customInstant === null || !canAdd) return;
    onChange(withCustom(reminders, customInstant, repeat));
    custom = "";
    mode = "none";
    endKind = "never";
  }

  function numberFrom(event: Event & { currentTarget: HTMLInputElement }) {
    return event.currentTarget.valueAsNumber;
  }
</script>

{#if showTime && dueDate !== null}
  <label class="field">
    <span>{t.reminders.timeLabel}</span>
    <input
      type="time"
      value={dueTime ?? ""}
      onchange={(event) => onSetTime(event.currentTarget.value || null)}
    />
    {#if dueTime === null}
      <span class="muted">{t.reminders.noTime}</span>
    {/if}
  </label>
{/if}

<div class="presets">
  {#each PRESET_OFFSETS as minutes (minutes)}
    {@const ahead = offsetIsAhead(due, minutes, now())}
    <button
      type="button"
      class="chip"
      aria-pressed={isOn(minutes)}
      disabled={!isOn(minutes) && !ahead}
      title={due !== null && !ahead && !isOn(minutes)
        ? t.reminders.pastOption
        : undefined}
      onclick={() => onChange(togglePreset(reminders, minutes))}
    >
      {offsetLabel(minutes)}
    </button>
  {/each}
</div>
{#if hint}
  <p class="muted">{hint}</p>
{/if}

<div class="custom">
  <label class="field">
    <span>{t.reminders.customLabel}</span>
    <input type="datetime-local" bind:value={custom} />
  </label>

  <label class="field">
    <span>{t.reminders.repeatLabel}</span>
    <select bind:value={mode}>
      {#each MODE_OPTIONS as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </label>

  {#if mode === "custom"}
    <div class="field">
      <span>{t.reminders.repeatEvery}</span>
      <input
        class="number"
        type="number"
        min="1"
        max="365"
        aria-label={t.reminders.repeatEvery}
        value={every}
        oninput={(event) => (every = numberFrom(event))}
      />
      <select aria-label={t.reminders.repeatUnit} bind:value={unit}>
        {#each UNITS as option (option)}
          <option value={option}>{t.reminders.units[option].option}</option>
        {/each}
      </select>
    </div>
  {/if}

  {#if mode !== "none"}
    <div class="field">
      <span>{t.reminders.repeatEnds}</span>
      <select aria-label={t.reminders.repeatEnds} bind:value={endKind}>
        <option value="never">{t.reminders.endNever}</option>
        <option value="count">{t.reminders.endCount}</option>
        <option value="until">{t.reminders.endUntil}</option>
      </select>
      {#if endKind === "count"}
        <input
          class="number"
          type="number"
          min="2"
          max="1000"
          aria-label={t.reminders.endCountValue}
          value={count}
          oninput={(event) => (count = numberFrom(event))}
        />
        <span>{t.reminders.timesSuffix}</span>
      {:else if endKind === "until"}
        <input
          type="date"
          aria-label={t.reminders.endUntilValue}
          bind:value={until}
        />
      {/if}
    </div>
  {/if}

  <button type="button" class="chip" disabled={!canAdd} onclick={addCustom}>
    {t.reminders.customAdd}
  </button>
</div>

{#if active.length > 0}
  <ul class="list">
    {#each active as reminder (reminder.id)}
      {@const when = formatMoment(reminder.notifyAt)}
      <li>
        <span>
          {reminder.offsetMinutes === null
            ? when
            : `${offsetLabel(reminder.offsetMinutes)} · ${when}`}
          {#if reminder.repeat}
            <span class="muted">
              · {describeRepeat(reminder.repeat)}{reminder.repeat.count !== null
                ? ` (${t.reminders.occurrence(reminder.repeat.occurrence, reminder.repeat.count)})`
                : ""}
            </span>
          {/if}
        </span>
        <button
          type="button"
          class="remove"
          aria-label={t.reminders.remove(when)}
          title={t.reminders.remove(when)}
          onclick={() => onChange(withoutReminder(reminders, reminder.id))}
        >
          &times;
        </button>
      </li>
    {/each}
  </ul>
{/if}

{#if history.length > 0}
  <h6>{t.reminders.sentHeading}</h6>
  <ul class="list history" aria-label={t.reminders.sentHeading}>
    {#if hiddenHistory > 0}
      <li class="muted">{t.reminders.earlier(hiddenHistory)}</li>
    {/if}
    {#each shownHistory as reminder (reminder.id)}
      <li class="done" data-sent>
        <span>
          {reminder.offsetMinutes === null
            ? formatMoment(reminder.notifyAt)
            : `${offsetLabel(reminder.offsetMinutes)} · ${formatMoment(reminder.notifyAt)}`}
        </span>
        <span class="badge">{t.reminders.sentBadge}</span>
      </li>
    {/each}
  </ul>
{/if}

<style>
  h6 {
    margin: var(--space-1) 0 0;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .field {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 12px;
    color: var(--text-secondary);
  }

  input,
  select {
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    padding: 2px var(--space-2);
    color-scheme: light dark;
  }

  select option {
    background: var(--bg-window-solid);
    color: var(--text-primary);
  }

  .number {
    width: 56px;
  }

  .presets,
  .custom {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .chip {
    padding: 2px var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .chip[aria-pressed="true"] {
    border-color: var(--accent);
    color: var(--accent);
  }

  .chip:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .muted {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .list {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 12px;
  }

  .list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  /* Already sounded: same rows, visibly set apart from the ones still to come. */
  .done {
    color: var(--text-secondary);
    opacity: 0.75;
  }

  .badge {
    padding: 0 var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-chip);
    font-size: 11px;
  }

  .remove {
    border: 0;
    background: transparent;
    color: var(--text-secondary);
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
  }

  .remove:hover {
    color: var(--danger);
  }
</style>
