<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import type { CalendarEvent } from "../../types";
  import EventForm from "./EventForm.svelte";

  // The event dialog of the page: a new event, or the event with `eventId`, which is read first
  // because an occurrence does not carry the repeat rule or the reminders.
  interface Props {
    store: EventsStore;
    eventId?: string;
    defaultDate: string;
    defaultTime?: string | null;
    onClose: () => void;
    onConvert?: (event: CalendarEvent) => void;
  }

  let {
    store,
    eventId,
    defaultDate,
    defaultTime = null,
    onClose,
    onConvert,
  }: Props = $props();

  let loaded = $state.raw<CalendarEvent | null>(null);
  // svelte-ignore state_referenced_locally
  let loading = $state(eventId !== undefined);

  onMount(() => {
    if (eventId === undefined) return;
    void store.get(eventId).then((event) => {
      loaded = event;
      loading = false;
      // The event is gone: the store already told why.
      if (!event) onClose();
    });
  });
</script>

{#if loading}
  <Modal label={t.events.dialogEdit} closeLabel={t.events.close} {onClose}>
    <p class="loading" role="status">{t.events.loading}</p>
  </Modal>
{:else if eventId === undefined || loaded}
  <EventForm
    {store}
    event={loaded ?? undefined}
    {defaultDate}
    {defaultTime}
    {onClose}
    {onConvert}
  />
{/if}

<style>
  .loading {
    margin: 0;
    color: var(--text-secondary);
  }
</style>
