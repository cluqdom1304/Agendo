<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import type { Tag } from "../../types";
  import type {
    ChecklistItem,
    FindState,
    ImageHost,
    NoteEditorHandle,
    NoteLinkHost,
    TagQuery,
    TaskRefHost,
    ToggleName,
    ToolbarState,
  } from "./editor";
  import { matchTags } from "./tagQuery";
  import NoteToolbar from "./NoteToolbar.svelte";
  import { NoteSaver } from "./noteSaver";
  import { IDLE_TOOLBAR } from "./toolbarState";

  interface Props {
    /** Notes as they are when the editor opens; later changes come from the editor itself. */
    contentJson: string;
    /** Saves the document; rejects when the backend refuses it. */
    save: (contentJson: string) => Promise<void>;
    /** `inline` sits under a task row of the list; `field` is a form field (the default);
     * `page` is the body of a note: it draws no format bar of its own, whoever holds it shows
     * one in its header and drives it through the exported functions. */
    variant?: "inline" | "field" | "page";
    /** Shows the note without letting it be edited. */
    readonly?: boolean;
    /** Accessible name of the text box. */
    label?: string;
    placeholder?: string;
    /** Takes the cursor when it opens: only for a note the user has just opened. */
    autofocus?: boolean;
    onAutofocus?: () => void;
    /** The focus left a note without text, towards `next`. */
    onEmptyBlur?: (next: Element | null) => void;
    /** Turns on the `#tag` shortcut: the tags to offer and what to do with the chosen one. */
    tagSuggest?: { tags: Tag[]; onPick: (tag: Tag) => void };
    /** Turns on `[[note]]` links (D-46): how to draw and open them, and what to offer while one
     * is typed. `create` makes a note on the spot and resolves with its id. */
    links?: {
      host: NoteLinkHost;
      search: (text: string) => Promise<{ id: string; title: string }[]>;
      create: (title: string) => Promise<string | null>;
      /** Reads the reactive state the titles depend on, so links are redrawn when it changes. */
      track?: () => void;
    };
    /** Turns on task references (D-45): how to read and drive the tasks they point at. */
    taskHost?: TaskRefHost;
    /** Reads the reactive state the references depend on, so they are redrawn when it changes. */
    taskTrack?: () => void;
    /** Headings, quotes, code blocks, tables, highlight and find: the body of a note. */
    advanced?: boolean;
    /** Turns on images (D-47): where they are stored and shown. Only for notes. */
    images?: ImageHost;
    /** The user asked for a task made from the note (`/tarea`, Ctrl+Mayús+T). */
    onTaskRequest?: (prefill: string) => void;
  }

  let {
    contentJson,
    save,
    variant = "field",
    readonly = false,
    label = t.notes.editorLabel,
    placeholder = t.notes.placeholder,
    autofocus = false,
    onAutofocus,
    onEmptyBlur,
    tagSuggest,
    links,
    taskHost,
    advanced = false,
    images,
    taskTrack,
    onTaskRequest,
  }: Props = $props();

  let host = $state<HTMLDivElement>();
  let handle = $state.raw<NoteEditorHandle | null>(null);
  let toolbar = $state.raw<ToolbarState>(IDLE_TOOLBAR);
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

  // `[[note]]`: the notes that match what is typed, plus "create it" when none has that title.
  type LinkChoice = { id: string; title: string } | { create: string };
  let linkQuery = $state.raw<TagQuery | null>(null);
  let linkChoices = $state.raw<LinkChoice[]>([]);
  let linkHighlighted = $state(0);
  let linkDismissedFor = $state<string | null>(null);
  // Answers can arrive out of order; only the latest request may update the list.
  let linkRequestToken = 0;

  const linkVisible = $derived(
    linkQuery !== null &&
      linkDismissedFor !== linkQuery.query &&
      linkChoices.length > 0,
  );

  async function refreshLinkChoices(text: string | null) {
    const token = ++linkRequestToken;
    if (text === null || !links) {
      linkChoices = [];
      return;
    }
    let found: { id: string; title: string }[];
    try {
      found = await links.search(text);
    } catch {
      found = [];
    }
    if (token !== linkRequestToken) return;
    const wanted = text.trim();
    const exact = found.some(
      (item) => item.title.trim().toLowerCase() === wanted.toLowerCase(),
    );
    linkChoices = [...found, ...(wanted && !exact ? [{ create: wanted }] : [])];
  }

  async function pickLink(choice: LinkChoice) {
    if ("create" in choice) {
      const id = await links?.create(choice.create);
      if (id) handle?.insertNoteLink(id);
    } else {
      handle?.insertNoteLink(choice.id);
    }
  }

  function onLinkKey(event: KeyboardEvent): boolean {
    if (!linkVisible || event.ctrlKey || event.metaKey) return false;
    if (event.key === "ArrowDown") {
      linkHighlighted = (linkHighlighted + 1) % linkChoices.length;
      return true;
    }
    if (event.key === "ArrowUp") {
      linkHighlighted =
        (linkHighlighted - 1 + linkChoices.length) % linkChoices.length;
      return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      const chosen = linkChoices[linkHighlighted];
      if (chosen) void pickLink(chosen);
      return true;
    }
    if (event.key === "Escape") {
      linkDismissedFor = linkQuery?.query ?? null;
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
          label,
          placeholder,
          editable: !readonly,
          onChange: (json) => saver.schedule(json),
          onState: (state) => (toolbar = state),
          onLinkShortcut: () => (linkRequest += 1),
          onOpenLink: (href) => void openLink(href).catch(() => {}),
          ...(links
            ? {
                linkHost: links.host,
                onLinkQuery: (next: TagQuery | null) => {
                  linkQuery = next;
                  linkHighlighted = 0;
                  void refreshLinkChoices(next ? next.query : null);
                },
                onLinkKey,
              }
            : {}),
          ...(taskHost ? { taskHost } : {}),
          ...(images ? { images } : {}),
          ...(advanced ? { advanced } : {}),
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
  $effect(() => {
    links?.track?.();
    handle?.refreshNoteLinks();
  });

  /** Puts a reference to the task after the cursor. */
  export function insertTaskRef(taskId: string): void {
    handle?.insertTaskRef(taskId);
  }

  export function insertTable(): void {
    handle?.insertTable();
  }

  export function insertRule(): void {
    handle?.insertRule();
  }

  /** Marks `query` in the note and moves to the first match. */
  export function find(query: string): FindState {
    return handle?.find(query) ?? { count: 0, current: -1 };
  }

  export function findStep(direction: 1 | -1): FindState {
    return handle?.findStep(direction) ?? { count: 0, current: -1 };
  }

  export function clearFind(): void {
    handle?.clearFind();
  }

  /** Stores the image files and puts them in the note. */
  export function insertImages(files: File[]): Promise<void> {
    return handle?.insertImages(files) ?? Promise.resolve();
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

  /** What a format bar outside the editor has to draw. Reactive: read it where it is shown. */
  export function formats(): ToolbarState {
    return toolbar;
  }

  /** Grows each time the note asks for the link field (Ctrl+L). */
  export function linkRequests(): number {
    return linkRequest;
  }

  export function toggle(name: ToggleName): void {
    handle?.toggle(name);
  }

  /** Sets the link on the selection, or removes it with `null`. False if it is refused. */
  export function setLink(href: string | null): boolean {
    return handle?.setLink(href) ?? false;
  }

  export function undo(): void {
    handle?.undo();
  }

  export function redo(): void {
    handle?.redo();
  }

  export function focus(position: "start" | "end" = "end"): void {
    handle?.focus(position);
  }
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
    {#if focused && variant !== "page"}
      <div class="tools">
        <NoteToolbar
          formats={toolbar}
          {linkRequest}
          onToggle={toggle}
          onSetLink={setLink}
          onUndo={undo}
          onRedo={redo}
        />
      </div>
    {/if}
    {#if !handle}
      <p class="status loading">{t.notes.loading}</p>
    {/if}
  {/if}
  <div class="surface" bind:this={host}></div>
  {#if linkQuery && linkVisible}
    <ul
      class="suggest"
      role="listbox"
      aria-label={t.notesApp.linkSuggestions}
      style:left="{linkQuery.left}px"
      style:top="{linkQuery.bottom + 6}px"
    >
      {#each linkChoices as choice, index ("create" in choice ? `new:${choice.create}` : choice.id)}
        <li role="presentation">
          <button
            type="button"
            role="option"
            class="suggestion"
            class:active={index === linkHighlighted}
            aria-selected={index === linkHighlighted}
            tabindex="-1"
            onmousedown={(event) => {
              event.preventDefault();
              void pickLink(choice);
            }}
          >
            {#if "create" in choice}
              <span class="plus" aria-hidden="true">+</span>
              {t.notesApp.linkCreate(choice.create)}
            {:else}
              <svg
                viewBox="0 0 16 16"
                width="12"
                height="12"
                aria-hidden="true"
              >
                <path
                  d="M4 2.5h5.5L12.5 5.5v8H4zM9.5 2.5v3h3"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.4"
                  stroke-linejoin="round"
                />
              </svg>
              <span class="link-title"
                >{choice.title || t.notesApp.untitled}</span
              >
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
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

  /* The body of a note: no field chrome, reading-size text, and as tall as the room it is
     given so a click anywhere under the text lands in it. */
  .as-page,
  .as-page .surface {
    display: flex;
    flex: 1;
    flex-direction: column;
  }

  .as-page .surface {
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

  .plus {
    flex: none;
    width: 12px;
    text-align: center;
    font-weight: 600;
  }

  .link-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* A link to another note: accent text with a soft underline, like a link in Notes. */
  .surface :global(.note-link) {
    color: var(--accent);
    border-radius: 3px;
    cursor: pointer;
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, var(--accent) 40%, transparent);
    text-underline-offset: 2px;
    transition: background-color var(--motion-instant) linear;
  }

  .surface :global(.note-link:hover) {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .surface :global(.note-link:focus-visible),
  .surface :global(.note-link.ProseMirror-selectednode) {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .surface :global(.note-link.is-gone) {
    color: var(--text-secondary);
    text-decoration: line-through;
    cursor: default;
  }

  .surface :global(.note-image) {
    position: relative;
    display: block;
    width: fit-content;
    max-width: 100%;
    margin: var(--space-3) 0;
  }

  .surface :global(.note-image img) {
    display: block;
    max-width: 100%;
    height: auto;
    border-radius: var(--radius-control);
    cursor: zoom-in;
  }

  .surface :global(.note-image img.is-loading) {
    min-width: 120px;
    min-height: 80px;
    background: var(--bg-control-hover);
  }

  .surface :global(.note-image.ProseMirror-selectednode img) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .surface :global(.note-image .image-gone) {
    display: block;
    padding: var(--space-4);
    border: 1px dashed var(--separator);
    border-radius: var(--radius-control);
    color: var(--text-secondary);
    font-size: 12px;
  }

  /* The corner to drag: only on hover or while the image is chosen. */
  .surface :global(.note-image .image-grip) {
    position: absolute;
    right: -5px;
    bottom: -5px;
    width: 12px;
    height: 12px;
    border: 2px solid var(--bg-popover);
    border-radius: 50%;
    background: var(--accent);
    cursor: nwse-resize;
    opacity: 0;
    transition: opacity var(--motion-fast) linear;
  }

  .surface :global(.note-image:hover .image-grip),
  .surface :global(.note-image.ProseMirror-selectednode .image-grip),
  .surface :global(.note-image .image-grip:focus-visible) {
    opacity: 1;
  }

  .surface :global(.note-image.is-gone .image-grip) {
    display: none;
  }

  .surface :global(.note-link.is-loading) {
    color: var(--text-secondary);
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

  /* The blocks of a note: titles, quotes, code, rules, highlight and tables (F12-03). */
  .surface :global(.tiptap h1),
  .surface :global(.tiptap h2),
  .surface :global(.tiptap h3) {
    margin: 1.1em 0 0.35em;
    color: var(--text-primary);
    font-family: var(--font-display);
    letter-spacing: -0.01em;
    line-height: 1.25;
  }

  .surface :global(.tiptap h1) {
    font-size: 1.7em;
    font-weight: 700;
  }

  .surface :global(.tiptap h2) {
    font-size: 1.35em;
    font-weight: 650;
  }

  .surface :global(.tiptap h3) {
    font-size: 1.12em;
    font-weight: 600;
  }

  .surface :global(.tiptap blockquote) {
    margin: var(--space-3) 0;
    padding: 2px 0 2px var(--space-4);
    border-left: 3px solid var(--separator);
    color: var(--text-secondary);
  }

  .surface :global(.tiptap hr) {
    margin: var(--space-4) 0;
    border: 0;
    border-top: 1px solid var(--separator);
  }

  .surface :global(.tiptap hr.ProseMirror-selectednode) {
    border-top-color: var(--accent);
  }

  .surface :global(.tiptap mark) {
    padding: 0 2px;
    border-radius: 3px;
    background: color-mix(in srgb, #ffd60a 45%, transparent);
    color: inherit;
  }

  .surface :global(.tiptap pre) {
    margin: var(--space-3) 0;
    padding: var(--space-3) var(--space-4);
    overflow-x: auto;
    border-radius: var(--radius-control);
    background: var(--bg-control);
    color: var(--text-primary);
    font-family: var(
      --font-mono,
      ui-monospace,
      "Cascadia Code",
      Consolas,
      monospace
    );
    font-size: 0.88em;
    line-height: 1.5;
    tab-size: 2;
  }

  .surface :global(.tiptap pre code) {
    padding: 0;
    background: transparent;
    font: inherit;
    white-space: pre;
  }

  .surface :global(.tok-comment) {
    color: var(--text-secondary);
    font-style: italic;
  }

  .surface :global(.tok-string) {
    color: #1f8f4a;
  }

  .surface :global(.tok-number) {
    color: #c26a00;
  }

  .surface :global(.tok-keyword) {
    color: #a540c9;
    font-weight: 600;
  }

  .surface :global(.tok-type) {
    color: #0a6ebd;
  }

  @media (prefers-color-scheme: dark) {
    .surface :global(.tok-string) {
      color: #6bd68a;
    }

    .surface :global(.tok-number) {
      color: #ffb15c;
    }

    .surface :global(.tok-keyword) {
      color: #d49bff;
    }

    .surface :global(.tok-type) {
      color: #6cb6ff;
    }
  }

  .surface :global(.tiptap table) {
    width: 100%;
    margin: var(--space-3) 0;
    border-collapse: collapse;
    table-layout: fixed;
    overflow: hidden;
  }

  .surface :global(.tiptap th),
  .surface :global(.tiptap td) {
    position: relative;
    min-width: 40px;
    padding: 5px 8px;
    border: 1px solid var(--separator);
    vertical-align: top;
    text-align: left;
  }

  .surface :global(.tiptap th) {
    background: var(--bg-control);
    color: var(--text-primary);
    font-weight: 600;
  }

  .surface :global(.tiptap .selectedCell::after) {
    content: "";
    position: absolute;
    inset: 0;
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    pointer-events: none;
  }

  .surface :global(.find-match) {
    border-radius: 2px;
    background: color-mix(in srgb, #ffd60a 40%, transparent);
  }

  .surface :global(.find-current) {
    background: #ffd60a;
    color: #1d1d1f;
  }

  .as-page .surface :global(.tiptap) {
    flex: 1;
    min-height: 160px;
    padding-bottom: var(--space-6);
    font-size: 15px;
    line-height: 1.6;
  }

  .as-page .surface :global(.tiptap[contenteditable="false"]) {
    cursor: default;
  }

  /* Lists breathe a little at reading size. */
  .as-page .surface :global(.tiptap > ul),
  .as-page .surface :global(.tiptap > ol) {
    margin: 2px 0;
  }

  .as-page .surface :global(.tiptap ul[data-type="taskList"] li) {
    padding: 2px 0;
  }

  .as-page .surface :global(.tiptap ul[data-type="taskList"] li > label) {
    padding-top: 4.5px;
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
    margin: 4px 0;
    padding: 6px 10px;
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

  .surface :global(.task-ref .tr-restore) {
    flex: none;
    padding: 2px 8px;
    border-radius: var(--radius-small);
    color: var(--accent);
    font-size: 12px;
    font-weight: 500;
  }

  .surface :global(.task-ref .tr-restore:hover) {
    background: var(--bg-hover);
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
