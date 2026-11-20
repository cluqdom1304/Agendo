<script lang="ts">
  import type { Snippet } from "svelte";

  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import Menu, { type MenuItem } from "../../shell/Menu.svelte";
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
    /** Drawn between the header and the list (the tabs of a tag). */
    subheader?: Snippet;
    /** Shown instead of the notes, under the same header (the tasks of a tag). */
    body?: Snippet;
    /** Replaces the "n notas" line when the column shows something else. */
    countLabel?: string;
  }

  const ROW_HEIGHT = 80;
  const MAX_CHIPS = 2;

  let {
    notes,
    tags,
    onCreate,
    now = () => new Date(),
    subheader,
    body,
    countLabel,
  }: Props = $props();

  let scroller = $state<HTMLDivElement>();
  let scrollTop = $state(0);
  let viewport = $state(600);
  let sortOpen = $state(false);

  const range = $derived(
    virtualRange(scrollTop, viewport, ROW_HEIGHT, notes.notes.length),
  );
  const inTrash = $derived(notes.scope === "trash");
  const title = $derived(
    notes.scopeTagId
      ? t.notesApp.tagScopeTitle(tags.byId.get(notes.scopeTagId)?.name ?? "")
      : notes.scopeTitle,
  );

  // Everything a row draws is worked out here, once per change, not in the template.
  const rows = $derived.by(() => {
    const today = now();
    const openId = notes.open?.id;
    return notes.notes.slice(range.start, range.end).map((note) => {
      const own = note.tagIds
        .map((id) => tags.byId.get(id))
        .filter((tag) => tag !== undefined);
      return {
        note,
        title: note.title || t.notesApp.untitled,
        date: formatNoteDate(note.updatedAt, today),
        excerpt: note.excerpt || t.notesApp.noText,
        chips: own.slice(0, MAX_CHIPS),
        extra: Math.max(0, own.length - MAX_CHIPS),
        selected: note.id === openId,
      };
    });
  });

  const sortItems = $derived<MenuItem[]>([
    {
      id: "updated",
      label: t.notesApp.sortUpdated,
      checked: notes.sort === "updated",
    },
    {
      id: "created",
      label: t.notesApp.sortCreated,
      checked: notes.sort === "created",
    },
    {
      id: "title",
      label: t.notesApp.sortTitle,
      checked: notes.sort === "title",
    },
  ]);

  const empty = $derived(
    inTrash
      ? {
          title: t.notesApp.emptyTrashTitle,
          description: t.notesApp.emptyTrashDescription,
        }
      : notes.scope === "orphans"
        ? {
            title: t.notesApp.orphansEmpty,
            description: t.notesApp.orphansHint,
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

  function press(event: PointerEvent, note: NoteSummary) {
    if (inTrash) return;
    folderDrag.begin(
      event,
      { id: `note:${note.id}`, title: note.title || t.notesApp.untitled },
      (key) => {
        // "root" is where folders go to reach the top level: a note has no such place.
        if (key === "root") return;
        const target = key === "inbox" ? null : key;
        if (target !== note.folderId) void notes.moveNote(note.id, target);
      },
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
      <span class="count" data-tauri-drag-region>
        {countLabel ?? t.notesApp.count(notes.notes.length)}
      </span>
    </div>
    {#if !inTrash}
      {#if !body}
        <div class="anchor">
          <button
            type="button"
            class="icon-btn"
            class:open={sortOpen}
            data-menu-toggle
            aria-haspopup="menu"
            aria-expanded={sortOpen}
            aria-label={t.notesApp.sort}
            title={t.notesApp.sort}
            onclick={() => (sortOpen = !sortOpen)}
          >
            <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
              <path
                d="M4.5 3v10M4.5 13l-2.3-2.4M4.5 13l2.3-2.4M9 4.5h5M9 8h3.6M9 11.5h2.2"
                fill="none"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
          {#if sortOpen}
            <Menu
              label={t.notesApp.sort}
              items={sortItems}
              onSelect={(sort) => void notes.setSort(sort as NotesSortKey)}
              onClose={() => (sortOpen = false)}
            />
          {/if}
        </div>
      {/if}
      <button
        type="button"
        class="icon-btn"
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

  {@render subheader?.()}

  {#if body}
    {@render body()}
  {:else if notes.notes.length === 0}
    <EmptyState
      title={empty.title}
      description={empty.description}
      actionLabel={inTrash ||
      notes.scope === "pinned" ||
      notes.scope === "orphans"
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
      {#each rows as row (row.note.id)}
        <div class="slot" style:height="{ROW_HEIGHT}px" role="presentation">
          <div
            class="item"
            class:selected={row.selected}
            class:in-trash={inTrash}
            role="presentation"
          >
            <button
              type="button"
              class="card"
              role="option"
              aria-selected={row.selected}
              data-note-id={row.note.id}
              aria-label={t.notesApp.open(row.title)}
              onclick={() => void notes.openNote(row.note.id)}
              onpointerdown={(event) => press(event, row.note)}
            >
              <span class="line">
                <span class="title" class:placeholder={!row.note.title}>
                  {row.title}
                </span>
                {#if row.note.isPinned}
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
              <!-- Without tags the excerpt takes the line they would use. -->
              <span class="excerpt" class:two={row.chips.length === 0}>
                <span class="date">{row.date}</span>
                {row.excerpt}
              </span>
              {#if row.chips.length > 0}
                <span class="tags">
                  {#each row.chips as tag (tag.id)}
                    <TagChip name={tag.name} color={tag.colorHex} />
                  {/each}
                  {#if row.extra > 0}
                    <span class="extra">+{row.extra}</span>
                  {/if}
                </span>
              {/if}
            </button>
            {#if inTrash}
              <button
                type="button"
                class="btn small restore"
                onclick={() => void notes.restoreNote(row.note.id)}
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
    gap: 4px;
    height: 52px;
    padding: 0 var(--space-2) 0 var(--space-4);
  }

  .titles {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    margin-right: var(--space-1);
  }

  h2 {
    margin: 0;
    overflow: hidden;
    font-family: var(--font-display);
    font-size: var(--text-title);
    font-weight: 700;
    letter-spacing: -0.015em;
    line-height: 1.3;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    line-height: 1.3;
  }

  .anchor {
    position: relative;
  }

  .icon-btn.open {
    background: var(--bg-active);
    color: var(--text-primary);
  }

  .scroller {
    flex: 1;
    min-height: 0;
    padding: 2px var(--space-2) var(--space-2);
    overflow-y: auto;
    outline: none;
  }

  .slot {
    box-sizing: border-box;
    padding: 1px 0;
  }

  .item {
    position: relative;
    height: 100%;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .item:hover {
    background: var(--bg-hover);
  }

  /* A neutral tile, as the selected item of the sidebar: the open note is clear at a glance
     and the text keeps its own colours. */
  .item.selected {
    background: var(--bg-active);
  }

  .card {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: 100%;
    height: 100%;
    padding: 10px var(--space-3);
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
    flex: none;
    align-items: center;
    gap: var(--space-1);
    height: 18px;
  }

  .in-trash .line {
    padding-right: 84px;
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-size: 14px;
    font-weight: 600;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title.placeholder {
    color: var(--text-secondary);
  }

  .pin {
    flex: none;
    color: var(--text-secondary);
  }

  .selected .pin {
    color: var(--accent);
  }

  .excerpt {
    display: block;
    flex: none;
    margin-top: 2px;
    overflow: hidden;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 17px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .excerpt.two {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow-wrap: anywhere;
    white-space: normal;
  }

  .date {
    margin-right: 4px;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .tags {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-1);
    min-width: 0;
    margin-top: 3px;
    overflow: hidden;
  }

  .extra {
    flex: none;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
  }

  .restore {
    position: absolute;
    top: 7px;
    right: 8px;
    color: var(--accent);
  }

  .card:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
