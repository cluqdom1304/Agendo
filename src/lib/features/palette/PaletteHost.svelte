<script lang="ts">
  import { onMount } from "svelte";

  import { searchAll } from "../../api";
  import { notesStore } from "../../stores/notes.svelte";
  import { savedSearches } from "../../stores/savedSearches.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import { tasksStore } from "../../stores/tasks.svelte";
  import { viewStore } from "../../stores/view.svelte";
  import { showNote } from "../notes/navigation";
  import { buildCommands } from "./commands";
  import Palette from "./Palette.svelte";
  import type { PaletteMode } from "./palette";

  // Puts the palette on the real stores: where each kind of result leads, and the commands.
  interface Props {
    mode: PaletteMode;
    /** Shows the task in the agenda (the page knows how to reveal it). */
    onTask: (taskId: string) => void;
    /** Opens the creation dialog. */
    onNewTask?: () => void;
    /** Opens the event dialog. */
    onNewEvent?: () => void;
    /** Opens the habit dialog. */
    onNewHabit?: () => void;
    onClose: () => void;
  }

  let { mode, onTask, onNewTask, onNewEvent, onNewHabit, onClose }: Props =
    $props();

  const commands = buildCommands({
    view: viewStore,
    notes: notesStore,
    tasks: tasksStore,
    tags: tagsStore,
    newTask: () => onNewTask?.(),
    newEvent: () => onNewEvent?.(),
    newHabit: () => onNewHabit?.(),
  });

  // The recent notes are known by id: their titles are looked up when the palette opens.
  onMount(() => {
    void savedSearches.load();
    void notesStore
      .ensureLoaded()
      .then(() => notesStore.ensureTitles(notesStore.recent));
  });

  function openTag(id: string) {
    if (viewStore.section === "notes") {
      void notesStore.selectScope(`tag:${id}`);
    } else {
      tagsStore.only(id);
    }
  }
</script>

<Palette
  {mode}
  search={(query) => searchAll(query)}
  {commands}
  recent={notesStore.recentNotes()}
  views={savedSearches.items}
  onSaveView={(query) => void savedSearches.add(query)}
  onDeleteView={(query) => void savedSearches.remove(query)}
  onNote={(id) => void showNote(id)}
  {onTask}
  onTag={openTag}
  {onClose}
/>
