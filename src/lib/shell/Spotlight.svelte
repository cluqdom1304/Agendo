<script lang="ts">
  import { onMount } from "svelte";

  import { formatDisplayDate } from "../dates";
  import { t } from "../i18n";
  import type { SearchHit } from "../types";

  interface Props {
    search: (query: string) => Promise<SearchHit[]>;
    onSelect: (hit: SearchHit) => void;
    onClose: () => void;
  }

  let { search, onSelect, onClose }: Props = $props();

  let query = $state("");
  let hits = $state.raw<SearchHit[]>([]);
  let active = $state(0);
  let failed = $state(false);
  let input = $state<HTMLInputElement>();

  // Answers can arrive out of order; only the latest request may update the list.
  let latest = 0;

  const hasQuery = $derived(query.trim().length > 0);
  const status = $derived(
    failed
      ? t.search.failed
      : !hasQuery
        ? t.search.hint
        : hits.length === 0
          ? t.search.empty
          : t.search.resultCount(hits.length),
  );

  onMount(() => input?.focus());

  async function run(text: string) {
    const request = ++latest;
    if (!text.trim()) {
      hits = [];
      failed = false;
      return;
    }
    try {
      const found = await search(text);
      if (request !== latest) return;
      hits = found;
      active = 0;
      failed = false;
    } catch {
      if (request !== latest) return;
      hits = [];
      failed = true;
    }
  }

  function choose(index: number) {
    const hit = hits[index];
    if (hit) onSelect(hit);
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    // The dialog owns the keyboard while it is open: nothing reaches the document shortcuts.
    event.stopPropagation();
    const key = event.key.toLowerCase();
    if (event.key === "Escape" || (event.ctrlKey && key === "k")) {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown" && hits.length > 0) {
      event.preventDefault();
      active = (active + 1) % hits.length;
    } else if (event.key === "ArrowUp" && hits.length > 0) {
      event.preventDefault();
      active = (active - 1 + hits.length) % hits.length;
    } else if (event.key === "Enter") {
      event.preventDefault();
      choose(active);
    }
  }

  $effect(() => {
    document
      .getElementById(`hit-${active}`)
      ?.scrollIntoView?.({ block: "nearest" });
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={t.search.title}
    tabindex="-1"
    {onkeydown}
    onclick={(event) => event.stopPropagation()}
  >
    <input
      bind:this={input}
      bind:value={query}
      oninput={() => void run(query)}
      class="query"
      type="text"
      role="combobox"
      autocomplete="off"
      spellcheck="false"
      aria-label={t.search.placeholder}
      aria-expanded={hits.length > 0}
      aria-controls="search-results"
      aria-activedescendant={hits.length > 0 ? `hit-${active}` : undefined}
      placeholder={t.search.placeholder}
    />

    <ul
      id="search-results"
      class="results"
      role="listbox"
      aria-label={t.search.results}
    >
      {#each hits as hit, index (hit.taskId)}
        <li
          id={`hit-${index}`}
          role="option"
          aria-selected={index === active}
          class="hit"
          class:active={index === active}
          onmousemove={() => (active = index)}
          onclick={() => choose(index)}
        >
          <span class="title" class:done={hit.isCompleted}>{hit.title}</span>
          {#if hit.dueDate}
            <span class="date">{formatDisplayDate(hit.dueDate)}</span>
          {/if}
          {#if hit.snippet}
            <span class="snippet">{hit.snippet}</span>
          {/if}
        </li>
      {/each}
    </ul>

    <p class="status" role="status">{status}</p>
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
    padding-top: 12vh;
    background: var(--backdrop);
    animation: fade-in var(--motion-fast) linear;
  }

  .dialog {
    width: min(600px, calc(100vw - 2 * var(--space-6)));
    border-radius: var(--radius-panel);
    background: var(--bg-popover);
    box-shadow: var(--shadow-modal);
    overflow: hidden;
    outline: none;
    animation: sheet-in var(--motion-spring) var(--ease-spring);
  }

  .query {
    width: 100%;
    box-sizing: border-box;
    padding: 14px 18px;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-family: var(--font-display);
    font-size: 18px;
    letter-spacing: -0.01em;
    outline: none;
  }

  .query::placeholder {
    color: var(--text-secondary);
  }

  .results {
    max-height: 320px;
    margin: 0;
    padding: 6px;
    border-top: 1px solid var(--separator);
    list-style: none;
    overflow-y: auto;
  }

  .results:empty {
    display: none;
  }

  .hit {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 1px var(--space-3);
    padding: 6px 10px;
    border-radius: var(--radius-control);
    cursor: pointer;
  }

  /* Solid accent, as in a system launcher: the choice is unmistakable at a glance. */
  .hit.active {
    background: var(--accent);
  }

  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .title.done {
    color: var(--text-secondary);
    text-decoration: line-through;
  }

  .date {
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .snippet {
    grid-column: 1 / -1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .hit.active .title,
  .hit.active .date,
  .hit.active .snippet {
    color: var(--on-accent);
  }

  .status {
    margin: 0;
    padding: var(--space-2) 18px;
    border-top: 1px solid var(--separator);
    color: var(--text-secondary);
    font-size: 12px;
  }

  @media (prefers-reduced-motion: reduce) {
    .backdrop,
    .dialog {
      animation: none;
    }
  }
</style>
