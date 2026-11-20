<script lang="ts">
  import { t } from "../../i18n";
  import type { ToggleName, ToolbarState } from "./editor";

  type Icon =
    ToggleName | "link" | "undo" | "redo" | "rule" | "table" | "image";

  interface Props {
    formats: ToolbarState;
    /** Changes every time the editor asks for the link field (Ctrl+L). */
    linkRequest: number;
    /** The link field floats under the bar instead of pushing the text down: for a bar that
     * lives in a header. The holder of the bar must be positioned. */
    floating?: boolean;
    /** Adds the buttons for headings, quotes, code, highlight, tables and images (notes). */
    advanced?: boolean;
    onToggle: (name: ToggleName) => void;
    onInsertRule?: () => void;
    onInsertTable?: () => void;
    /** Opens the file picker for an image. */
    onInsertImage?: () => void;
    /** Returns false when the address is refused. `null` removes the link. */
    onSetLink: (href: string | null) => boolean;
    onUndo: () => void;
    onRedo: () => void;
  }

  let {
    formats,
    linkRequest,
    floating = false,
    advanced = false,
    onToggle,
    onInsertRule,
    onInsertTable,
    onInsertImage,
    onSetLink,
    onUndo,
    onRedo,
  }: Props = $props();

  let linkOpen = $state(false);
  let linkText = $state("");
  let linkError = $state(false);
  let linkInput = $state<HTMLInputElement>();
  // The requests made before the bar existed are not for it.
  // svelte-ignore state_referenced_locally
  let handledRequest = linkRequest;

  $effect(() => {
    if (linkRequest !== handledRequest) {
      handledRequest = linkRequest;
      openLink();
    }
  });

  function openLink() {
    linkText = formats.linkHref;
    linkError = false;
    linkOpen = true;
    queueMicrotask(() => {
      linkInput?.focus();
      linkInput?.select();
    });
  }

  function applyLink() {
    if (onSetLink(linkText)) {
      linkOpen = false;
    } else {
      linkError = true;
    }
  }

  function removeLink() {
    onSetLink(null);
    linkOpen = false;
  }

  function onLinkKeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    if (event.key === "Enter") {
      event.preventDefault();
      applyLink();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      linkOpen = false;
    }
  }

  // A floating field is a popover: it goes away when the focus moves on.
  function onLinkFocusOut(event: FocusEvent) {
    if (!floating) return;
    const next = event.relatedTarget;
    if (
      next instanceof Node &&
      (event.currentTarget as HTMLElement).contains(next)
    )
      return;
    linkOpen = false;
  }

  // A toolbar button must not take the focus on mouse down: the editor keeps its selection.
  const keepFocus = (event: MouseEvent) => event.preventDefault();
</script>

