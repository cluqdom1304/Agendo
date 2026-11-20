<script lang="ts">
  import { minutesToTime, slotAt } from "./grid";
  import type { TagDrag } from "../../stores/tagDrag.svelte";

  // The time under the pointer while a task of the inbox is dragged over the calendar (F24),
  // in steps of 15 minutes: the time the task will take if it is dropped there.
  interface Props {
    drag: TagDrag;
  }

  let { drag }: Props = $props();

  const slot = $derived(
    drag.active ? slotAt(drag.active.x, drag.active.y) : null,
  );
</script>

{#if drag.active && slot}
  <div
    class="tip"
    data-slot-tip
    aria-hidden="true"
    style:transform="translate3d({drag.active.x + 14}px, {drag.active.y - 34}px,
    0)"
  >
    {minutesToTime(slot.minutes)}
  </div>
{/if}

<style>
  .tip {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 101;
    padding: 2px 8px;
    border-radius: var(--radius-chip);
    background: var(--accent);
    color: var(--on-accent);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    pointer-events: none;
  }
</style>
