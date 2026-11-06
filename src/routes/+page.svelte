<script lang="ts">
  import { onMount } from "svelte";

  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";

  onMount(() => {
    void tasksStore.load();
    return () => tasksStore.dispose();
  });
</script>

<TodoView store={tasksStore} />
