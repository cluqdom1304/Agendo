<script lang="ts">
  import { t } from "../../i18n";
  import { DAY_CODES } from "../../recurrence";
  import Modal from "../../shell/Modal.svelte";
  import type { HabitsStore } from "../../stores/habits.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { Habit } from "../../types";
  import { ALL_DAYS, indexesOf, maskOf } from "./habits";

  // Creates a habit, or edits `habit` (D-95).
  interface Props {
    store: HabitsStore;
    habit?: Habit;
    onClose: () => void;
  }

  let { store, habit, onClose }: Props = $props();

  // The form starts from the habit and then owns its state.
  // svelte-ignore state_referenced_locally
  let title = $state(habit?.title ?? "");
  // svelte-ignore state_referenced_locally
  let selected = $state<number[]>(indexesOf(habit?.days ?? ALL_DAYS));
  // svelte-ignore state_referenced_locally
  let time = $state(habit?.time ?? "");
  // svelte-ignore state_referenced_locally
  let tagId = $state(habit?.tagId ?? "");
  let attempted = $state(false);
  let saving = $state(false);

  const editing = $derived(habit !== undefined);
  const titleMissing = $derived(attempted && title.trim() === "");
  const daysMissing = $derived(attempted && selected.length === 0);

  function toggleDay(index: number) {
    selected = selected.includes(index)
      ? selected.filter((day) => day !== index)
      : [...selected, index];
  }

  async function submit() {
    attempted = true;
    if (saving || title.trim() === "" || selected.length === 0) return;
    saving = true;
    const days = maskOf(selected);
    const saved = habit
      ? await store.update(habit.id, {
          title,
          days,
          time: time || null,
          tagId: tagId || null,
        })
      : await store.create({
          title,
          days,
          ...(time ? { time } : {}),
          ...(tagId ? { tagId } : {}),
        });
    saving = false;
    if (saved) onClose();
  }

  async function toggleArchived() {
    if (!habit) return;
    const saved = await store.update(habit.id, { archived: !habit.isArchived });
    if (saved) onClose();
  }

  async function remove() {
    if (!habit) return;
    if (await store.remove(habit)) onClose();
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
</script>

<Modal
  label={editing ? t.habits.dialogEdit : t.habits.dialogNew}
  closeLabel={t.habits.close}
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
    <h2>{editing ? t.habits.dialogEdit : t.habits.dialogNew}</h2>

    <label class="block">
      <span class="label">{t.habits.titleLabel}</span>
      <input
        class="text"
        type="text"
        data-field="title"
        data-autofocus
        autocomplete="off"
        maxlength="200"
        placeholder={t.habits.titlePlaceholder}
        aria-invalid={titleMissing}
        bind:value={title}
      />
    </label>
    {#if titleMissing}
      <p class="error" role="alert">{t.habits.needsTitle}</p>
    {/if}

    <div class="block" role="group" aria-label={t.habits.daysLabel}>
      <span class="label">{t.habits.daysLabel}</span>
      <div class="days">
        {#each DAY_CODES as code, index (code)}
          <button
            type="button"
            class="day"
            aria-pressed={selected.includes(index)}
            aria-label={t.recurrence.dayNames[code]}
            title={t.recurrence.dayNames[code]}
            onclick={() => toggleDay(index)}
          >
            {t.recurrence.dayInitials[code]}
          </button>
        {/each}
      </div>
    </div>
    {#if daysMissing}
      <p class="error" role="alert">{t.habits.needsDays}</p>
    {/if}

    <div class="row">
      <label class="block">
        <span class="label">{t.habits.timeLabel}</span>
        <input type="time" bind:value={time} />
      </label>
      <label class="block">
        <span class="label">{t.habits.tagLabel}</span>
        <select class="text" bind:value={tagId}>
          <option value="">{t.habits.noTag}</option>
          {#each tagsStore.tags as tag (tag.id)}
            <option value={tag.id}>{tag.name}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="actions">
      {#if habit}
        <button type="button" class="btn danger" onclick={() => void remove()}>
          {t.habits.delete}
        </button>
        <button type="button" class="btn" onclick={() => void toggleArchived()}>
          {habit.isArchived ? t.habits.unarchive : t.habits.archive}
        </button>
      {/if}
      <span class="spacer"></span>
      <button type="button" class="btn" onclick={onClose}>
        {t.habits.cancel}
      </button>
      <button type="submit" class="btn primary" disabled={saving}>
        {editing ? t.habits.save : t.habits.create}
      </button>
    </div>
  </form>
</Modal>

<style>
  .form {
    display: grid;
    gap: var(--space-4);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.01em;
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

  .label {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
  }

  .text {
    width: 100%;
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
  }

  .days {
    display: flex;
    gap: var(--space-2);
  }

  .day {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
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

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .spacer {
    flex: 1;
  }
</style>
