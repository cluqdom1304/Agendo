<script lang="ts">
  import { onMount } from "svelte";

  import { searchAll } from "../../api";
  import { notesStore } from "../../stores/notes.svelte";
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
    onClose: () => void;
  }

  let { mode, onTask, onClose }: Props = $props();

  const commands = buildCommands({
    view: viewStore,
    notes: notesStore,
    tasks: tasksStore,
    tags: tagsStore,
  });

  // The recent notes are known by id: their titles are looked up when the palette opens.
  onMount(() => {
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
  onNote={(id) => void showNote(id)}
  {onTask}
  onTag={openTag}
  {onClose}
/>
