<script lang="ts">
  import { t } from "../i18n";
  import Menu, { type MenuItem } from "./Menu.svelte";

  interface Props {
    sidebarOpen: boolean;
    /** A tag filter hides tasks: the fold button carries a dot so it is not forgotten. */
    filtered: boolean;
    canGoBack: boolean;
    canGoForward: boolean;
    /** What the «new» button creates where the user is. */
    newLabel: string;
    /** The inbox toggle shows only where the panel can open (F20-04). */
    inboxAvailable?: boolean;
    inboxOpen?: boolean;
    onToggleInbox?: () => void;
    /** The countdown of the focus session in progress; `null` when there is none (F25-01). */
    focusLabel?: string | null;
    focusTitle?: string;
    focusPaused?: boolean;
    onFocusPause?: () => void;
    onFocusResume?: () => void;
    onFocusSwitch?: () => void;
    /** Ends the session and keeps the time spent. */
    onFocusFinish?: () => void;
    /** Deletes the session without keeping any time. */
    onFocusDiscard?: () => void;
    onToggleSidebar: () => void;
    onNew: () => void;
    onBack: () => void;
    onForward: () => void;
    onSearch: () => void;
    onSettings: () => void;
  }

  let {
    sidebarOpen,
    filtered,
    canGoBack,
    canGoForward,
    newLabel,
    inboxAvailable = false,
    inboxOpen = false,
    onToggleInbox,
    focusLabel = null,
    focusTitle = "",
    focusPaused = false,
    onFocusPause,
    onFocusResume,
    onFocusSwitch,
    onFocusFinish,
    onFocusDiscard,
    onToggleSidebar,
    onNew,
    onBack,
    onForward,
    onSearch,
    onSettings,
  }: Props = $props();

  const l = t.topBar;
  let focusMenuOpen = $state(false);
  const focusItems = $derived<MenuItem[]>([
    focusPaused
      ? { id: "resume", label: t.focus.resume }
      : { id: "pause", label: t.focus.pause },
    { id: "switch", label: t.focus.switchTask },
    { id: "finish", label: t.focus.finish },
    { id: "discard", label: t.focus.discard, danger: true, separated: true },
  ]);

  function selectFocus(id: string) {
    focusMenuOpen = false;
    if (id === "pause") onFocusPause?.();
    else if (id === "resume") onFocusResume?.();
    else if (id === "switch") onFocusSwitch?.();
    else if (id === "finish") onFocusFinish?.();
    else if (id === "discard") onFocusDiscard?.();
  }
  const toggleLabel = $derived(
    filtered ? `${t.sidebar.toggle}. ${t.tags.filterActive}` : t.sidebar.toggle,
  );
</script>

<!-- One bar across the window, sidebar included, and its drag region (D-39). The caption
     buttons float over its right end. -->
