<script lang="ts">
  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import type { FocusStore } from "../../stores/focus.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";

  // Moves the running focus session to another task (F25-05): the time spent so far stays on the
  // current one and the rest of the session goes on with the chosen one.
  interface Props {
    focus: FocusStore;
    store: TasksStore;
    onClose: () => void;
  }

  let { focus, store, onClose }: Props = $props();
  const l = t.focus;

  let query = $state("");
  const MAX = 50;

  const matches = $derived.by(() => {
    const text = query.trim().toLocaleLowerCase();
    return store.tasks
      .filter(
        (task) =>
          !task.isCompleted &&
          (text === "" || task.title.toLocaleLowerCase().includes(text)),
      )
      .slice(0, MAX);
  });

  async function choose(taskId: string) {
    await focus.switchTo(taskId);
    onClose();
  }
</script>

<Modal label={l.switchTitle} closeLabel={t.detail.close} {onClose}>
  <div class="switch">
    <h2>{l.switchTitle}</h2>
    <p class="hint">{l.switchHint}</p>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="search"
      autofocus
      placeholder={l.switchSearch}
      aria-label={l.switchSearch}
      bind:value={query}
    />
    {#if matches.length === 0}
      <p class="hint">{l.switchNone}</p>
    {:else}
      <ul class="choice-list">
        {#each matches as task (task.id)}
          <li>
            <button
              type="button"
              class="choice"
              disabled={task.id === focus.current?.taskId}
              onclick={() => void choose(task.id)}
            >
              <span class="name">{task.title || t.reminders.untitled}</span>
              {#if task.id === focus.current?.taskId}
                <span class="tag">{l.current}</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</Modal>

<style>
  .switch {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
  }

  .hint {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .choice-list {
    max-height: 320px;
  }

  .choice {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    width: 100%;
    min-height: 40px;
    padding: 0 14px;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 14px;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .choice:hover:not(:disabled) {
    background: var(--bg-card-hover);
  }

  /* The list clips its rounded corners: the ring is drawn inside the row so it shows whole. */
  .choice:focus-visible {
    outline-offset: -2px;
  }

  .choice:disabled {
    color: var(--text-secondary);
    cursor: default;
  }

  .name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag {
    flex: none;
    padding: 1px 9px;
    border-radius: var(--radius-chip);
    background: var(--bg-control-hover);
    font-size: 11.5px;
    font-weight: 600;
  }
</style>
