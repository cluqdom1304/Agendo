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
    const first = dialog?.querySelector<HTMLElement>(FOCUSABLE);
    (first ?? dialog)?.focus();
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
    class="dialog"
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
      &times;
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
    background: rgba(0, 0, 0, 0.28);
  }

  .dialog {
    position: relative;
    box-sizing: border-box;
    width: min(520px, calc(100vw - 2 * var(--space-6)));
    max-height: 84vh;
    padding: var(--space-4) var(--space-6) var(--space-6);
    overflow-y: auto;
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    background: var(--bg-window-solid);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
    animation: modal-in var(--motion-fast) var(--ease-apple);
  }

  .close {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-secondary);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }

  .close:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .close:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @keyframes modal-in {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
  }
</style>
