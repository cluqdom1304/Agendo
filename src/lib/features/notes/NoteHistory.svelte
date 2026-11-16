<script lang="ts">
  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { Note, NoteVersion, NoteVersionSummary } from "../../types";
  import { formatNoteStamp } from "./format";
  import { blocksOf, diffBlocks } from "./versionDiff";

  // Version history of a note (D-48): pick a version, see what changed against the note as it
  // is now, and restore it. Restoring keeps the replaced text as one more version.
  interface Props {
    notes: NotesStore;
    note: Note;
    onClose: () => void;
  }

  let { notes, note, onClose }: Props = $props();
  const l = t.notesApp.history;

  let versions = $state.raw<NoteVersionSummary[] | null>(null);
  let selected = $state.raw<NoteVersion | null>(null);
  let restoring = $state(false);

  $effect(() => {
    void notes.versionList(note.id).then((found) => (versions = found));
  });

  const lines = $derived(
    selected
      ? diffBlocks(blocksOf(selected.contentJson), blocksOf(note.contentJson))
      : [],
  );
  const changes = $derived(lines.filter((line) => line.kind !== "same").length);

  async function pick(id: string) {
    selected = await notes.versionGet(id);
  }

  async function restore() {
    if (!selected || restoring) return;
    restoring = true;
    const done = await notes.restoreVersion(selected.id);
    restoring = false;
    if (done) onClose();
  }
</script>

<Modal label={l.title} closeLabel={l.close} wide {onClose}>
  <div class="box">
    <h2>{l.title}</h2>

    {#if versions === null}
      <p class="muted">{l.loading}</p>
    {:else if versions.length === 0}
      <p class="muted">{l.empty}</p>
    {:else}
      <div class="layout">
        <ul class="list" aria-label={l.listLabel}>
          {#each versions as version (version.id)}
            <li>
              <button
                type="button"
                class="version"
                aria-pressed={selected?.id === version.id}
                onclick={() => void pick(version.id)}
              >
                <span class="stamp">{formatNoteStamp(version.createdAt)}</span>
                <span class="vtitle"
                  >{version.title || t.notesApp.untitled}</span
                >
              </button>
            </li>
          {/each}
        </ul>

        <div class="preview" aria-live="polite">
          {#if selected === null}
            <p class="muted">{l.pick}</p>
          {:else}
            <p class="muted">
              {changes === 0 ? l.identical : l.changes(changes)}
            </p>
            <div class="diff" role="list" aria-label={l.diffLabel}>
              {#each lines as line, index (index)}
                <p class="line {line.kind}" role="listitem">
                  <span class="mark" aria-hidden="true">
                    {line.kind === "added"
                      ? "+"
                      : line.kind === "removed"
                        ? "−"
                        : " "}
                  </span>
                  {#if line.kind !== "same"}
                    <span class="sr">
                      {line.kind === "added" ? l.added : l.removed}:
                    </span>
                  {/if}
                  {line.text}
                </p>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <div class="actions">
      <button type="button" class="btn" onclick={onClose}>{l.close}</button>
      <button
        type="button"
        class="btn primary"
        disabled={selected === null || restoring}
        onclick={() => void restore()}
      >
        {l.restore}
      </button>
    </div>
  </div>
</Modal>

<style>
  .box {
    display: grid;
    gap: var(--space-4);
    min-width: 0;
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .muted {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(150px, 1fr) 2fr;
    gap: var(--space-4);
    min-height: 0;
  }

  .list {
    display: grid;
    align-content: start;
    gap: 1px;
    max-height: 50vh;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .version {
    display: grid;
    gap: 2px;
    width: 100%;
    padding: 6px var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .version:hover {
    background: var(--bg-hover);
  }

  /* The chosen version, tinted like the open note in the list. */
  .version[aria-pressed="true"] {
    background: color-mix(in srgb, var(--accent) 16%, transparent);
  }

  .version:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .stamp {
    color: var(--text-secondary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .vtitle {
    overflow: hidden;
    font-size: 13px;
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview {
    display: grid;
    align-content: start;
    gap: var(--space-2);
    min-width: 0;
  }

  .diff {
    max-height: 45vh;
    padding: var(--space-2) 0;
    overflow-y: auto;
    border-radius: var(--radius-card);
    background: var(--bg-control);
  }

  .line {
    display: flex;
    gap: var(--space-2);
    margin: 0;
    min-height: 1.5em;
    padding: 1px var(--space-3);
    overflow-wrap: anywhere;
    font-size: 13px;
  }

  .line.added {
    background: var(--diff-added);
  }

  .line.removed {
    background: var(--diff-removed);
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  .mark {
    flex: none;
    width: 1ch;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    text-decoration: none;
  }

  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  @media (max-width: 560px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
