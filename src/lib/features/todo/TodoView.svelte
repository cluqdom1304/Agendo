<script lang="ts">
  import { tick } from "svelte";

  import { t } from "../../i18n";
  import { findShortcut, focusContextOf, ROW_ATTRIBUTE } from "../../keymap";
  import EmptyState from "../../shell/EmptyState.svelte";
  import Toast from "../../shell/Toast.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import { NO_FILTER } from "../../stores/listFilter";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import DayGroup from "./DayGroup.svelte";
  import ListFilters from "./ListFilters.svelte";
  import SectionHeader from "./SectionHeader.svelte";
  import SelectionBar from "./SelectionBar.svelte";

  interface Props {
    store: TasksStore;
  }

  let { store }: Props = $props();

  const lastDeletion = $derived(store.undoStack.at(-1));

  // A date filter that only matches dated tasks leaves POR HACER out, and «Sin fecha» the days.
  const showTodo = $derived(
    store.listFilter.date === "all" || store.listFilter.date === "undated",
  );
  const showAgenda = $derived(store.listFilter.date !== "undated");
  const nothingMatches = $derived(
    store.listFiltering &&
      !store.draft &&
      store.listTodoBlock.open.length + store.listTodoBlock.completed.length ===
        0 &&
      store.listAgendaBlocks.length === 0,
  );

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

  function onKeydown(event: KeyboardEvent) {
    if (
      event.key === "Escape" &&
      store.batch.count > 0 &&
      focusContextOf(event.target) !== "title"
    ) {
      store.batch.clear();
      event.preventDefault();
      return;
    }
    const shortcut = findShortcut(event, focusContextOf(event.target));
    if (!shortcut) return;
    const row = rowOf(event.target);
    const id = row?.dataset.taskId;

    switch (shortcut) {
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
        if (store.batch.entry) void store.batch.undo();
        else if (lastDeletion) void store.undoDelete();
        else return;
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
    <EmptyState variant="loading" title={t.tasks.loading} />
  {:else if store.status === "error"}
    <EmptyState
      variant="error"
      title={t.tasks.loadFailed}
      actionLabel={t.tasks.retry}
      onAction={() => void store.load()}
    />
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
    <header class="view-head head">
      <div class="view-heading">
        <h2 class="view-title">{t.sidebar.tasks}</h2>
        <p class="view-sub">
          {t.tasks.pendingCount(store.listTodoBlock.open.length)}
        </p>
      </div>
    </header>
    <ListFilters
      filter={store.listFilter}
      onChange={(filter) => (store.listFilter = filter)}
    />
    {#if nothingMatches}
      <EmptyState
        title={t.listFilter.noneTitle}
        description={t.listFilter.noneDescription}
        actionLabel={t.listFilter.clear}
        onAction={() => (store.listFilter = NO_FILTER)}
      />
    {:else}
      {#if showTodo}
        <SectionHeader
          title={t.tasks.sectionTodo}
          detail={t.tasks.pendingCount(store.listTodoBlock.open.length)}
        />
        <DayGroup block={store.listTodoBlock} {store} />
      {/if}

      {#if showAgenda}
        <SectionHeader title={t.tasks.sectionAgenda} />
        {#each store.listAgendaBlocks as block (block.date)}
          <DayGroup {block} {store} />
        {/each}
      {/if}
    {/if}
  {/if}
</div>

<SelectionBar {store} />

{#if store.batch.entry}
  {@const entry = store.batch.entry}
  <Toast
    message={entry.message}
    actionLabel={t.tasks.undo}
    closeLabel={t.tasks.dismiss}
    onAction={() => void store.batch.undo()}
    onClose={() => store.batch.dismiss()}
  />
{:else if lastDeletion}
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
    padding: var(--space-3) var(--space-6) 96px;
  }

  .head {
    padding: 0 var(--space-3) var(--space-4);
  }
</style>
