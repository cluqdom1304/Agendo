<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import type { NoteEditorHandle, ToggleName, ToolbarState } from "./editor";
  import NoteToolbar from "./NoteToolbar.svelte";
  import { NoteSaver } from "./noteSaver";

  interface Props {
    /** Notes as they are when the editor opens; later changes come from the editor itself. */
    contentJson: string;
    /** Saves the document; rejects when the backend refuses it. */
    save: (contentJson: string) => Promise<void>;
  }

  let { contentJson, save }: Props = $props();

  const idleState: ToolbarState = {
    bold: false,
    italic: false,
    code: false,
    bulletList: false,
    orderedList: false,
    link: false,
    linkHref: "",
    canUndo: false,
    canRedo: false,
  };

  let host = $state<HTMLDivElement>();
  let handle = $state.raw<NoteEditorHandle | null>(null);
  let toolbar = $state.raw<ToolbarState>(idleState);
  let linkRequest = $state(0);
  let failed = $state(false);

  // The saver outlives the editor module's loading: edits only exist after it is ready.
  const saver = new NoteSaver((json) => save(json));

  async function openLink(href: string) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(href);
  }

  onMount(() => {
    let disposed = false;
    let created: NoteEditorHandle | null = null;

    // TipTap is a separate chunk, fetched when the first note opens (D-10).
    import("./editor")
      .then(({ createNoteEditor }) => {
        if (disposed || !host) return;
        created = createNoteEditor({
          element: host,
          contentJson,
          label: t.notes.editorLabel,
          onChange: (json) => saver.schedule(json),
          onState: (state) => (toolbar = state),
          onLinkShortcut: () => (linkRequest += 1),
          onOpenLink: (href) => void openLink(href).catch(() => {}),
        });
        handle = created;
        toolbar = created.state();
        created.focus();
      })
      .catch(() => (failed = true));

    return () => {
      disposed = true;
      void saver.flush();
      created?.destroy();
    };
  });

  const toggle = (name: ToggleName) => handle?.toggle(name);
  const setLink = (href: string | null) => handle?.setLink(href) ?? false;
</script>

<div class="note">
  {#if failed}
    <p class="status" role="alert">{t.notes.loadFailed}</p>
  {:else}
    <NoteToolbar
      formats={toolbar}
      {linkRequest}
      onToggle={toggle}
      onSetLink={setLink}
      onUndo={() => handle?.undo()}
      onRedo={() => handle?.redo()}
    />
    {#if !handle}
      <p class="status">{t.notes.loading}</p>
    {/if}
  {/if}
  <div class="surface" bind:this={host}></div>
</div>

<style>
  .note {
    /* Sub-points sit under the task title, one 24 px step in from the row (F3-04). */
    margin: 0 var(--space-3) var(--space-2) calc(var(--space-3) + 24px);
    padding-left: var(--space-3);
    border-left: 2px solid var(--border-subtle);
  }

  .status {
    margin: 0;
    padding: var(--space-1) 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .surface :global(.tiptap) {
    min-height: 28px;
    padding: 2px 0;
    outline: none;
    color: var(--text-primary);
    font-size: 14px;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }

  .surface :global(.tiptap p) {
    margin: 0;
  }

  .surface :global(.tiptap ul),
  .surface :global(.tiptap ol) {
    margin: 0;
    padding-left: 24px;
  }

  .surface :global(.tiptap ul) {
    list-style: disc;
  }

  .surface :global(.tiptap ol) {
    list-style: decimal;
  }

  .surface :global(.tiptap li > p) {
    margin: 0;
  }

  .surface :global(.tiptap code) {
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--bg-hover);
    font-family: var(--font-mono);
    font-size: 0.92em;
  }

  .surface :global(.tiptap a) {
    color: var(--accent);
    text-decoration: underline;
    cursor: text;
  }
</style>
