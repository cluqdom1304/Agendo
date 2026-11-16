<script lang="ts">
  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";

  // Templates for new notes (F13-02): use, rename, rewrite with the open note, delete.
  interface Props {
    notes: NotesStore;
    onClose: () => void;
  }

  let { notes, onClose }: Props = $props();
  const l = t.notesApp.templates;

  let newName = $state("");
  let renaming = $state<{ id: string; name: string } | null>(null);

  const canSave = $derived(notes.open !== null && newName.trim() !== "");

  async function saveNew() {
    if (!canSave) return;
    if (await notes.saveOpenAsTemplate(newName)) newName = "";
  }

  async function commitRename() {
    if (!renaming || renaming.name.trim() === "") return;
    if (await notes.renameTemplate(renaming.id, renaming.name)) renaming = null;
  }
</script>

<Modal label={l.title} closeLabel={l.close} {onClose}>
  <div class="box">
    <h2>{l.title}</h2>

    {#if notes.templates.length === 0}
      <p class="muted">{l.empty}</p>
    {:else}
      <ul class="list">
        {#each notes.templates as template (template.id)}
          <li class="item">
            {#if renaming?.id === template.id}
              <form
                class="rename"
                onsubmit={(event) => {
                  event.preventDefault();
                  void commitRename();
                }}
              >
                <input
                  class="text"
                  aria-label={l.renameLabel(template.name)}
                  data-autofocus
                  maxlength="100"
                  bind:value={renaming.name}
                />
                <button type="submit" class="btn small primary">
                  {l.save}
                </button>
                <button
                  type="button"
                  class="btn small"
                  onclick={() => (renaming = null)}
                >
                  {l.cancel}
                </button>
              </form>
            {:else}
              <span class="name">{template.name}</span>
              <span class="buttons">
                <button
                  type="button"
                  class="btn small primary"
                  onclick={() => void notes.createFromTemplate(template.id)}
                >
                  {l.use}
                </button>
                <button
                  type="button"
                  class="btn small quiet"
                  onclick={() =>
                    (renaming = { id: template.id, name: template.name })}
                >
                  {l.rename}
                </button>
                <button
                  type="button"
                  class="btn small quiet"
                  disabled={notes.open === null}
                  onclick={() =>
                    void notes.saveOpenAsTemplate(template.name, template.id)}
                >
                  {l.overwrite}
                </button>
                <button
                  type="button"
                  class="btn small quiet danger"
                  onclick={() => void notes.deleteTemplate(template.id)}
                >
                  {l.delete}
                </button>
              </span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <form
      class="save"
      onsubmit={(event) => {
        event.preventDefault();
        void saveNew();
      }}
    >
      <h3>{l.saveHeading}</h3>
      {#if notes.open === null}
        <p class="muted">{l.noOpenNote}</p>
      {/if}
      <label class="block">
        <span class="label">{l.nameLabel}</span>
        <input
          class="text"
          maxlength="100"
          placeholder={l.namePlaceholder}
          disabled={notes.open === null}
          bind:value={newName}
        />
      </label>
      <div class="actions">
        <button type="submit" class="btn primary" disabled={!canSave}>
          {l.saveNew}
        </button>
      </div>
    </form>
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

  h3 {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .muted {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .list {
    margin: 0;
    padding: 0;
    overflow: hidden;
    border-radius: var(--radius-card);
    background: var(--bg-control);
    list-style: none;
  }

  .item {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-1) var(--space-2);
    min-height: 40px;
    padding: 6px var(--space-2) 6px var(--space-3);
  }

  .item + .item {
    box-shadow: inset 0 1px 0 var(--separator);
  }

  .name {
    flex: 1;
    min-width: 96px;
    overflow-wrap: anywhere;
    font-size: 13px;
    font-weight: 500;
  }

  .buttons,
  .rename,
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
  }

  .rename,
  .actions {
    gap: var(--space-2);
  }

  .btn.quiet.danger {
    color: var(--danger);
  }

  .rename {
    flex: 1;
  }

  .rename .text {
    flex: 1;
    min-width: 0;
  }

  .actions {
    justify-content: flex-end;
  }

  .save {
    display: grid;
    gap: var(--space-3);
    padding-top: var(--space-4);
    border-top: 1px solid var(--separator);
  }

  .block {
    display: grid;
    gap: 6px;
  }

  .label {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
  }

  .text {
    width: 100%;
  }
</style>
