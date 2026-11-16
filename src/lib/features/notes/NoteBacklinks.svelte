<script lang="ts">
  import { t } from "../../i18n";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { NoteSummary } from "../../types";

  // "Enlaces a esta nota" (F11-02): the notes that link here. A note nobody links to shows no
  // panel at all.
  interface Props {
    notes: NotesStore;
    noteId: string;
  }

  let { notes, noteId }: Props = $props();

  let linking = $state.raw<NoteSummary[]>([]);
  let collapsed = $state(false);

  // The list of notes changes whenever one is saved, which is when a link can appear or go.
  $effect(() => {
    void notes.notes;
    let current = true;
    notes
      .backlinks(noteId)
      .then((found) => {
        if (current) linking = found;
      })
      .catch(() => {
        if (current) linking = [];
      });
    return () => {
      current = false;
    };
  });
</script>

{#if linking.length > 0}
  <section class="panel" aria-label={t.notesApp.backlinks}>
    <button
      type="button"
      class="head"
      aria-expanded={!collapsed}
      onclick={() => (collapsed = !collapsed)}
    >
      <svg
        class="chevron"
        class:turned={!collapsed}
        viewBox="0 0 12 12"
        width="10"
        height="10"
        aria-hidden="true"
      >
        <path
          d="M4 2.5l3.5 3.5L4 9.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <span class="title">{t.notesApp.backlinks}</span>
      <span class="count">{t.notesApp.backlinksCount(linking.length)}</span>
    </button>

    {#if !collapsed}
      <ul class="rows">
        {#each linking as source (source.id)}
          <li>
            <button
              type="button"
              class="name"
              title={t.notesApp.linkOpen(source.title || t.notesApp.untitled)}
              onclick={() => void notes.openNote(source.id)}
            >
              <span class="link">
                {source.title || t.notesApp.untitled}
              </span>
              {#if source.excerpt}
                <span class="excerpt">{source.excerpt}</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

<style>
  .panel {
    flex: none;
    margin-top: var(--space-4);
    padding-top: var(--space-2);
    border-top: 1px solid var(--separator);
    animation: fade-in var(--motion-base) var(--ease-out);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 4px 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .chevron {
    flex: none;
    transition: transform var(--motion-fast) var(--ease-out);
  }

  .chevron.turned {
    transform: rotate(90deg);
  }

  .title {
    flex: 1;
    font-size: 12px;
    font-weight: 600;
  }

  .count {
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .name {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    box-sizing: border-box;
    width: calc(100% + 2 * var(--space-2));
    min-height: 28px;
    margin: 0 calc(var(--space-2) * -1);
    padding: 4px var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--accent);
    font: inherit;
    font-size: 13px;
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .name:hover {
    background: var(--bg-hover);
  }

  .link {
    flex: none;
    max-width: 60%;
    overflow: hidden;
    font-weight: 500;
    text-overflow: ellipsis;
  }

  .excerpt {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--text-secondary);
    font-size: 12px;
    text-overflow: ellipsis;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (prefers-reduced-motion: reduce) {
    .chevron {
      transition: none;
    }
  }
</style>
