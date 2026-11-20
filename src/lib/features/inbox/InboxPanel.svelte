<script lang="ts">
  import { t } from "../../i18n";
  import { taskFieldsOf } from "../../naturalInput";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import { BLOCK_MINUTES, minutesToTime, slotAt } from "../calendar/grid";
  import { slotDrag } from "../../stores/tagDrag.svelte";
  import Checkbox from "../todo/Checkbox.svelte";
  import ChecklistProgress from "../todo/ChecklistProgress.svelte";
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
          <Checkbox
            checked={false}
            label={t.tasks.markDone(task.title || t.reminders.untitled) +
              t.priority.suffix(task.priority)}
            priority={task.priority}
            onToggle={() => void store.setCompleted(task.id, true)}
          />
          <button
            type="button"
            class="open"
            onclick={() => onOpenTask(task.id)}
          >
            <span class="title">{task.title || t.reminders.untitled}</span>
            {#if task.checklistTotal > 0}
              <ChecklistProgress
                done={task.checklistDone}
                total={task.checklistTotal}
              />
            {/if}
          </button>
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
    width: 280px;
    min-height: 0;
    margin-left: var(--panel-gap);
    overflow: hidden;
    border-radius: var(--radius-panel);
    background: var(--bg-panel);
    box-shadow:
      0 0 0 1px var(--panel-border),
      var(--shadow-raised);
    animation: rise-in var(--motion-base) var(--ease-out);
  }

  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-4) var(--space-2);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 700;
  }

  .count {
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .add {
    padding: 0 var(--space-3) var(--space-2);
  }

  .add input {
    box-sizing: border-box;
    width: 100%;
    height: 30px;
    padding: 0 var(--space-3);
    border: 0;
    border-radius: var(--radius-control);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
  }

  .add input::placeholder {
    color: var(--text-secondary);
  }

  .add input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .list {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0 var(--space-2) var(--space-3);
    overflow: auto;
    list-style: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 2px;
    padding-left: var(--space-2);
    border-radius: var(--radius-control);
    transition: background-color var(--motion-instant) linear;
  }

  .row:hover {
    background: var(--bg-hover);
  }

  .open {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    min-height: 34px;
    padding: 4px var(--space-2);
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    border-radius: var(--radius-control);
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .none {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-3) var(--space-4);
    color: var(--text-secondary);
    font-size: 12px;
  }

  .none strong {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
  }
</style>
