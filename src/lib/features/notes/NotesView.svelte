<script lang="ts">
  import { t } from "../../i18n";
  import { findShortcut } from "../../keymap";
  import EmptyState from "../../shell/EmptyState.svelte";
  import Toast from "../../shell/Toast.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import NoteList from "./NoteList.svelte";
  import NotePane from "./NotePane.svelte";

  interface Props {
    notes: NotesStore;
    tags: TagsStore;
  }

  let { notes, tags }: Props = $props();

  const lastTrashed = $derived(notes.undoStack.at(-1));

  function create() {
    void notes.createNote();
  }

  // Ctrl+N is the section's own "new": the agenda's handler is not mounted here.
  function onKeydown(event: KeyboardEvent) {
    if (findShortcut(event, "other") !== "newTask") return;
    event.preventDefault();
    create();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="notes">
  {#if notes.status === "error"}
    <p class="status" role="alert">{t.notesApp.loadFailed}</p>
  {:else}
    <NoteList {notes} {tags} onCreate={create} />
    <div class="pane">
      {#if notes.open}
        {#key notes.open.id}
          <NotePane {notes} note={notes.open} />
        {/key}
      {:else}
        <EmptyState
          title={t.notesApp.noSelectionTitle}
          description={t.notesApp.noSelectionDescription}
        />
      {/if}
    </div>
  {/if}
</div>

{#if lastTrashed}
  <Toast
    message={t.notesApp.deleted}
    actionLabel={t.notesApp.undo}
    closeLabel={t.notesApp.dismiss}
    onAction={() => void notes.undoTrash()}
    onClose={() => (notes.undoStack = [])}
  />
{:else if notes.notice}
  <Toast
    kind={notes.notice.kind}
    message={notes.notice.message}
    closeLabel={t.notesApp.dismiss}
    onClose={() => notes.dismissNotice()}
  />
{/if}

<style>
  .notes {
    display: flex;
    height: 100%;
  }

  .pane {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .status {
    padding: var(--space-6);
    color: var(--text-secondary);
  }
</style>