<div class="top-bar chrome-dark" data-tauri-drag-region>
  <div class="lead" data-tauri-drag-region>
    <button
      type="button"
      class="icon-btn fold"
      class:filtered
      aria-pressed={sidebarOpen}
      aria-label={toggleLabel}
      title={filtered ? t.tags.filterActive : t.sidebar.toggle}
      onclick={onToggleSidebar}
    >
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <rect
          x="2"
          y="3"
          width="12"
          height="10"
          rx="2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
        />
        <path d="M6 3v10" stroke="currentColor" stroke-width="1.4" />
      </svg>
    </button>
    <button
      type="button"
      class="icon-btn"
      aria-label={newLabel}
      title={newLabel}
      onclick={onNew}
    >
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path
          d="M13 9v3.5a1 1 0 01-1 1H3.5a1 1 0 01-1-1V4a1 1 0 011-1H7M11.3 2.2l2.5 2.5L8.5 10H6V7.5z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="icon-btn"
      aria-label={l.back}
      title={l.back}
      disabled={!canGoBack}
      onclick={onBack}
    >
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path
          d="M10 3.5L5.5 8l4.5 4.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="icon-btn"
      aria-label={l.forward}
      title={l.forward}
      disabled={!canGoForward}
      onclick={onForward}
    >
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path
          d="M6 3.5L10.5 8 6 12.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>

  <button
    type="button"
    class="search"
    aria-haspopup="dialog"
    aria-keyshortcuts="Control+K"
    onclick={onSearch}
  >
    <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
      <circle
        cx="7"
        cy="7"
        r="4.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      />
      <path
        d="M10.5 10.5L14 14"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
      />
    </svg>
    <span class="placeholder">{l.search}</span>
    <kbd>Ctrl K</kbd>
  </button>

  <div class="trail" data-tauri-drag-region>
    {#if focusLabel}
      <div class="anchor">
        <button
          type="button"
          class="focus"
          class:paused={focusPaused}
          data-menu-toggle
          aria-haspopup="menu"
          aria-expanded={focusMenuOpen}
          aria-label={`${t.focus.menu}: ${focusTitle}${focusPaused ? `. ${t.focus.paused}` : ""}`}
          title={`${focusPaused ? t.focus.paused : t.focus.running}: ${focusTitle}`}
          onclick={() => (focusMenuOpen = !focusMenuOpen)}
        >
          <span class="dot" aria-hidden="true"></span>
          <span class="time">{focusLabel}</span>
          {#if focusTitle}
            <span class="what">{focusTitle}</span>
          {/if}
        </button>
        {#if focusMenuOpen}
          <Menu
            label={t.focus.menu}
            items={focusItems}
            onSelect={selectFocus}
            onClose={() => (focusMenuOpen = false)}
          />
        {/if}
      </div>
    {/if}
    {#if inboxAvailable}
      <button
        type="button"
        class="icon-btn"
        aria-pressed={inboxOpen}
        aria-label={t.inbox.toggle}
        title={t.inbox.toggle}
        onclick={() => onToggleInbox?.()}
      >
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path
            d="M2.5 9.5l1.6-5.2a1 1 0 011-.8h5.8a1 1 0 011 .8l1.6 5.2v3a1 1 0 01-1 1h-9a1 1 0 01-1-1zM2.5 9.5h3l.8 1.5h3.4l.8-1.5h3"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    {/if}
    <button
      type="button"
      class="icon-btn"
      aria-label={t.settings.open}
      title={t.settings.open}
      aria-haspopup="dialog"
      onclick={onSettings}
    >
      <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
        <path
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          d="M4 7h10M18 7h2M4 17h2M10 17h10"
        />
        <circle
          cx="16"
          cy="7"
          r="2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        />
        <circle
          cx="8"
          cy="17"
          r="2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        />
      </svg>
    </button>
  </div>
</div>

<style>
  .top-bar {
    display: grid;
    grid-template-columns: 1fr minmax(180px, 420px) 1fr;
    align-items: center;
    gap: var(--space-3);
    flex: none;
    height: 48px;
    padding-left: var(--space-3);
  }

  .lead,
  .trail {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    height: 100%;
  }

  .trail {
    justify-content: flex-end;
    /* Room for the three 46 px caption buttons that float over this bar. */
    padding-right: 146px;
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .fold {
    position: relative;
  }

  .fold.filtered::after {
    content: "";
    position: absolute;
    top: 4px;
    right: 4px;
    width: 7px;
    height: 7px;
    border-radius: var(--radius-chip);
    background: var(--accent);
  }

  .focus {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 28px;
    padding: 0 var(--space-3);
    border: 0;
    border-radius: var(--radius-control);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  .focus:hover {
    background: var(--bg-control-hover);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-chip);
    background: var(--accent);
  }

  .time {
    font-variant-numeric: tabular-nums;
  }

  .anchor {
    position: relative;
  }

  .what {
    max-width: 160px;
    overflow: hidden;
    color: var(--text-secondary);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* A paused session stops pulsing the accent: the dot goes grey and the time dims. */
  .focus.paused .dot {
    background: var(--text-secondary);
  }

  .focus.paused .time {
    opacity: 0.7;
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    height: 30px;
    padding: 0 6px 0 var(--space-3);
    border: 0;
    border-radius: var(--radius-control);
    background: var(--bg-control);
    color: var(--text-secondary);
    font: inherit;
    font-size: 13px;
    cursor: text;
    transition: background-color var(--motion-instant) linear;
  }

  .search:hover {
    background: var(--bg-control-hover);
  }

  .placeholder {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-align: left;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  kbd {
    flex: none;
    padding: 1px 6px;
    border-radius: var(--radius-small);
    background: var(--bg-control);
    font: inherit;
    font-size: 11px;
  }
</style>
