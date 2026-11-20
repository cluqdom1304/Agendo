<script lang="ts">
  import { onMount } from "svelte";

  import { toInstant, toLocalIsoDate } from "../../dates";
  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { NoteReminder } from "../../types";
  import { formatNoteStamp } from "./format";

  // The reminder of a note (F13-04): one date and time at which the scheduler of the tasks
  // also notifies about the note.
  interface Props {
    notes: NotesStore;
    noteId: string;
    onClose: () => void;
  }

  let { notes, noteId, onClose }: Props = $props();
  const l = t.notesApp.reminder;

  const pad = (value: number) => String(value).padStart(2, "0");

  function tomorrowMorning(): Date {
    const now = new Date();
    return new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1, 9, 0);
  }

  const start = tomorrowMorning();
  let date = $state(toLocalIsoDate(start));
  let time = $state("09:00");
  let current = $state.raw<NoteReminder | null>(null);
  let loaded = $state(false);
  let saving = $state(false);

  onMount(() => {
    void notes.reminderGet(noteId).then((found) => {
      current = found;
      if (found) {
        const at = new Date(found.notifyAt);
        date = toLocalIsoDate(at);
        time = `${pad(at.getHours())}:${pad(at.getMinutes())}`;
      }
      loaded = true;
    });
  });

  const chosen = $derived(date && time ? new Date(`${date}T${time}:00`) : null);
  const inFuture = $derived(
    chosen !== null &&
      !Number.isNaN(chosen.getTime()) &&
      chosen.getTime() > Date.now(),
  );

  async function save() {
    if (!chosen || !inFuture || saving) return;
    saving = true;
    const result = await notes.reminderSet(noteId, toInstant(chosen));
    saving = false;
    if (result.ok) onClose();
  }

  async function remove() {
    if (saving) return;
    saving = true;
    const result = await notes.reminderSet(noteId, null);
    saving = false;
    if (result.ok) onClose();
  }
</script>

<Modal label={l.title} closeLabel={l.close} {onClose}>
  <form
    class="form"
    onsubmit={(event) => {
      event.preventDefault();
      void save();
    }}
  >
    <h2>{l.title}</h2>
    {#if loaded}
      <p class="muted">
        {current ? l.current(formatNoteStamp(current.notifyAt)) : l.none}
      </p>
    {/if}
    <div class="row">
      <label class="block">
        <span class="label">{l.dateLabel}</span>
        <input class="text" type="date" data-autofocus bind:value={date} />
      </label>
      <label class="block">
        <span class="label">{l.timeLabel}</span>
        <input class="text" type="time" bind:value={time} />
      </label>
    </div>
    {#if !inFuture}
      <p class="error" role="alert">{l.past}</p>
    {/if}
    <div class="actions">
      {#if current}
        <button
          type="button"
          class="btn danger"
          disabled={saving}
          onclick={() => void remove()}
        >
          {l.remove}
        </button>
      {/if}
      <button type="submit" class="btn primary" disabled={!inFuture || saving}>
        {l.save}
      </button>
    </div>
  </form>
</Modal>

<style>
  .form {
    display: grid;
    gap: var(--space-4);
    min-width: min(360px, 90vw);
  }

  .muted {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 13px;
  }

  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: var(--space-3);
  }

  .block {
    display: grid;
    min-width: 0;
    gap: 6px;
  }

  .label {
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
  }

  .text {
    width: 100%;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>
