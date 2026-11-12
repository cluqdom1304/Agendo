<script lang="ts">
  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import type { NotesSortKey, NotesStore } from "../../stores/notes.svelte";
  import { folderDrag } from "../../stores/tagDrag.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import type { NoteSummary } from "../../types";
  import TagChip from "../tags/TagChip.svelte";
  import { formatNoteDate } from "./format";
  import { virtualRange } from "./virtual";

  interface Props {
    notes: NotesStore;
    tags: TagsStore;
    onCreate: () => void;
    now?: () => Date;
  }

  const ROW_HEIGHT = 78;

  let { notes, tags, onCreate, now = () => new Date() }: Props = $props();

  let scroller = $state<HTMLDivElement>();
  let scrollTop = $state(0);
  let viewport = $state(600);

  const range = $derived(
    virtualRange(scrollTop, viewport, ROW_HEIGHT, notes.notes.length),
  );
  const visible = $derived(notes.notes.slice(range.start, range.end));
  const inTrash = $derived(notes.scope === "trash");
  const title = $derived(
    notes.scopeTagId
      ? t.notesApp.tagScopeTitle(tags.byId.get(notes.scopeTagId)?.name ?? "")
      : notes.scopeTitle,
  );

  const SORTS: { value: NotesSortKey; label: string }[] = [
    { value: "updated", label: t.notesApp.sortUpdated },
    { value: "created", label: t.notesApp.sortCreated },
    { value: "title", label: t.notesApp.sortTitle },
  ];

  const empty = $derived(
    inTrash
      ? {
          title: t.notesApp.emptyTrashTitle,
          description: t.notesApp.emptyTrashDescription,
        }
      : notes.scope === "pinned"
        ? {
            title: t.notesApp.emptyPinnedTitle,
            description: t.notesApp.emptyPinnedDescription,
          }
        : {
            title: t.notesApp.emptyTitle,
            description: t.notesApp.emptyDescription,
          },
  );

  function chips(note: NoteSummary) {
    return note.tagIds
      .map((id) => tags.byId.get(id))
      .filter((tag) => tag !== undefined)
      .slice(0, 2);
  }

  function press(event: PointerEvent, note: NoteSummary) {
    if (inTrash) return;
    folderDrag.begin(
      event,
      { id: `note:${note.id}`, title: note.title || t.notesApp.untitled },
      (key) => void notes.moveNote(note.id, key === "inbox" ? null : key),
    );
  }

  // Arrow keys move between rows; Enter and Space open through the button itself.
  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    const list = notes.notes;
    if (list.length === 0) return;
    event.preventDefault();
    const current = list.findIndex((note) => note.id === notes.open?.id);
    const next = Math.min(
      list.length - 1,
      Math.max(0, current + (event.key === "ArrowDown" ? 1 : -1)),
    );
    const target = list[next];
    if (!target) return;
    void notes.openNote(target.id);
    // The row may sit outside the rendered window: bring it into view, then focus it.
    if (scroller) {
      const top = next * ROW_HEIGHT;
      if (top < scroller.scrollTop) scroller.scrollTop = top;
      else if (top + ROW_HEIGHT > scroller.scrollTop + scroller.clientHeight) {
        scroller.scrollTop = top + ROW_HEIGHT - scroller.clientHeight;
      }
    }
    requestAnimationFrame(() =>
      scroller
        ?.querySelector<HTMLElement>(`[data-note-id="${target.id}"]`)
        ?.focus(),
    );
  }
</script>

