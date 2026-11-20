<script lang="ts">
  import SidebarTags from "../features/tags/SidebarTags.svelte";
  import { t } from "../i18n";
  import type { NotesStore } from "../stores/notes.svelte";
  import type { TagsStore } from "../stores/tags.svelte";
  import type { TasksStore } from "../stores/tasks.svelte";
  import type { Section } from "../stores/view.svelte";
  import type { NoteSummary } from "../types";

  interface Props {
    open: boolean;
    section: Section;
    notes: NotesStore;
    tags: TagsStore;
    tasks: TasksStore;
    onSection: (section: Section) => void;
    onOpenNote: (id: string) => void;
    /** Milliseconds before the notes load for Favourites, Recent and Folders. */
    idleMs?: number;
  }

  let {
    open,
    section,
    notes,
    tags,
    tasks,
    onSection,
    onOpenNote,
    idleMs = 600,
  }: Props = $props();

  const RECENT_SHOWN = 5;

  const NAV: { value: Section; label: string; icon: string }[] = [
    {
      value: "today",
      label: t.sidebar.today,
      icon: "M8 4.5a3.5 3.5 0 110 7 3.5 3.5 0 010-7zM8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3.05 3.05l1.06 1.06M11.89 11.89l1.06 1.06M3.05 12.95l1.06-1.06M11.89 4.11l1.06-1.06",
    },
    {
      value: "upcoming",
      label: t.sidebar.upcoming,
      icon: "M2.5 4.5h11M2.5 8h7M2.5 11.5h4M11 9.5l2.5 2-2.5 2",
    },
    {
      value: "tasks",
      label: t.sidebar.tasks,
      icon: "M3.5 2.5h9a1 1 0 011 1v9a1 1 0 01-1 1h-9a1 1 0 01-1-1v-9a1 1 0 011-1zM5.5 8l1.8 1.8 3.4-3.6",
    },
    {
      value: "calendar",
      label: t.sidebar.calendar,
      icon: "M3.5 3h9a1 1 0 011 1v8.5a1 1 0 01-1 1h-9a1 1 0 01-1-1V4a1 1 0 011-1zM2.5 6.5h11M5.5 1.5V4M10.5 1.5V4",
    },
    {
      value: "events",
      label: t.sidebar.events,
      icon: "M3.5 3h9a1 1 0 011 1v8.5a1 1 0 01-1 1h-9a1 1 0 01-1-1V4a1 1 0 011-1zM2.5 6.5h11M5.5 9.5h2v2h-2z",
    },
    {
      value: "habits",
      label: t.sidebar.habits,
      icon: "M2.5 8.5l3 3 5-6M10 11.5h3.5",
    },
    {
      value: "summary",
      label: t.sidebar.summary,
      icon: "M3 13V8.5M7 13V3.5M11 13V6.5M2 13.5h12",
    },
    {
      value: "notes",
      label: t.sidebar.notes,
      icon: "M3 4h10M3 8h10M3 12h6",
    },
  ];

  let Folders = $state<
    typeof import("../features/notes/SidebarFolders.svelte").default | null
  >(null);
  let pinned = $state.raw<NoteSummary[]>([]);
  let showFavorites = $state(true);
  let showRecent = $state(true);

  // Folders, favourites and recent notes come from the notes: they load once the window is idle,
  // so they do not delay the first paint (N-01).
  $effect(() => {
    const timer = setTimeout(() => {
      void Promise.all([
        import("../features/notes/SidebarFolders.svelte"),
        notes.ensureLoaded(),
      ]).then(([module]) => {
        Folders = module.default;
      });
    }, idleMs);
    return () => clearTimeout(timer);
  });

  // The pinned notes follow every change of the list: pinning, renaming or deleting a note.
  $effect(() => {
    if (notes.status !== "ready") return;
    void notes.notes;
    void notes.folders;
    let current = true;
    void notes.pinnedNotes().then((list) => {
      if (current) pinned = list;
    });
    return () => {
      current = false;
    };
  });

  const recentKey = $derived(notes.recent.join(","));
  $effect(() => {
    if (recentKey) void notes.ensureTitles(recentKey.split(","));
  });

  const recent = $derived(
    notes
      .recentNotes()
      .filter((note) => !pinned.some((other) => other.id === note.id))
      .slice(0, RECENT_SHOWN),
  );

  function openNote(id: string) {
    onOpenNote(id);
  }
</script>

<aside
  class="app-sidebar"
  class:open
  aria-label={t.sidebar.label}
  inert={!open}
