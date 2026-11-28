<script lang="ts">
  import { onMount } from "svelte";

  import { formatDisplayDate } from "../../dates";
  import { t } from "../../i18n";
  import { parseNatural } from "../../naturalInput";
  import type { FoundItem } from "../../types";
  import { captureChips, captureOptions } from "../capture/capture.svelte";
  import { toLocalIsoDate } from "../../dates";
  import {
    buildGroups,
    type CreateKind,
    planQuery,
    type PaletteCommand,
    type PaletteMode,
    type PaletteRow,
  } from "./palette";

  // Ctrl+K searches tasks, notes and tags and runs commands; Ctrl+P ("Ir a…") only jumps to
  // notes. One dialog, two modes (F11-03 to F11-05).
  interface Props {
    mode: PaletteMode;
    search: (query: string) => Promise<FoundItem[]>;
    commands: PaletteCommand[];
    /** Notes opened lately, most recent first. */
    recent: { id: string; title: string }[];
    /** Saved searches; choosing one fills the box with it (F17-08). */
    views?: string[];
    onSaveView?: (query: string) => void;
    onDeleteView?: (query: string) => void;
    onNote: (id: string) => void;
    onTask: (id: string) => void;
    onTag: (id: string) => void;
    /** Creates what the text describes; without it the «Crear» rows are not offered. */
    onCreate?: (what: CreateKind, text: string) => void;
    onClose: () => void;
  }

  let {
    mode,
    search,
    commands,
    recent,
    views = [],
    onSaveView,
    onDeleteView,
    onNote,
    onTask,
    onTag,
    onCreate,
    onClose,
  }: Props = $props();

  let query = $state("");
  let found = $state.raw<FoundItem[]>([]);
  let active = $state(0);
  let failed = $state(false);
  let input = $state<HTMLInputElement>();

  // Answers can arrive out of order; only the latest request may update the list.
  let latest = 0;

  const plan = $derived(planQuery(query, mode));
  const groups = $derived(
    buildGroups({
      mode,
      plan,
      commands,
      recent,
      found,
      views,
      create: onCreate !== undefined && mode === "all",
    }),
  );
  const canSave = $derived(
    mode === "all" &&
      !plan.commandsOnly &&
      plan.text !== "" &&
      !views.includes(plan.text) &&
      onSaveView !== undefined,
  );
  const rows = $derived(groups.flatMap((group) => group.rows));
  const groupTitles = $derived({
    create: t.palette.groupCreate,
    views: t.palette.groupViews,
    commands: t.palette.groupCommands,
    recent: t.palette.groupRecent,
    notes: t.palette.groupNotes,
    tasks: t.palette.groupTasks,
    tags: t.palette.groupTags,
  });

  const status = $derived(
    failed
      ? t.palette.failed
      : rows.length > 0
        ? plan.text === "" && !plan.commandsOnly
          ? mode === "notes"
            ? t.palette.hintNotes
            : t.palette.hintAll
          : t.palette.resultCount(rows.length)
        : plan.text === "" && !plan.commandsOnly
          ? mode === "notes"
            ? t.palette.hintNotes
            : t.palette.hintAll
          : t.palette.empty,
  );

  onMount(() => {
    input?.focus();
    // "Ir a…" lists the notes edited lately before anything is typed.
    if (mode === "notes") void run();
  });

  async function run() {
    const request = ++latest;
    const wanted = plan.search;
    if (wanted === null) {
      found = [];
      failed = false;
      active = 0;
      return;
    }
    try {
      const answer = await search(wanted);
      if (request !== latest) return;
      found = answer;
      failed = false;
    } catch {
      if (request !== latest) return;
      found = [];
      failed = true;
    }
    active = 0;
  }

  function onInput() {
    active = 0;
    void run();
  }

  function choose(row: PaletteRow | undefined) {
    if (!row) return;
    if (row.kind === "view") {
      query = row.query;
      onInput();
      return;
    }
    onClose();
    if (row.kind === "create") onCreate?.(row.what, row.text);
    else if (row.kind === "command") row.command.run();
    else if (row.kind === "note") onNote(row.id);
    else if (row.kind === "task") onTask(row.id);
    else onTag(row.id);
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    // The dialog owns the keyboard while it is open: nothing reaches the document shortcuts.
    event.stopPropagation();
    const key = event.key.toLowerCase();
    if (
      event.key === "Escape" ||
      (event.ctrlKey && (key === "k" || key === "p"))
    ) {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown" && rows.length > 0) {
      event.preventDefault();
      active = (active + 1) % rows.length;
    } else if (event.key === "ArrowUp" && rows.length > 0) {
      event.preventDefault();
      active = (active - 1 + rows.length) % rows.length;
    } else if (event.key === "Enter") {
      event.preventDefault();
      choose(rows[active]);
    }
  }

  $effect(() => {
    document
      .getElementById(`row-${active}`)
      ?.scrollIntoView?.({ block: "nearest" });
  });

  const today = () => toLocalIsoDate(new Date());
  const readText = (text: string) =>
    parseNatural(text, captureOptions(today()));
  const titleOf = (row: PaletteRow): string =>
    row.kind === "create"
      ? t.palette.create[row.what](readText(row.text)?.title ?? row.text)
      : row.kind === "command"
        ? row.command.label
        : row.kind === "view"
          ? row.query
          : row.kind === "tag"
            ? row.name
            : row.title || t.palette.untitled;
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={mode === "notes" ? t.palette.titleNotes : t.palette.titleAll}
    tabindex="-1"
    {onkeydown}
    onclick={(event) => event.stopPropagation()}
  >
    <input
      bind:this={input}
      bind:value={query}
      oninput={onInput}
      class="query"
      type="text"
      role="combobox"
      autocomplete="off"
      spellcheck="false"
      aria-label={mode === "notes"
        ? t.palette.placeholderNotes
        : t.palette.placeholderAll}
      aria-expanded={rows.length > 0}
      aria-controls="palette-results"
      aria-activedescendant={rows.length > 0 ? `row-${active}` : undefined}
      placeholder={mode === "notes"
        ? t.palette.placeholderNotes
        : t.palette.placeholderAll}
    />

    <ul
      id="palette-results"
      class="results"
      role="listbox"
      aria-label={t.palette.results}
    >
      {#each groups as group (group.id)}
        <li class="group" role="presentation">{groupTitles[group.id]}</li>
        {#each group.rows as row (row.key)}
          {@const index = rows.indexOf(row)}
          <li
            id={`row-${index}`}
            role="option"
            aria-selected={index === active}
            class="hit"
            class:active={index === active}
            onmousemove={() => (active = index)}
            onclick={() => choose(row)}
          >
            <span class="lead" aria-hidden="true">
              {#if row.kind === "view"}
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <circle
                    cx="7"
                    cy="7"
                    r="4"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                  />
                  <path
                    d="M10 10l3.5 3.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                    stroke-linecap="round"
                  />
                </svg>
              {:else if row.kind === "tag"}
                <span class="dot" style:background={row.color}></span>
              {:else if row.kind === "task"}
                <span class="box" class:done={row.done}></span>
              {:else if row.kind === "note"}
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path
                    d="M4 2.5h5.5L12.5 5.5v8H4zM9.5 2.5v3h3"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                    stroke-linejoin="round"
                  />
                </svg>
              {:else}
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path
                    d="M6 4.5L3 8l3 3.5M10 4.5L13 8l-3 3.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.4"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              {/if}
            </span>
            <span class="title" class:done={row.kind === "task" && row.done}>
              {titleOf(row)}
            </span>
            <span class="detail">
              {#if row.kind === "create"}
                {#if row.what !== "note"}
                  {@const capture = readText(row.text)}
                  {#if capture}{captureChips(capture, today()).join(" · ")}{/if}
                {/if}
              {:else if row.kind === "view"}
                <button
                  type="button"
                  class="forget"
                  aria-label={t.palette.deleteView(row.query)}
                  title={t.palette.deleteView(row.query)}
                  onclick={(event) => {
                    event.stopPropagation();
                    onDeleteView?.(row.query);
                  }}
                >
                  ✕
                </button>
              {:else if row.kind === "command"}
                {#if row.command.shortcut}
                  <kbd>{row.command.shortcut}</kbd>
                {/if}
              {:else if row.kind === "task"}
                {#if row.dueDate}{formatDisplayDate(row.dueDate)}{/if}
              {:else if row.kind === "tag"}
                {t.palette.tagCounts(row.notes, row.pending)}
              {/if}
            </span>
            {#if (row.kind === "note" || row.kind === "task") && row.snippet}
              <span class="snippet">{row.snippet}</span>
            {/if}
          </li>
        {/each}
      {/each}
    </ul>

    <p class="status" role="status">{status}</p>
    {#if canSave}
      <button
        type="button"
        class="save"
        onclick={() => onSaveView?.(plan.text)}
      >
        {t.palette.saveView}
      </button>
    {/if}
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
    animation: fade-in var(--motion-base) var(--ease-out);
  }

  .dialog {
    width: min(620px, calc(100vw - 2 * var(--space-6)));
    border-radius: var(--radius-sheet);
    background: var(--bg-popover);
    box-shadow: var(--shadow-modal);
    overflow: hidden;
    outline: none;
    animation: sheet-in var(--motion-spring) var(--ease-spring);
  }

  .query {
    width: 100%;
    box-sizing: border-box;
    padding: 16px 20px;
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
    max-height: min(360px, calc(88vh - 150px));
    margin: 0;
    padding: 6px;
    border-top: 1px solid var(--separator);
    list-style: none;
    overflow-y: auto;
  }

  .results:empty {
    display: none;
  }

  .group {
    padding: 8px 10px 3px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.01em;
  }

  .hit {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr) auto;
    align-items: center;
    gap: 1px var(--space-3);
    padding: 7px 10px;
    border-radius: var(--radius-card);
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  /* The same soft highlight as every menu (F26.1, D-101). */
  .hit.active {
    background: var(--bg-menu-hover);
  }

  .lead {
    display: flex;
    justify-content: center;
    color: var(--text-secondary);
  }

  .dot {
    width: 9px;
    height: 9px;
    border-radius: var(--radius-chip);
  }

  .box {
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
  }

  .box.done {
    border-color: var(--ink);
    border-radius: 3px;
    background: var(--ink);
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

  .detail {
    max-width: 240px;
    overflow: hidden;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  kbd {
    font: inherit;
    font-size: 11px;
  }

  .snippet {
    grid-column: 2 / -1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .save,
  .forget {
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  /* The accent is kept for what is "now" (D-101): saving a view is a plain action. */
  .save {
    margin: 0 var(--space-4) var(--space-3);
    padding: 2px 0;
    font-weight: 600;
  }

  .save:hover {
    text-decoration: underline;
  }

  .forget {
    display: inline-grid;
    place-items: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border-radius: var(--radius-chip);
    color: var(--text-secondary);
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear;
  }

  .forget:hover {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  /* Inside the clipped detail column: the ring is drawn inwards so it is not cut. */
  .forget:focus-visible {
    outline-offset: -2px;
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
