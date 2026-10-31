<script lang="ts">
  import "../app.css";
  import TitleBar from "$lib/shell/TitleBar.svelte";

  let { children } = $props();
</script>

<div class="flex h-screen flex-col overflow-hidden">
  <TitleBar />
  <div class="min-h-0 flex-1 overflow-auto">
    {@render children()}
  </div>
</div>
