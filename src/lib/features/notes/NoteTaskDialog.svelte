<script lang="ts">
  import { t } from "../../i18n";
  import { offsetLabel, presetOffsets } from "../../reminders";
  import Modal from "../../shell/Modal.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Task } from "../../types";
  import TagPicker from "../tags/TagPicker.svelte";

  // The small dialog behind `/tarea` and Ctrl+Mayús+T: just what a task needs to exist, with
  // the note it comes from. The rest (duration, repeats) is edited later from the task itself.
  interface Props {
    tasks: TasksStore;
    noteId: string;
    /** Text to start the title with: the selection, or the line the user was on. */
    prefill?: string;
    /** The tags of the note, offered as the starting ones. */
    tagIds?: string[];
    onCreated: (task: Task) => void;
    onClose: () => void;
  }

  let {
    tasks,
    noteId,
    prefill = "",
    tagIds: noteTagIds = [],
    onCreated,
    onClose,
  }: Props = $props();

  // The form starts from the props and then owns its state.
  // svelte-ignore state_referenced_locally
  let title = $state(prefill);
  let dueDate = $state("");
  let dueTime = $state("");
  // svelte-ignore state_referenced_locally
  let tagIds = $state<string[]>([...noteTagIds]);
  /** Minutes before, as text so "none" fits in the same select. */
  let reminder = $state("");
  let missingTitle = $state(false);
  let saving = $state(false);

  const offsets = $derived(presetOffsets(dueTime !== ""));

  // A reminder needs a date, and an untimed task only allows whole days: drop what no longer fits.
  $effect(() => {
    if (
      dueDate === "" ||
      (reminder !== "" && !offsets.includes(Number(reminder)))
    ) {
      reminder = "";
    }
  });
  $effect(() => {
    if (dueDate === "") dueTime = "";
  });

  async function submit() {
    if (saving) return;
    if (title.trim() === "") {
      missingTitle = true;
      return;
    }
    saving = true;
    const created = await tasks.create({
      title,
      ...(dueDate ? { dueDate } : {}),
      ...(dueDate && dueTime ? { dueTime } : {}),
      ...(tagIds.length > 0 ? { tagIds } : {}),
      ...(reminder !== ""
        ? { reminders: [{ offsetMinutes: Number(reminder) }] }
        : {}),
      sourceNoteId: noteId,
    });
    saving = false;
    if (!created) return;
    onCreated(created);
    onClose();
  }
</script>

<Modal
  label={t.notesApp.taskDialogTitle}
  closeLabel={t.newTask.close}
  {onClose}
>
  <form
    class="form"
    onsubmit={(event) => {
      event.preventDefault();
      void submit();
    }}
  >
    <h2>{t.notesApp.taskDialogTitle}</h2>

    <label class="block">
      <span class="label">{t.newTask.titleLabel}</span>
      <input
        class="text"
        type="text"
        data-autofocus
        autocomplete="off"
        maxlength="500"
        placeholder={t.newTask.titlePlaceholder}
        bind:value={title}
        oninput={() => (missingTitle = false)}
      />
    </label>
    {#if missingTitle}
      <p class="error" role="alert">{t.newTask.needsTitle}</p>
    {/if}

    <div class="row">
      <label class="block">
        <span class="label">{t.newTask.dateLabel}</span>
        <input class="text" type="date" bind:value={dueDate} />
      </label>
      <label class="block">
        <span class="label">{t.newTask.timeLabel}</span>
        <input
          class="text"
          type="time"
          disabled={dueDate === ""}
          bind:value={dueTime}
        />
      </label>
    </div>

    <label class="block">
      <span class="label">{t.notesApp.reminderLabel}</span>
      <select class="text" bind:value={reminder} disabled={dueDate === ""}>
        <option value="">{t.notesApp.noReminder}</option>
        {#each offsets as minutes (minutes)}
          <option value={String(minutes)}>
            {offsetLabel(minutes, dueTime === "")}
          </option>
        {/each}
      </select>
    </label>

    <div class="block">
      <span class="label">{t.tags.taskLabel}</span>
      <TagPicker
        tags={tagsStore.tags}
        values={tagIds}
        onChange={(ids) => (tagIds = ids)}
      />
    </div>

    <div class="actions">
      <button type="button" class="btn" onclick={onClose}>
        {t.newTask.cancel}
      </button>
      <button type="submit" class="btn primary" disabled={saving}>
        {t.notesApp.taskDialogCreate}
      </button>
    </div>
  </form>
</Modal>

<style>
  .form {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-4);
  }

  .block {
    display: grid;
    min-width: 0;
    gap: 6px;
  }

  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
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

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
  }
</style>
