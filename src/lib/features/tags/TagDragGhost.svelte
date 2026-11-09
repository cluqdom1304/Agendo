<script lang="ts">
  import { t } from "../../i18n";
  import type { TagDrag } from "../../stores/tagDrag.svelte";

  interface Props {
    drag: TagDrag;
  }

  let { drag }: Props = $props();
</script>

{#if drag.active}
  <div
    class="ghost"
    aria-hidden="true"
    style:left="{drag.active.x + 14}px"
    style:top="{drag.active.y + 14}px"
  >
    <span class="title">{drag.active.title || t.calendar.untitled}</span>
    {#if drag.active.overKey === null}
      <span class="hint">{t.tags.dropHint}</span>
    {/if}
  </div>
{/if}

<style>
  .ghost {
    position: fixed;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-width: 220px;
    padding: 4px 8px;
    border: 1px solid var(--accent);
    border-radius: var(--radius-control);
    background: var(--bg-window-solid);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.3);
    font-size: 12px;
    pointer-events: none;
  }

  .title {
    overflow: hidden;
    color: var(--text-primary);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hint {
    color: var(--text-secondary);
    font-size: 10px;
  }
</style>
