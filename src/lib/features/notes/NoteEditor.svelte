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
    /** `inline` sits under a task row of the list; `field` is a form field (the default). */
    variant?: "inline" | "field";
    /** Takes the cursor when it opens: only for a note the user has just opened. */
    autofocus?: boolean;
    onAutofocus?: () => void;
    /** The focus left a note without text, towards `next`. */
    onEmptyBlur?: (next: Element | null) => void;
  }

  let {
    contentJson,
    save,
    variant = "field",
    autofocus = false,
    onAutofocus,
    onEmptyBlur,
  }: Props = $props();

  const idleState: ToolbarState = {
    bold: false,
    italic: false,
    code: false,
    bulletList: false,
    orderedList: false,
    taskList: false,
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
  // The controls only show while the focus is inside the note (editor or toolbar).
  let focused = $state(false);

  function onFocusOut(event: FocusEvent) {
    const next = event.relatedTarget;
    if (
      next instanceof Node &&
      (event.currentTarget as HTMLElement).contains(next)
    )
      return;
    focused = false;
    if (handle?.isEmpty()) onEmptyBlur?.(next instanceof Element ? next : null);
  }

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
          placeholder: t.notes.placeholder,
          onChange: (json) => saver.schedule(json),
          onState: (state) => (toolbar = state),
          onLinkShortcut: () => (linkRequest += 1),
          onOpenLink: (href) => void openLink(href).catch(() => {}),
        });
        handle = created;
        toolbar = created.state();
        // Notes shown by default must not steal the cursor when the app opens.
        if (autofocus) {
          focused = true;
          created.focus();
          onAutofocus?.();
        }
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

<div
  class="note {variant}"
  class:focused
  onfocusin={() => (focused = true)}
  onfocusout={onFocusOut}
>
  {#if failed}
    <p class="status" role="alert">{t.notes.loadFailed}</p>
  {:else}
    {#if focused}
      <div class="tools">
        <NoteToolbar
          formats={toolbar}
          {linkRequest}
          onToggle={toggle}
          onSetLink={setLink}
          onUndo={() => handle?.undo()}
          onRedo={() => handle?.redo()}
        />
      </div>
    {/if}
    {#if !handle}
      <p class="status loading">{t.notes.loading}</p>
    {/if}
  {/if}
  <div class="surface" bind:this={host}></div>
</div>

<style>
  .note {
    position: relative;
    border-radius: var(--radius-small);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  /* In the list the note reads as a quiet second line under the title, aligned with its
     text (row padding + checkbox + gap), and only looks like a field while it is edited. */
  .inline {
    margin: -2px var(--space-3) var(--space-1) 40px;
    padding: 1px 6px 3px;
  }

  .field {
    min-height: 72px;
    padding: var(--space-2) 10px;
    background: var(--bg-control);
  }

  .note.focused {
    background: var(--bg-control);
  }

  .field.focused {
    box-shadow:
      inset 0 0 0 1px var(--accent),
      0 0 0 3px color-mix(in srgb, var(--accent) 35%, transparent);
  }

  /* The format bar floats under the note: showing it never pushes the list around. */
  .inline .tools {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 5;
    padding: 3px;
    border-radius: var(--radius-card);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    transform-origin: top right;
    animation: pop-in var(--motion-fast) var(--ease-out);
    --note-toolbar-bg: transparent;
    --note-toolbar-margin: 0;
  }

  .status {
    margin: 0;
    padding: 2px 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  /* The editor module usually arrives at once: the notice only shows if it does not. */
  .status.loading {
    position: absolute;
    animation: fade-in var(--motion-fast) linear 500ms both;
  }

  .surface :global(.tiptap) {
    min-height: 20px;
    padding: 2px 0;
    outline: none;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
    overflow-wrap: anywhere;
    transition: color var(--motion-fast) var(--ease-out);
  }

  .focused .surface :global(.tiptap),
  .field .surface :global(.tiptap) {
    color: var(--text-primary);
  }

  .surface :global(.tiptap p) {
    margin: 0;
  }

  .surface :global(.tiptap p.is-editor-empty:first-child::before) {
    content: attr(data-placeholder);
    float: left;
    height: 0;
    color: var(--text-secondary);
    pointer-events: none;
  }

  .surface :global(.tiptap ul),
  .surface :global(.tiptap ol) {
    margin: 0;
    padding-left: 22px;
  }

  .surface :global(.tiptap ul) {
    list-style: disc;
  }

  .surface :global(.tiptap ol) {
    list-style: decimal;
  }

  .surface :global(.tiptap ul[data-type="taskList"]) {
    padding-left: 0;
    list-style: none;
  }

  .surface :global(.tiptap ul[data-type="taskList"] li) {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 1px 0;
  }

  .surface :global(.tiptap ul[data-type="taskList"] li > label) {
    flex: none;
    padding-top: 2px;
    user-select: none;
  }

  .surface :global(.tiptap ul[data-type="taskList"] li > div) {
    flex: 1;
    min-width: 0;
  }

  /* Round, like the task checkbox above it; the tick is two borders of a rotated box. */
  .surface :global(.tiptap ul[data-type="taskList"] input[type="checkbox"]) {
    position: relative;
    display: block;
    appearance: none;
    width: 15px;
    height: 15px;
    margin: 0;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
    background: transparent;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      transform var(--motion-base) var(--ease-bounce);
  }

  .surface
    :global(.tiptap ul[data-type="taskList"] input[type="checkbox"]:active) {
    transform: scale(0.82);
  }

  .surface
    :global(.tiptap ul[data-type="taskList"] input[type="checkbox"]:checked) {
    border-color: var(--accent);
    background: var(--accent);
  }

  .surface
    :global(
      .tiptap ul[data-type="taskList"] input[type="checkbox"]:checked::after
    ) {
    content: "";
    position: absolute;
    top: 1.5px;
    left: 4px;
    width: 4px;
    height: 7px;
    border: solid var(--on-accent);
    border-width: 0 1.5px 1.5px 0;
    transform: rotate(45deg);
  }

  .surface :global(.tiptap li[data-checked="true"] > div) {
    color: var(--text-secondary);
    text-decoration: line-through;
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
