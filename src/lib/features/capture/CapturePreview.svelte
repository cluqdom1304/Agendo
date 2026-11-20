<script lang="ts">
  import { t } from "../../i18n";
  import type { Capture } from "../../naturalInput";
  import { captureChips } from "./capture.svelte";

  // What the text being typed will create with (F23-02): shown under the field, gone after Esc.
  interface Props {
    capture: Capture | null;
    today: string;
    /** Laid over what follows (rows, calendar slots) instead of pushing it down. */
    floating?: boolean;
  }

  let { capture, today, floating = false }: Props = $props();

  const chips = $derived(capture ? captureChips(capture, today) : []);
</script>

<div class="capture" class:floating aria-live="polite" data-capture-preview>
  {#if capture}
    <span class="sr">{t.capture.label}</span>
    <span class="title">{capture.title}</span>
    {#each chips as chip, index (index)}
      <span class="chip">{chip}</span>
    {/each}
    <kbd>{t.capture.ignore}</kbd>
  {/if}
</div>

<style>
  .capture:empty {
    display: none;
  }

  .capture {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    margin-top: 4px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .capture.floating {
    position: absolute;
    top: calc(100% - 2px);
    left: 0;
    z-index: 5;
    max-width: 100%;
    margin: 0;
    padding: 4px 6px;
    border-radius: var(--radius-small);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
  }

  .title {
    overflow: hidden;
    max-width: 16em;
    color: var(--text-primary);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip {
    padding: 1px 6px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    color: var(--text-primary);
    white-space: nowrap;
  }

  kbd {
    margin-left: auto;
    padding-left: 6px;
    font: inherit;
  }

  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
