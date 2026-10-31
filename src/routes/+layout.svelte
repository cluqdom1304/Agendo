<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import TitleBar from "$lib/shell/TitleBar.svelte";
  import { markFirstPaint } from "$lib/startup";
  import { windowEffect } from "$lib/stores/windowEffect.svelte";

  let { children } = $props();

  onMount(() => {
    void windowEffect.request("mica");
    markFirstPaint();
  });
</script>

<div class="app-root flex flex-col" class:native-effect={windowEffect.isNative}>
  <TitleBar />
  <div class="min-h-0 flex-1 overflow-auto">
    {@render children()}
  </div>
</div>
