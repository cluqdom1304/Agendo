<script lang="ts">
  import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
  import { t } from "../i18n";

  async function run(action: (appWindow: Window) => Promise<void>) {
    try {
      await action(getCurrentWindow());
    } catch (error) {
      console.error("Window control failed:", error);
    }
  }
</script>

<div class="flex h-full items-stretch">
  <button
    type="button"
    class="control"
    aria-label={t.window.minimize}
    title={t.window.minimize}
    onclick={() => run((appWindow) => appWindow.minimize())}
  >
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
      <path d="M1 5h8" stroke="currentColor" stroke-width="1" fill="none" />
    </svg>
  </button>
  <button
    type="button"
    class="control"
    aria-label={t.window.toggleMaximize}
    title={t.window.toggleMaximize}
    onclick={() => run((appWindow) => appWindow.toggleMaximize())}
  >
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
      <rect
        x="1"
        y="1"
        width="8"
        height="8"
        stroke="currentColor"
        stroke-width="1"
        fill="none"
      />
    </svg>
  </button>
  <button
    type="button"
    class="control close"
    aria-label={t.window.close}
    title={t.window.close}
    onclick={() => run((appWindow) => appWindow.close())}
  >
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
      <path
        d="M1 1l8 8M9 1L1 9"
        stroke="currentColor"
        stroke-width="1"
        fill="none"
      />
    </svg>
  </button>
</div>

<style>
  .control {
    display: inline-flex;
    width: 46px;
    align-items: center;
    justify-content: center;
    color: var(--text-primary);
    background: transparent;
    border: 0;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear;
  }

  .control:hover {
    background: var(--bg-hover);
  }

  .control.close:hover {
    background: var(--danger);
    color: var(--on-danger);
  }

  .control:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
