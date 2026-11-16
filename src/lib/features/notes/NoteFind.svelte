<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import type { FindState } from "./editor";

  // Find inside the open note (F12-04). The editor marks the matches; this bar types the text
  // and walks through them.
  interface Props {
    find: (query: string) => FindState;
    step: (direction: 1 | -1) => FindState;
    onClose: () => void;
  }

  let { find, step, onClose }: Props = $props();

  let query = $state("");
  let found = $state<FindState>({ count: 0, current: -1 });
  let input = $state<HTMLInputElement>();

  onMount(() => {
    input?.focus();
    input?.select();
  });

  function onInput() {
    found = find(query);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    // The bar owns these keys while it has the focus.
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
    } else if (event.key === "Enter") {
      event.preventDefault();
      found = step(event.shiftKey ? -1 : 1);
    }
  }

  const summary = $derived(
    query === ""
      ? ""
      : found.count === 0
        ? t.notesApp.findNone
        : t.notesApp.findCount(found.current + 1, found.count),
  );
</script>

<div class="find" role="search" aria-label={t.notesApp.find}>
  <input
    bind:this={input}
    bind:value={query}
    type="text"
    class="field"
    aria-label={t.notesApp.findLabel}
    placeholder={t.notesApp.findPlaceholder}
    spellcheck="false"
    oninput={onInput}
    onkeydown={onKeydown}
  />
  <span class="count" role="status" aria-live="polite">{summary}</span>
  <button
    type="button"
    class="step"
    aria-label={t.notesApp.findPrevious}
    title={t.notesApp.findPrevious}
    disabled={found.count === 0}
    onclick={() => (found = step(-1))}
  >
    <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
      <path d="M2.5 7.5L6 4l3.5 3.5" />
    </svg>
  </button>
  <button
    type="button"
    class="step"
    aria-label={t.notesApp.findNext}
    title={t.notesApp.findNext}
    disabled={found.count === 0}
    onclick={() => (found = step(1))}
  >
    <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
      <path d="M2.5 4.5L6 8l3.5-3.5" />
    </svg>
  </button>
  <button
    type="button"
    class="step"
    aria-label={t.notesApp.findClose}
    title={t.notesApp.findClose}
    onclick={onClose}
  >
    <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
      <path d="M3 3l6 6M9 3l-6 6" />
    </svg>
  </button>
</div>

<style>
  .find {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    margin: 0 0 var(--space-3);
    padding: 3px 4px 3px var(--space-3);
    border-radius: var(--radius-control);
    background: var(--bg-control);
    box-shadow: inset 0 0 0 0.5px var(--border-subtle);
    animation: rise-in var(--motion-fast) var(--ease-out);
  }

  .find:focus-within {
    box-shadow:
      inset 0 0 0 1px var(--accent),
      0 0 0 3px color-mix(in srgb, var(--accent) 35%, transparent);
  }

  .field {
    flex: 1;
    min-width: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    outline: none;
  }

  .field::placeholder {
    color: var(--text-secondary);
  }

  .count {
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .step {
    display: grid;
    flex: none;
    place-items: center;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .step:hover:not(:disabled) {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .step:active:not(:disabled) {
    transform: scale(0.9);
  }

  .step:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .step:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  svg {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  @media (prefers-reduced-motion: reduce) {
    .find {
      animation: none;
    }
  }
</style>
