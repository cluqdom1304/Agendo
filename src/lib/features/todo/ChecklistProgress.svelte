<script lang="ts">
  import { t } from "../../i18n";

  // «2/5» next to a task whose notes hold a checklist (F20-03). The counts are stored by the
  // backend on save (D-93), so drawing them parses nothing.
  interface Props {
    done: number;
    total: number;
  }

  let { done, total }: Props = $props();

  const label = $derived(t.tasks.checklist(done, total));
</script>

<span class="checklist" class:complete={done >= total} title={label}>
  <svg viewBox="0 0 16 16" width="11" height="11" aria-hidden="true">
    <rect
      x="2.5"
      y="2.5"
      width="11"
      height="11"
      rx="3"
      fill="none"
      stroke="currentColor"
      stroke-width="1.4"
    />
    <path
      d="M5.5 8.2l1.8 1.8 3.3-3.6"
      fill="none"
      stroke="currentColor"
      stroke-width="1.4"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
  <span aria-hidden="true">{done}/{total}</span>
  <span class="spoken">{label}</span>
</span>

<style>
  .checklist {
    position: relative;
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: 3px;
    color: var(--text-secondary);
    font-size: var(--text-caption);
    font-weight: 500;
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  /* A finished list reads in ink: coral is kept for what is "now" (D-101). */
  .checklist.complete {
    color: var(--text-primary);
  }

  /* Read by screen readers instead of the bare «2/5». */
  .spoken {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
