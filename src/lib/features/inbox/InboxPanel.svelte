<script lang="ts">
  import { t } from "../../i18n";
  import { taskFieldsOf } from "../../naturalInput";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import { BLOCK_MINUTES, minutesToTime, slotAt } from "../calendar/grid";
  import { slotDrag } from "../../stores/tagDrag.svelte";
  import TaskLine from "../todo/TaskLine.svelte";
  import { CaptureField, captureOptions } from "../capture/capture.svelte";
  import CapturePreview from "../capture/CapturePreview.svelte";

  // «Bandeja de entrada» (F20-04): the pending tasks without a date, beside Hoy and the
  // calendar, with a field to drop new ones in. Loaded with `import()` (D-98).
  interface Props {
    store: TasksStore;
    onOpenTask: (taskId: string) => void;
  }

  let { store, onOpenTask }: Props = $props();
  const l = t.inbox;

  const field = new CaptureField(() => captureOptions(store.today()));
  let adding = $state(false);

  /** Pending undated tasks in the order of the list's POR HACER (tag filter included). */
  const tasks = $derived(store.todoBlock.open);

  async function add(event: SubmitEvent) {
    event.preventDefault();
    const title = field.text.trim();
    if (!title || adding) return;
    adding = true;
    const capture = field.capture;
    const created = await store.create(
      capture ? { title: capture.title, ...taskFieldsOf(capture) } : { title },
    );
    adding = false;
    if (created) field.reset();
  }

  // A press on a row becomes a drag after a few pixels; dropped on a column of the calendar it
  // sets day, start and length (F24-01).
  function drag(event: PointerEvent, id: string, title: string) {
    slotDrag.begin(event, { id, title }, (_key, point) => {
      const slot = slotAt(point.x, point.y);
      if (slot) {
        void store.schedule(
          id,
          slot.date,
          minutesToTime(slot.minutes),
          BLOCK_MINUTES,
        );
      }
    });
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && field.ignore()) {
      event.preventDefault();
      event.stopPropagation();
    }
  }
</script>

<aside class="inbox" aria-labelledby="inbox-title" data-menu-area="agenda">
  <header class="head">
    <h2 id="inbox-title">{l.title}</h2>
    <span class="count">{tasks.length}</span>
  </header>

  <form class="add" onsubmit={add}>
    <span class="ring" aria-hidden="true"></span>
    <input
      type="text"
      aria-label={l.addLabel}
      placeholder={l.addPlaceholder}
      value={field.text}
      oninput={(event) => field.set(event.currentTarget.value)}
      {onkeydown}
      disabled={adding}
    />
    <CapturePreview capture={field.capture} today={field.today()} />
  </form>

  {#if tasks.length === 0}
    <div class="none">
      <strong>{l.empty}</strong>
      <span>{l.emptyHint}</span>
    </div>
  {:else}
    <ul class="list">
      {#each tasks as task (task.id)}
        <li
          class="row"
          data-task-id={task.id}
          onpointerdown={(event) => drag(event, task.id, task.title)}
        >
          <TaskLine
            title={task.title}
            priority={task.priority}
            checklistDone={task.checklistDone}
            checklistTotal={task.checklistTotal}
            tagIds={task.tagIds}
            checkLabel={t.tasks.markDone(task.title || t.reminders.untitled) +
              t.priority.suffix(task.priority)}
            onToggle={() => void store.setCompleted(task.id, true)}
            onOpen={() => onOpenTask(task.id)}
          />
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .inbox {
    display: flex;
    flex: none;
    flex-direction: column;
    width: 290px;
    min-height: 0;
    margin-left: var(--panel-gap);
    overflow: hidden;
    border-radius: var(--radius-panel);
    background: var(--bg-panel);
    box-shadow:
      0 0 0 1px var(--panel-border),
      var(--shadow-raised);
    animation: slide-in-right var(--motion-spring) var(--ease-spring);
  }

  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    padding: 18px 18px 10px;
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--text-title);
    font-weight: 700;
    letter-spacing: -0.015em;
  }

  .count {
    padding: 1px 8px;
    border-radius: var(--radius-chip);
    background: var(--bg-control);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  /* «Nueva tarea» reads as one more row: an empty ring and a field with no box. */
  .add {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin: 0 8px 6px;
    padding: 4px 12px;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .add:focus-within {
    background: var(--bg-control);
  }

  .ring {
    flex: none;
    box-sizing: border-box;
    width: 18px;
    height: 18px;
    border: 1.5px dashed
      color-mix(in srgb, var(--text-secondary) 70%, transparent);
    border-radius: var(--radius-chip);
  }

  .add input {
    flex: 1;
    box-sizing: border-box;
    min-width: 0;
    height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-body);
    outline: none;
  }

  .add input::placeholder {
    color: var(--text-secondary);
  }

  .add :global([data-capture-preview]) {
    flex-basis: 100%;
  }

  .list {
    display: grid;
    align-content: start;
    gap: 1px;
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0 8px 12px;
    overflow: auto;
    list-style: none;
  }

  .row {
    display: flex;
    padding: 10px 12px;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .row:hover {
    background: var(--bg-hover);
  }

  .none {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-3) 18px;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .none strong {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
  }
</style>