{#snippet icon(kind: Icon)}
  {#if kind === "bold"}
    <span class="glyph bold" aria-hidden="true">B</span>
  {:else if kind === "italic"}
    <span class="glyph italic" aria-hidden="true">I</span>
  {:else if kind === "underline"}
    <span class="glyph underline" aria-hidden="true">U</span>
  {:else if kind === "strike"}
    <span class="glyph strike" aria-hidden="true">S</span>
  {:else if kind === "heading"}
    <span class="glyph" aria-hidden="true">
      H{#if formats.heading}<sub>{formats.heading}</sub>{/if}
    </span>
  {:else}
    <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
      {#if kind === "code"}
        <path d="M5.5 4.5L2 8l3.5 3.5M10.5 4.5L14 8l-3.5 3.5" />
      {:else if kind === "link"}
        <path
          d="M6.9 9.1a2.5 2.5 0 0 0 3.6 0l2.2-2.2a2.5 2.5 0 0 0-3.6-3.6l-.9.9M9.1 6.9a2.5 2.5 0 0 0-3.6 0L3.3 9.1a2.5 2.5 0 0 0 3.6 3.6l.9-.9"
        />
      {:else if kind === "bulletList"}
        <path d="M6.5 4.5h7M6.5 8h7M6.5 11.5h7" />
        <path class="dots" d="M3 4.5h.01M3 8h.01M3 11.5h.01" />
      {:else if kind === "orderedList"}
        <path d="M7 4.5h6.5M7 8h6.5M7 11.5h6.5" />
        <text x="2" y="6.3">1</text>
        <text x="2" y="9.8">2</text>
        <text x="2" y="13.3">3</text>
      {:else if kind === "taskList"}
        <path
          d="M2.5 4.2l1.2 1.2 2-2.2M2.5 10.2l1.2 1.2 2-2.2M8 4.5h5.5M8 10.5h5.5"
        />
      {:else if kind === "blockquote"}
        <path d="M3 4v8M6.5 5h7M6.5 8h7M6.5 11h5" />
      {:else if kind === "codeBlock"}
        <path
          d="M6 3.5c-1.5 0-1.5 1.5-1.5 3S4 8 3 8c1 0 1.5.5 1.5 1.5s0 3 1.5 3M10 3.5c1.5 0 1.5 1.5 1.5 3S12 8 13 8c-1 0-1.5.5-1.5 1.5s0 3-1.5 3"
        />
      {:else if kind === "highlight"}
        <path d="M3 13h10M5 10.5l5.5-5.5 2 2L7 12.5H5z" />
      {:else if kind === "rule"}
        <path d="M2 8h12M5 4.5h6M5 11.5h6" />
      {:else if kind === "table"}
        <rect x="2" y="3" width="12" height="10" rx="1.5" />
        <path d="M2 7h12M2 10h12M6.5 3v10" />
      {:else if kind === "image"}
        <rect x="2" y="3" width="12" height="10" rx="1.5" />
        <path d="M2.5 11.5l3-3 2.5 2.5 2-2 3.5 3.5" />
        <circle cx="10.5" cy="6.5" r="1" />
      {:else if kind === "undo"}
        <path d="M5.5 3l-3 3 3 3M2.5 6H9a3.5 3.5 0 0 1 0 7H6.5" />
      {:else if kind === "redo"}
        <path d="M10.5 3l3 3-3 3M13.5 6H7a3.5 3.5 0 0 0 0 7h2.5" />
      {/if}
    </svg>
  {/if}
{/snippet}

{#snippet toggle(name: ToggleName, label: string, active: boolean)}
  <button
    type="button"
    class="tool"
    class:active
    aria-label={label}
    title={label}
    aria-pressed={active}
    onmousedown={keepFocus}
    onclick={() => onToggle(name)}
  >
    {@render icon(name)}
  </button>
{/snippet}

{#snippet action(name: Icon, label: string, run: (() => void) | undefined)}
  <button
    type="button"
    class="tool"
    aria-label={label}
    title={label}
    onmousedown={keepFocus}
    onclick={() => run?.()}
  >
    {@render icon(name)}
  </button>
{/snippet}

<div class="toolbar" role="toolbar" aria-label={t.notes.toolbar}>
  {@render toggle("bold", t.notes.bold, formats.bold)}
  {@render toggle("italic", t.notes.italic, formats.italic)}
  {@render toggle("underline", t.notes.underline, formats.underline)}
  {@render toggle("strike", t.notes.strike, formats.strike)}
  {@render toggle("code", t.notes.code, formats.code)}
  <button
    type="button"
    class="tool"
    class:active={formats.link}
    aria-label={t.notes.link}
    title={t.notes.link}
    aria-pressed={formats.link}
    onmousedown={keepFocus}
    onclick={() => (linkOpen ? (linkOpen = false) : openLink())}
  >
    {@render icon("link")}
  </button>
  <span class="separator" aria-hidden="true"></span>
  {@render toggle("bulletList", t.notes.bulletList, formats.bulletList)}
  {@render toggle("orderedList", t.notes.orderedList, formats.orderedList)}
  {@render toggle("taskList", t.notes.taskList, formats.taskList)}
  {#if advanced}
    <span class="separator" aria-hidden="true"></span>
    {@render toggle("heading", t.notes.heading, formats.heading > 0)}
    {@render toggle("blockquote", t.notes.blockquote, formats.blockquote)}
    {@render toggle("codeBlock", t.notes.codeBlock, formats.codeBlock)}
    {@render toggle("highlight", t.notes.highlight, formats.highlight)}
    {@render action("rule", t.notes.rule, onInsertRule)}
    {@render action("table", t.notes.table, onInsertTable)}
    {@render action("image", t.notes.image, onInsertImage)}
  {/if}
  <span class="separator history" aria-hidden="true"></span>
  <button
    type="button"
    class="tool history"
    aria-label={t.notes.undo}
    title={t.notes.undo}
    disabled={!formats.canUndo}
    onmousedown={keepFocus}
    onclick={onUndo}
  >
    {@render icon("undo")}
  </button>
  <button
    type="button"
    class="tool history"
    aria-label={t.notes.redo}
    title={t.notes.redo}
    disabled={!formats.canRedo}
    onmousedown={keepFocus}
    onclick={onRedo}
  >
    {@render icon("redo")}
  </button>
</div>

{#if linkOpen}
  <div class="link-bar" class:floating onfocusout={onLinkFocusOut}>
    <input
      bind:this={linkInput}
      bind:value={linkText}
      type="text"
      class="link-input"
      aria-label={t.notes.linkLabel}
      aria-invalid={linkError}
      placeholder={t.notes.linkPlaceholder}
      oninput={() => (linkError = false)}
      onkeydown={onLinkKeydown}
    />
    <button type="button" class="link-action" onclick={applyLink}>
      {t.notes.linkApply}
    </button>
    {#if formats.link}
      <button type="button" class="link-action" onclick={removeLink}>
        {t.notes.linkRemove}
      </button>
    {/if}
    {#if linkError}
      <span class="link-error" role="alert">{t.notes.linkInvalid}</span>
    {/if}
  </div>
{/if}

<style>
  .toolbar {
    display: inline-flex;
    align-items: center;
    gap: 1px;
    margin: var(--note-toolbar-margin, 0 0 var(--space-1));
    padding: 2px;
    border-radius: var(--radius-control);
    background: var(--note-toolbar-bg, var(--bg-control));
    max-width: 100%;
    overflow-x: auto;
    scrollbar-width: none;
    animation: fade-in var(--motion-fast) linear;
  }

  .tool {
    display: grid;
    flex: none;
    place-items: center;
    min-width: var(--note-tool-size, 26px);
    height: var(--note-tool-size, 24px);
    padding: 0 5px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .tool:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tool:active:not(:disabled) {
    transform: scale(0.92);
  }

  .tool.active {
    background: var(--ink);
    color: var(--on-ink);
  }

  .tool:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .glyph {
    font-family: var(--font-display);
    font-size: 13px;
    line-height: 1;
  }

  .glyph sub {
    font-size: 8px;
    font-weight: 700;
  }

  .glyph.bold {
    font-weight: 700;
  }

  .glyph.underline {
    text-decoration: underline;
  }

  .glyph.strike {
    text-decoration: line-through;
  }

  .glyph.italic {
    font-family: Georgia, "Times New Roman", serif;
    font-size: 14px;
    font-style: italic;
    font-weight: 500;
  }

  .tool svg {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .tool svg .dots {
    stroke-width: 2.2;
  }

  .tool svg text {
    fill: currentColor;
    stroke: none;
    font-family: var(--font-ui);
    font-size: 4.6px;
    font-weight: 700;
  }

  .tool:focus-visible,
  .link-action:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .separator {
    width: 1px;
    height: 14px;
    margin: 0 var(--space-1);
    background: var(--border-subtle);
  }

  .link-bar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 260px;
    padding: var(--space-1) 2px 2px;
  }

  /* In a header the field hangs under the bar, like the menus next to it. */
  .link-bar.floating {
    position: absolute;
    top: calc(100% + 2px);
    left: var(--space-3);
    z-index: 20;
    flex-wrap: wrap;
    width: min(340px, calc(100% - var(--space-6)));
    min-width: 0;
    box-sizing: border-box;
    padding: 6px;
    border-radius: var(--radius-menu);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    transform-origin: top left;
    animation: menu-in var(--motion-base) var(--ease-spring);
  }

  .link-input {
    flex: 1;
    min-width: 0;
    height: 26px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: var(--bg-control);
    box-shadow: inset 0 0 0 0.5px var(--border-subtle);
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    outline: none;
  }

  .link-input:focus-visible {
    box-shadow:
      inset 0 0 0 1px var(--accent),
      0 0 0 3px color-mix(in srgb, var(--accent) 35%, transparent);
  }

  .link-input[aria-invalid="true"] {
    box-shadow:
      inset 0 0 0 1px var(--danger),
      0 0 0 3px color-mix(in srgb, var(--danger) 30%, transparent);
  }

  .link-action {
    height: 24px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--accent);
    font: inherit;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .link-action:hover {
    background: var(--bg-hover);
  }

  .link-error {
    color: var(--danger);
    font-size: 12px;
  }
</style>
