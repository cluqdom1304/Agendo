<script lang="ts">
  import { onMount, type Snippet } from "svelte";

  interface Props {
    label: string;
    closeLabel: string;
    onClose: () => void;
    children: Snippet;
  }

  let { label, closeLabel, onClose, children }: Props = $props();

  let dialog = $state<HTMLDivElement>();

  const FOCUSABLE =
    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  onMount(() => {
    const opener = document.activeElement;
    // The field that wants the focus, else the first control that is not the close button.
    const target =
      dialog?.querySelector<HTMLElement>("[data-autofocus]") ??
      [...(dialog?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? [])].find(
        (element) => !element.classList.contains("close"),
      );
    (target ?? dialog)?.focus();
    return () => {
      if (opener instanceof HTMLElement) opener.focus();
    };
  });

  function onkeydown(event: KeyboardEvent) {
    // The dialog owns the keyboard while it is open: nothing reaches the document shortcuts.
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key !== "Tab" || !dialog) return;
    const items = [...dialog.querySelectorAll<HTMLElement>(FOCUSABLE)];
    if (items.length === 0) {
      event.preventDefault();
      return;
    }
    const first = items[0];
    const last = items[items.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    bind:this={dialog}
    class="dialog ui-form"
    role="dialog"
    aria-modal="true"
    aria-label={label}
    tabindex="-1"
    {onkeydown}
    onclick={(event) => event.stopPropagation()}
  >
    <button
      type="button"
      class="close"
      aria-label={closeLabel}
      onclick={onClose}
    >
      <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
        <path
          d="M1.5 1.5l7 7M8.5 1.5l-7 7"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
        />
      </svg>
    </button>
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 8vh;
    background: var(--backdrop);
    animation: fade-in var(--motion-fast) linear;
  }

  .dialog {
    position: relative;
    box-sizing: border-box;
    width: min(520px, calc(100vw - 2 * var(--space-6)));
    max-height: 84vh;
    padding: 20px var(--space-6) var(--space-6);
    overflow-y: auto;
    border-radius: var(--radius-panel);
    background: var(--bg-popover);
    box-shadow: var(--shadow-modal);
    outline: none;
    animation: sheet-in var(--motion-spring) var(--ease-spring);
  }

  .close {
    position: absolute;
    top: var(--space-3);
    right: var(--space-3);
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--bg-control);
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .close:hover {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .close:active {
    transform: scale(0.9);
  }

  .close:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