>
  <div class="inner">
    <nav class="nav" aria-label={t.sidebar.sections}>
      {#each NAV as item (item.value)}
        <button
          type="button"
          class="item"
          class:active={section === item.value}
          aria-current={section === item.value ? "page" : undefined}
          onclick={() => onSection(item.value)}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
              d={item.icon}
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <span class="label">{item.label}</span>
        </button>
      {/each}
    </nav>

    {#if pinned.length > 0}
      {@render group(
        "favorites",
        t.sidebar.favorites,
        showFavorites,
        () => (showFavorites = !showFavorites),
        pinned.map((note) => ({ id: note.id, title: note.title })),
        "M8 2l1.8 3.7 4 .6-2.9 2.8.7 4L8 11.2 4.4 13.1l.7-4L2.2 6.3l4-.6z",
      )}
    {/if}

    {#if recent.length > 0}
      {@render group(
        "recent",
        t.sidebar.recent,
        showRecent,
        () => (showRecent = !showRecent),
        recent,
        "M4.5 2.5h5l2.5 2.5v8.5h-7.5zM9.5 2.5V5H12",
      )}
    {/if}

    {#if Folders}
      <Folders
        {notes}
        {tags}
        full={section === "notes"}
        pendingOf={(id) =>
          tasks.tasks.filter(
            (task) => !task.isCompleted && task.tagIds.includes(id),
          ).length}
        onNavigate={() => onSection("notes")}
      />
    {/if}

    {#if section !== "notes"}
      <SidebarTags {tags} {tasks} />
    {/if}
  </div>
</aside>

{#snippet group(
  id: string,
  title: string,
  expanded: boolean,
  toggle: () => void,
  items: { id: string; title: string }[],
  icon: string,
)}
  <section class="group" aria-labelledby="sidebar-{id}">
    <h2 id="sidebar-{id}">
      <button
        type="button"
        class="group-toggle"
        aria-expanded={expanded}
        aria-label={expanded
          ? t.sidebar.collapseGroup(title)
          : t.sidebar.expandGroup(title)}
        onclick={toggle}
      >
        {title}
        <svg viewBox="0 0 12 12" width="9" height="9" aria-hidden="true">
          <path
            d={expanded ? "M2.5 4.5L6 8l3.5-3.5" : "M4.5 2.5L8 6 4.5 9.5"}
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </h2>
    {#if expanded}
      <ul>
        {#each items as item (item.id)}
          <li>
            <button
              type="button"
              class="item"
              data-note-id={item.id}
              onclick={() => openNote(item.id)}
            >
              <svg
                class:star={id === "favorites"}
                viewBox="0 0 16 16"
                width="14"
                height="14"
                aria-hidden="true"
              >
                <path
                  d={icon}
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.3"
                  stroke-linejoin="round"
                />
              </svg>
              <span class="label">{item.title || t.notesApp.untitled}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/snippet}

<style>
  .app-sidebar {
    flex: none;
    width: 0;
    overflow: hidden;
    color: var(--text-primary);
    transition: width var(--motion-base) var(--ease-out);
  }

  .app-sidebar.open {
    width: var(--sidebar-width);
  }

  /* Fixed width: the list never reflows while the panel slides, only fades and shifts. */
  .inner {
    display: flex;
    flex-direction: column;
    gap: 22px;
    box-sizing: border-box;
    width: var(--sidebar-width);
    height: 100%;
    padding: var(--space-1) var(--space-2) var(--space-3);
    overflow-y: auto;
    opacity: 0;
    transform: translateX(-16px);
    transition:
      opacity var(--motion-base) var(--ease-out),
      transform var(--motion-base) var(--ease-out);
  }

  .open .inner {
    opacity: 1;
    transform: none;
  }

  .nav,
  ul {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 34px;
    padding: 5px 10px;
    border: 0;
    border-radius: var(--radius-card);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13.5px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      transform var(--motion-fast) var(--ease-spring);
  }

  .item:active {
    transform: scale(0.98);
    transition-duration: var(--motion-instant);
  }

  .item:hover {
    background: var(--bg-hover);
  }

  .item.active {
    background: var(--bg-active);
    font-weight: 600;
  }

  .item svg {
    flex: none;
    color: var(--text-secondary);
  }

  .item.active svg {
    color: var(--sidebar-accent);
  }

  /* Favourites carry a filled yellow star, as pinned items do in the reference. */
  .item svg.star {
    color: #ffcc00;
  }

  .item svg.star path {
    fill: currentColor;
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  h2 {
    margin: 0;
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .group-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 10px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    cursor: pointer;
    transition: color var(--motion-fast) var(--ease-out);
  }

  .group-toggle svg {
    transition: transform var(--motion-base) var(--ease-spring);
  }

  .group-toggle:hover {
    color: var(--text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (prefers-reduced-motion: reduce) {
    .app-sidebar,
    .inner {
      transition: none;
    }
  }
</style>
