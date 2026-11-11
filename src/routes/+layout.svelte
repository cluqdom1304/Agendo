<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import TitleBar from "$lib/shell/TitleBar.svelte";
  import { markFirstPaint } from "$lib/startup";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { windowEffect } from "$lib/stores/windowEffect.svelte";

  let { children } = $props();

  onMount(() => {
    void settingsStore.load();
    markFirstPaint();
    return settingsStore.watchSystemTheme();
  });
</script>

<div class="app-root" class:native-effect={windowEffect.isNative}>
  <TitleBar />
  {@render children()}
</div>
