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
      <ul>
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

  h2 {
    margin: 0;
    padding-right: var(--space-8);
    font-family: var(--font-display);
    font-size: 18px;
  }

  .hint {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  input {
    height: 32px;
    padding: 0 var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 1px;
    max-height: 320px;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .choice {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    width: 100%;
    padding: 6px var(--space-3);
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .choice:hover:not(:disabled) {
    background: var(--bg-control-hover);
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
    font-size: 11px;
  }
</style>