<section class="list" aria-label={t.notesApp.list}>
  <header class="head" data-tauri-drag-region>
    <div class="titles" data-tauri-drag-region>
      <h2 data-tauri-drag-region>{title}</h2>
      <span class="count">{t.notesApp.count(notes.notes.length)}</span>
    </div>
    {#if !inTrash}
      <label class="sort">
        <span class="visually-hidden">{t.notesApp.sort}</span>
        <select
          value={notes.sort}
          onchange={(event) =>
            void notes.setSort(event.currentTarget.value as NotesSortKey)}
        >
          {#each SORTS as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </label>
      <button
        type="button"
        class="icon-btn new"
        aria-label={t.notesApp.newNote}
        title={t.notesApp.newNoteHint}
        onclick={onCreate}
      >
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path
            d="M9 3H4.5C3.7 3 3 3.7 3 4.5v7c0 .8.7 1.5 1.5 1.5h7c.8 0 1.5-.7 1.5-1.5V7M11.5 2.5l2 2-5 5H6.5v-2z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    {/if}
  </header>

  {#if notes.notes.length === 0}
    <EmptyState
      title={empty.title}
      description={empty.description}
      actionLabel={inTrash || notes.scope === "pinned"
        ? undefined
        : t.notesApp.newNote}
      shortcut="Ctrl+N"
      onAction={onCreate}
    />
  {:else}
    <div
      class="scroller"
      bind:this={scroller}
      bind:clientHeight={viewport}
      onscroll={(event) => (scrollTop = event.currentTarget.scrollTop)}
      onkeydown={onKeydown}
      role="listbox"
      tabindex="-1"
      aria-label={t.notesApp.list}
    >
      <div style:height="{range.before}px" aria-hidden="true"></div>
      {#each visible as note (note.id)}
        {@const selected = notes.open?.id === note.id}
        <div class="slot" style:height="{ROW_HEIGHT}px" role="presentation">
          <div class="item" class:selected role="presentation">
            <button
              type="button"
              class="card"
              role="option"
              aria-selected={selected}
              data-note-id={note.id}
              aria-label={t.notesApp.open(note.title || t.notesApp.untitled)}
              onclick={() => void notes.openNote(note.id)}
              onpointerdown={(event) => press(event, note)}
            >
              <span class="line">
                <span class="title" class:placeholder={!note.title}>
                  {note.title || t.notesApp.untitled}
                </span>
                {#if note.isPinned}
                  <svg
                    class="pin"
                    viewBox="0 0 16 16"
                    width="11"
                    height="11"
                    role="img"
                    aria-label={t.notesApp.pinnedMark}
                  >
                    <path
                      d="M9.8 2.5l3.7 3.7-1.6.5-2.3 2.3.2 3-.9.9-2.7-2.7-3.5 3.5"
                      fill="currentColor"
                      stroke="currentColor"
                      stroke-width="1.2"
                      stroke-linejoin="round"
                    />
                  </svg>
                {/if}
              </span>
              <span class="meta">
                <span class="date">{formatNoteDate(note.updatedAt, now())}</span
                >
                <span class="excerpt">{note.excerpt || t.notesApp.noText}</span>
              </span>
              {#if note.tagIds.length > 0}
                <span class="tags">
                  {#each chips(note) as tag (tag.id)}
                    <TagChip name={tag.name} color={tag.colorHex} />
                  {/each}
                </span>
              {/if}
            </button>
            {#if inTrash}
              <button
                type="button"
                class="restore"
                onclick={() => void notes.restoreNote(note.id)}
              >
                {t.notesApp.restore}
              </button>
            {/if}
          </div>
        </div>
      {/each}
      <div style:height="{range.after}px" aria-hidden="true"></div>
    </div>
  {/if}
</section>

<style>
  .list {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
  }

  .head {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-2);
    height: 44px;
    padding: 0 var(--space-3);
  }

  .titles {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  h2 {
    margin: 0;
    overflow: hidden;
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .sort select {
    height: 26px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: var(--bg-control);
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .sort select:hover {
    background: var(--bg-control-hover);
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }

  .scroller {
    flex: 1;
    min-height: 0;
    padding: 0 var(--space-2);
    overflow-y: auto;
    outline: none;
  }

  .slot {
    box-sizing: border-box;
    padding: 2px 0;
  }

  .item {
    position: relative;
    height: 100%;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-instant) linear;
  }

  .item:hover {
    background: var(--bg-hover);
  }

  .item.selected {
    background: var(--accent);
    color: var(--on-accent);
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    height: 100%;
    padding: 9px var(--space-3);
    border: 0;
    border-radius: inherit;
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
  }

  .line {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-size: 13px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title.placeholder {
    opacity: 0.6;
  }

  .pin {
    flex: none;
    opacity: 0.7;
  }

  .meta {
    display: flex;
    gap: var(--space-2);
    min-width: 0;
    font-size: 12px;
  }

  .date {
    flex: none;
    font-variant-numeric: tabular-nums;
  }

  .excerpt {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .date,
  .excerpt {
    color: var(--text-secondary);
  }

  .selected .date,
  .selected .excerpt {
    color: color-mix(in srgb, var(--on-accent) 80%, transparent);
  }

  .tags {
    display: flex;
    gap: var(--space-1);
    margin-top: 2px;
  }

  .restore {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 2px 10px;
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--accent);
    color: var(--on-accent);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .card:focus-visible,
  .restore:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .selected .card:focus-visible {
    outline-color: var(--on-accent);
  }
</style>
