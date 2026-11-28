<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";

  // A picture of a note shown large (F12-02). Esc, the button or a click outside close it.
  interface Props {
    url: string;
    onClose: () => void;
  }

  let { url, onClose }: Props = $props();

  let close = $state<HTMLButtonElement>();
  let opener: Element | null = null;

  onMount(() => {
    opener = document.activeElement;
    close?.focus();
    return () => {
      if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
    };
  });

  function onKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    } else if (event.key === "Tab") {
      // The only control is the close button: the focus stays on it.
      event.preventDefault();
      close?.focus();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={t.notesApp.imagePreview}
    tabindex="-1"
    onkeydown={onKeydown}
    onclick={(event) => event.stopPropagation()}
  >
    <img src={url} alt="" />
    <button
      bind:this={close}
      type="button"
      class="close"
      aria-label={t.notesApp.imageClose}
      title={t.notesApp.imageClose}
      onclick={onClose}
    >
      <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
        <path
          d="M4 4l8 8M12 4l-8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 22;
    display: grid;
    place-items: center;
    padding: var(--space-6);
    background: var(--backdrop);
    animation: fade-in var(--motion-fast) linear;
  }

  .dialog {
    position: relative;
    display: grid;
    place-items: center;
    max-width: 100%;
    max-height: 100%;
    outline: none;
    animation: sheet-in var(--motion-spring) var(--ease-spring);
  }

  img {
    display: block;
    max-width: calc(100vw - 2 * var(--space-6));
    max-height: calc(100vh - 2 * var(--space-6));
    border-radius: var(--radius-panel);
    box-shadow: var(--shadow-modal);
    object-fit: contain;
  }

  .close {
    position: absolute;
    top: var(--space-3);
    right: var(--space-3);
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--bg-popover);
    color: var(--text-primary);
    box-shadow: var(--shadow-popover);
    cursor: pointer;
    transition: transform var(--motion-instant) var(--ease-out);
  }

  .close:hover {
    transform: scale(1.06);
  }

  .close:active {
    transform: scale(0.92);
  }

  .close:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .backdrop,
    .dialog {
      animation: none;
    }
  }
</style>
