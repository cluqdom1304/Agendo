<script lang="ts">
  import { t } from "../../i18n";
  import type { ToggleName, ToolbarState } from "./editor";

  interface Props {
    formats: ToolbarState;
    /** Changes every time the editor asks for the link field (Ctrl+L). */
    linkRequest: number;
    onToggle: (name: ToggleName) => void;
    /** Returns false when the address is refused. `null` removes the link. */
    onSetLink: (href: string | null) => boolean;
    onUndo: () => void;
    onRedo: () => void;
  }

  let { formats, linkRequest, onToggle, onSetLink, onUndo, onRedo }: Props =
    $props();

  let linkOpen = $state(false);
  let linkText = $state("");
  let linkError = $state(false);
  let linkInput = $state<HTMLInputElement>();
  let handledRequest = 0;

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
    queueMicrotask(() => linkInput?.select());
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

  // A toolbar button must not take the focus on mouse down: the editor keeps its selection.
  const keepFocus = (event: MouseEvent) => event.preventDefault();
</script>

{#snippet toggle(
  name: ToggleName,
  label: string,
  active: boolean,
  glyph: string,
)}
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
    {glyph}
  </button>
{/snippet}

<div class="toolbar" role="toolbar" aria-label={t.notes.toolbar}>
  {@render toggle("bold", t.notes.bold, formats.bold, "B")}
  {@render toggle("italic", t.notes.italic, formats.italic, "I")}
  {@render toggle("code", t.notes.code, formats.code, "</>")}
  <button
    type="button"
    class="tool"
    class:active={formats.link}
    aria-label={t.notes.link}
    title={t.notes.link}
    aria-pressed={formats.link}
    onmousedown={keepFocus}
    onclick={openLink}
  >
    &#128279;
  </button>
  <span class="separator" aria-hidden="true"></span>
  {@render toggle("bulletList", t.notes.bulletList, formats.bulletList, "•")}
  {@render toggle(
    "orderedList",
    t.notes.orderedList,
    formats.orderedList,
    "1.",
  )}
  {@render toggle("taskList", t.notes.taskList, formats.taskList, "☑")}
  <span class="separator" aria-hidden="true"></span>
  <button
    type="button"
    class="tool"
    aria-label={t.notes.undo}
    title={t.notes.undo}
    disabled={!formats.canUndo}
    onmousedown={keepFocus}
    onclick={onUndo}
  >
    &#8630;
  </button>
  <button
    type="button"
    class="tool"
    aria-label={t.notes.redo}
    title={t.notes.redo}
    disabled={!formats.canRedo}
    onmousedown={keepFocus}
    onclick={onRedo}
  >
    &#8631;
  </button>
</div>

{#if linkOpen}
  <div class="link-bar">
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
    animation: fade-in var(--motion-fast) linear;
  }

  .tool {
    min-width: 26px;
    height: 24px;
    padding: 0 6px;
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
    background: var(--thumb);
    box-shadow: var(--shadow-raised);
    color: var(--text-primary);
  }

  .tool:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .tool:focus-visible,
  .link-action:focus-visible {
    outline: 1.5px solid var(--accent);
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
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  .link-input[aria-invalid="true"] {
    box-shadow: inset 0 0 0 1px var(--danger);
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
    cursor: pointer;
  }

  .link-action:hover {
    background: var(--bg-hover);
  }

  .link-error {
    color: var(--danger);
    font-size: 12px;
  }
</style>
