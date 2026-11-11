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
    style:transform="translate3d({drag.active.x + 14}px, {drag.active.y + 14}px,
    0)"
  >
    <span class="title">{drag.active.title || t.calendar.untitled}</span>
    {#if drag.active.overKey === null}
      <span class="hint">{t.tags.dropHint}</span>
    {/if}
  </div>
{/if}

<style>
  /* Follows the pointer through transform alone: no layout while dragging. */
  .ghost {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-width: 220px;
    padding: 5px 10px;
    border-radius: var(--radius-control);
    background: var(--bg-popover);
    box-shadow:
      inset 0 0 0 1px var(--accent),
      var(--shadow-popover);
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
