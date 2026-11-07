<script lang="ts">
  import { onMount } from "svelte";

  import { onReminderFired } from "$lib/api";
  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";

  onMount(() => {
    void tasksStore.load();
    const unlisten = onReminderFired((fired) =>
      tasksStore.onReminderFired(fired.taskId),
    );
    return () => {
      void unlisten.then((stop) => stop());
      tasksStore.dispose();
    };
  });
</script>

<TodoView store={tasksStore} />
