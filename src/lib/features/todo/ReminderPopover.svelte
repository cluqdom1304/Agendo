<script lang="ts">
  import { t } from "../../i18n";
  import {
    PRESET_OFFSETS,
    customToInstant,
    dueInstant,
    formatMoment,
    offsetIsAhead,
    offsetLabel,
    pending,
    togglePreset,
    withCustom,
    withoutReminder,
  } from "../../reminders";
  import type { Reminder, ReminderInput } from "../../types";

  // F-12: "En el momento", "15 min antes", "1 hora antes", "1 día antes" or a custom time.
  interface Props {
    title: string;
    dueDate: string | null;
    dueTime: string | null;
    reminders: Reminder[];
    now?: () => Date;
    onSetTime: (time: string | null) => void;
    onChange: (reminders: ReminderInput[]) => void;
    onClose: () => void;
  }

  let {
    title,
    dueDate,
    dueTime,
    reminders,
    now = () => new Date(),
    onSetTime,
    onChange,
    onClose,
  }: Props = $props();

  let custom = $state("");

  const active = $derived(pending(reminders));
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

  function isOn(minutes: number): boolean {
    return active.some((reminder) => reminder.offsetMinutes === minutes);
  }

  function addCustom() {
    if (customInstant === null || !customAhead) return;
    onChange(withCustom(reminders, customInstant));
    custom = "";
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.stopPropagation();
    onClose();
  }
</script>

<div
  class="popover"
  role="dialog"
  aria-label={t.reminders.open(title)}
  tabindex="-1"
  onkeydown={onKeydown}
>
  <h5>{t.reminders.heading}</h5>

  {#if dueDate !== null}
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
    <button
      type="button"
      class="chip"
      disabled={!customAhead}
      onclick={addCustom}
    >
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
</div>

<style>
  .popover {
    display: grid;
    gap: var(--space-2);
    margin: var(--space-1) var(--space-3) var(--space-2) 36px;
    padding: var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: var(--bg-elevated);
    outline: none;
  }

  h5 {
    margin: 0;
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

  input {
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    padding: 2px var(--space-2);
    color-scheme: light dark;
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
