<script lang="ts">
  import { t } from "../../i18n";
  import type { Reminder, ReminderInput } from "../../types";
  import ReminderEditor from "./ReminderEditor.svelte";

  interface Props {
    title: string;
    dueDate: string | null;
    dueTime: string | null;
    reminders: Reminder[];
    now?: () => Date;
    onSetTime: (time: string | null) => void;
    onChange: (reminders: ReminderInput[]) => void;
    onClose: () => void;
  }

  let {
    title,
    dueDate,
    dueTime,
    reminders,
    now = () => new Date(),
    onSetTime,
    onChange,
    onClose,
  }: Props = $props();

  let panel = $state<HTMLElement>();

  // Clicking anywhere else closes the panel, except the bell that toggles it (it would
  // close and immediately reopen). Pickers of the browser draw outside the page and never
  // reach this handler.
  function onPointerDown(event: PointerEvent) {
    const target = event.target;
    if (!(target instanceof Element) || panel?.contains(target)) return;
    if (target.closest("[data-reminders-toggle]")) return;
    onClose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.stopPropagation();
    onClose();
  }
</script>

<svelte:window onpointerdown={onPointerDown} onkeydown={onKeydown} />

<div
  bind:this={panel}
  class="popover ui-form"
  role="dialog"
  aria-label={t.reminders.open(title)}
  tabindex="-1"
>
  <h5>{t.reminders.heading}</h5>
  <ReminderEditor
    {dueDate}
    {dueTime}
    {reminders}
    {now}
    {onSetTime}
    {onChange}
  />
</div>

<style>
  .popover {
    display: grid;
    gap: var(--space-2);
    margin: var(--space-1) var(--space-3) var(--space-2) 40px;
    padding: var(--space-3);
    border-radius: var(--radius-card);
    background: var(--bg-control);
    outline: none;
    animation: rise-in var(--motion-base) var(--ease-out);
  }

  h5 {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }
</style>
