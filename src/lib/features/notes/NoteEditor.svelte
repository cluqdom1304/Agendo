<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import type { Tag } from "../../types";
  import type {
    ChecklistItem,
    NoteEditorHandle,
    TagQuery,
    TaskRefHost,
    ToggleName,
    ToolbarState,
  } from "./editor";
  import { matchTags } from "./tagQuery";
  import NoteToolbar from "./NoteToolbar.svelte";
  import { NoteSaver } from "./noteSaver";

  interface Props {
    /** Notes as they are when the editor opens; later changes come from the editor itself. */
    contentJson: string;
    /** Saves the document; rejects when the backend refuses it. */
    save: (contentJson: string) => Promise<void>;
    /** `inline` sits under a task row of the list; `field` is a form field (the default);
     * `page` is the body of a note, with the format bar always showing. */
    variant?: "inline" | "field" | "page";
    placeholder?: string;
    /** Takes the cursor when it opens: only for a note the user has just opened. */
    autofocus?: boolean;
    onAutofocus?: () => void;
    /** The focus left a note without text, towards `next`. */
    onEmptyBlur?: (next: Element | null) => void;
    /** Turns on the `#tag` shortcut: the tags to offer and what to do with the chosen one. */
    tagSuggest?: { tags: Tag[]; onPick: (tag: Tag) => void };
    /** Turns on task references (D-45): how to read and drive the tasks they point at. */
    taskHost?: TaskRefHost;
    /** Reads the reactive state the references depend on, so they are redrawn when it changes. */
    taskTrack?: () => void;
    /** The user asked for a task made from the note (`/tarea`, Ctrl+Mayús+T). */
    onTaskRequest?: (prefill: string) => void;
  }

  let {
    contentJson,
    save,
    variant = "field",
    placeholder = t.notes.placeholder,
    autofocus = false,
    onAutofocus,
    onEmptyBlur,
    tagSuggest,
    taskHost,
    taskTrack,
    onTaskRequest,
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
  let query = $state.raw<TagQuery | null>(null);
  let highlighted = $state(0);
  // Escape hides the list until the text after the `#` changes.
  let dismissedFor = $state<string | null>(null);

  const suggestions = $derived(
    tagSuggest && query && dismissedFor !== query.query
      ? matchTags(tagSuggest.tags, query.query)
      : [],
  );

  function pick(tag: Tag) {
    handle?.removeTagQuery();
    tagSuggest?.onPick(tag);
  }

  function onTagKey(event: KeyboardEvent): boolean {
    if (suggestions.length === 0 || event.ctrlKey || event.metaKey)
      return false;
    if (event.key === "ArrowDown") {
      highlighted = (highlighted + 1) % suggestions.length;
      return true;
    }
    if (event.key === "ArrowUp") {
      highlighted = (highlighted - 1 + suggestions.length) % suggestions.length;
      return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      const chosen = suggestions[highlighted];
      if (chosen) pick(chosen);
      return true;
    }
    if (event.key === "Escape") {
      dismissedFor = query?.query ?? null;
      return true;
    }
    return false;
  }

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
          placeholder,
          onChange: (json) => saver.schedule(json),
          onState: (state) => (toolbar = state),
          onLinkShortcut: () => (linkRequest += 1),
          onOpenLink: (href) => void openLink(href).catch(() => {}),
          ...(taskHost ? { taskHost } : {}),
          ...(onTaskRequest ? { onTaskRequest } : {}),
          ...(tagSuggest
            ? {
                onTagQuery: (next: TagQuery | null) => {
                  query = next;
                  highlighted = 0;
                },
                onTagKey,
              }
            : {}),
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

  // The references show live data: redraw them whenever the tasks they read change.
  $effect(() => {
    taskTrack?.();
    handle?.refreshTaskRefs();
  });

  /** Puts a reference to the task after the cursor. */
  export function insertTaskRef(taskId: string): void {
    handle?.insertTaskRef(taskId);
  }

  /** Text selected in the note, to start a task from it. */
  export function selectedText(): string {
    return handle?.selectedText() ?? "";
  }

  /** See `NoteEditorHandle.convertChecklists`. */
  export function convertChecklists(
    create: (items: ChecklistItem[]) => Promise<string[] | null>,
  ): Promise<number> {
    return handle?.convertChecklists(create) ?? Promise.resolve(0);
  }

  export function hasChecklists(): boolean {
    return (handle?.checklistItems().length ?? 0) > 0;
  }

  const toggle = (name: ToggleName) => handle?.toggle(name);
  const setLink = (href: string | null) => handle?.setLink(href) ?? false;
</script>

<div
  class="note"
  class:in-list={variant === "inline"}
  class:as-field={variant === "field"}
  class:as-page={variant === "page"}
  class:focused
  onfocusin={() => (focused = true)}
  onfocusout={onFocusOut}
>
  {#if failed}
    <p class="status" role="alert">{t.notes.loadFailed}</p>
  {:else}
    {#if focused || variant === "page"}
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
  {#if query && suggestions.length > 0}
    <ul
      class="suggest"
      role="listbox"
      aria-label={t.notesApp.noteTags}
      style:left="{query.left}px"
      style:top="{query.bottom + 6}px"
    >
      {#each suggestions as tag, index (tag.id)}
        <li role="presentation">
          <button
            type="button"
            role="option"
            class="suggestion"
            class:active={index === highlighted}
            aria-selected={index === highlighted}
            tabindex="-1"
            onmousedown={(event) => {
              event.preventDefault();
              pick(tag);
            }}
          >
            <span class="dot" style:background={tag.colorHex}></span>
            {tag.name}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .note {
    position: relative;
    border-radius: var(--radius-small);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  /* In the list the note reads as a quiet second line under the title, aligned with its
     text (row padding + checkbox + gap), and only looks like a field while it is edited. */
  .in-list {
    margin: -2px var(--space-3) var(--space-1) 40px;
    padding: 1px 6px 3px;
  }

  .as-field {
    min-height: 72px;
    padding: var(--space-2) 10px;
    background: var(--bg-control);
  }

  .note.focused:not(.as-page) {
    background: var(--bg-control);
  }

  /* The body of a note: no field chrome, reading-size text. */
  .as-page {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }

  .as-page .tools {
    position: sticky;
    top: 0;
    z-index: 1;
    padding-bottom: var(--space-2);
    background: var(--bg-window-solid);
  }

  .as-page .surface {
    flex: 1;
    cursor: text;
  }

  .as-field.focused {
    box-shadow:
      inset 0 0 0 1px var(--accent),
      0 0 0 3px color-mix(in srgb, var(--accent) 35%, transparent);
  }

  /* The format bar opens right under the task title, above the text it formats. */
  .in-list .tools {
    padding-top: 3px;
  }

  .suggest {
    position: fixed;
    z-index: 30;
    min-width: 160px;
    max-width: 260px;
    margin: 0;
    padding: 4px;
    border-radius: var(--radius-card);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    list-style: none;
    animation: pop-in var(--motion-fast) var(--ease-out);
  }

  .suggestion {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 5px var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .suggestion.active {
    background: var(--accent);
    color: var(--on-accent);
  }

  .dot {
    flex: none;
    width: 9px;
    height: 9px;
    border-radius: var(--radius-chip);
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

  .as-page .surface :global(.tiptap) {
    min-height: 50vh;
    font-size: 15px;
    line-height: 1.6;
  }

  .focused .surface :global(.tiptap),
  .as-field .surface :global(.tiptap),
  .as-page .surface :global(.tiptap) {
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

  /* A reference to a task: a row that reads like the task's own row in the agenda. */
  .surface :global(.task-ref) {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 3px 0;
    padding: 5px var(--space-2);
    border-radius: var(--radius-control);
    background: var(--bg-control);
    font-size: 14px;
    line-height: 1.35;
    user-select: none;
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .surface :global(.task-ref:hover) {
    background: var(--bg-control-hover);
  }

  .surface :global(.task-ref.ProseMirror-selectednode) {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }

  .surface :global(.task-ref button) {
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .surface :global(.task-ref button:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .surface :global(.tr-check),
  .surface :global(.tr-mark) {
    position: relative;
    flex: none;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
    transition:
      background-color var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      transform var(--motion-base) var(--ease-bounce);
  }

  .surface :global(.tr-check:active) {
    transform: scale(0.82);
  }

  .surface :global(.task-ref.is-done .tr-check) {
    border-color: var(--accent);
    background: var(--accent);
  }

  .surface :global(.task-ref.is-done .tr-check::after) {
    content: "";
    position: absolute;
    top: 1.5px;
    left: 4.5px;
    width: 4px;
    height: 7px;
    border: solid var(--on-accent);
    border-width: 0 1.5px 1.5px 0;
    transform: rotate(45deg);
  }

  .surface :global(.tr-title) {
    flex: 1;
    min-width: 0;
    padding: 0;
    overflow: hidden;
    color: var(--text-primary);
    font-weight: 500;
    text-align: left;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .surface :global(.task-ref.is-done .tr-title) {
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  .surface :global(.tr-due) {
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .surface :global(.tr-tag) {
    flex: none;
    padding: 0 7px;
    border-radius: var(--radius-chip);
    background: color-mix(in srgb, var(--tag) 20%, transparent);
    color: color-mix(in srgb, var(--tag) 55%, var(--text-primary));
    font-size: 11px;
    font-weight: 600;
    line-height: 18px;
  }

  /* A task that was deleted: struck through, with a way back. */
  .surface :global(.task-ref.is-gone) {
    opacity: 0.75;
  }

  .surface :global(.tr-title.gone) {
    color: var(--text-secondary);
    font-style: italic;
    text-decoration: line-through;
  }

  .surface :global(.tr-mark.gone) {
    border-style: dashed;
  }

  .surface :global(.tr-restore) {
    flex: none;
    padding: 1px 10px;
    border-radius: var(--radius-chip);
    background: var(--accent);
    color: var(--on-accent);
    font-size: 12px;
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
