<script lang="ts">
  import { tick } from "svelte";

  import { t } from "../../i18n";
  import { findShortcut, focusContextOf, ROW_ATTRIBUTE } from "../../keymap";
  import Spotlight from "../../shell/Spotlight.svelte";
  import EmptyState from "../../shell/EmptyState.svelte";
  import Toast from "../../shell/Toast.svelte";
  import type { SearchHit } from "../../types";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import DayGroup from "./DayGroup.svelte";
  import SectionHeader from "./SectionHeader.svelte";

  interface Props {
    store: TasksStore;
  }

  let { store }: Props = $props();

  let searchOpen = $state(false);

  const lastDeletion = $derived(store.undoStack.at(-1));

  function rows(): HTMLElement[] {
    return [...document.querySelectorAll<HTMLElement>(`[${ROW_ATTRIBUTE}]`)];
  }

  function rowOf(target: EventTarget | null): HTMLElement | null {
    return target instanceof Element
      ? target.closest<HTMLElement>(`[${ROW_ATTRIBUTE}]`)
      : null;
  }

  async function focusRow(id: string) {
    await tick();
    document
      .querySelector<HTMLElement>(`[${ROW_ATTRIBUTE}][data-task-id="${id}"]`)
      ?.focus();
  }

  function move(from: HTMLElement | null, step: 1 | -1) {
    const all = rows();
    const index = from ? all.indexOf(from) : -1;
    const next = index < 0 ? (step === 1 ? 0 : all.length - 1) : index + step;
    all[next]?.focus();
  }

  async function revealHit(hit: SearchHit) {
    searchOpen = false;
    if (!store.reveal(hit.taskId)) return;
    await focusRow(hit.taskId);
    document
      .querySelector<HTMLElement>(
        `[${ROW_ATTRIBUTE}][data-task-id="${hit.taskId}"]`,
      )
      ?.scrollIntoView?.({ block: "center" });
  }

  function onKeydown(event: KeyboardEvent) {
    const shortcut = findShortcut(event, focusContextOf(event.target));
    if (!shortcut) return;
    const row = rowOf(event.target);
    const id = row?.dataset.taskId;

    switch (shortcut) {
      case "openSearch":
        searchOpen = true;
        break;
      case "newTask":
        store.startDraft(store.defaultTarget());
        break;
      case "newTaskToday":
        store.startDraft(store.todayTarget());
        break;
      case "toggleComplete": {
        const task = id ? store.find(id) : undefined;
        if (!task) return;
        void store.setCompleted(task.id, !task.isCompleted);
        break;
      }
      case "deleteTask": {
        if (!id) return;
        const all = rows();
        const index = row ? all.indexOf(row) : -1;
        const neighbourId = (all[index + 1] ?? all[index - 1])?.dataset.taskId;
        void store.remove(id);
        if (neighbourId) void focusRow(neighbourId);
        break;
      }
      case "undo":
        if (!lastDeletion) return;
        void store.undoDelete();
        break;
      case "selectPrevious":
        move(row, -1);
        break;
      case "selectNext":
        move(row, 1);
        break;
      default:
        return;
    }
    event.preventDefault();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="document" role="grid" aria-label={t.app.name}>
  {#if store.status === "loading" || store.status === "idle"}
    <p class="status">{t.tasks.loading}</p>
  {:else if store.allFiltered && !store.draft}
    <EmptyState
      title={t.tags.allHiddenTitle}
      description={t.tags.allHiddenDescription}
      actionLabel={t.tags.showAll}
      onAction={() => tagsStore.showAll()}
    />
  {:else if store.isEmpty && !store.draft}
    <EmptyState
      title={t.empty.title}
      description={t.empty.description}
      actionLabel={t.empty.action}
      shortcut={t.empty.shortcut}
      onAction={() => store.startDraft({ section: "todo" })}
    />
  {:else}
    <SectionHeader
      title={t.tasks.sectionTodo}
      detail={t.tasks.pendingCount(store.todoPendingCount)}
    />
    <DayGroup block={store.todoBlock} {store} />

    <SectionHeader title={t.tasks.sectionAgenda} />
    {#each store.agendaBlocks as block (block.date)}
      <DayGroup {block} {store} />
    {/each}
  {/if}
</div>

{#if searchOpen}
  <Spotlight
    search={(query) => store.search(query)}
    onSelect={revealHit}
    onClose={() => (searchOpen = false)}
  />
{/if}

{#if lastDeletion}
  <Toast
    message={t.tasks.deleted}
    actionLabel={t.tasks.undo}
    closeLabel={t.tasks.dismiss}
    onAction={() => void store.undoDelete()}
    onClose={() => (store.undoStack = [])}
  />
{:else if store.notice}
  <Toast
    kind={store.notice.kind}
    message={store.notice.message}
    closeLabel={t.tasks.dismiss}
    onClose={() => store.dismissNotice()}
  />
{/if}

<style>
  .document {
    max-width: 780px;
    margin: 0 auto;
    padding: 0 var(--space-6) 96px;
  }

  .status {
    padding: var(--space-6);
    color: var(--text-secondary);
  }
</style>
