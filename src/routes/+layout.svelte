<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import TitleBar from "$lib/shell/TitleBar.svelte";
  import { flushAllSavers } from "$lib/features/notes/noteSaver";
  import { muteSpellcheckOnBlur } from "$lib/spellcheck";
  import { markFirstPaint } from "$lib/startup";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { windowEffect } from "$lib/stores/windowEffect.svelte";

  let { children } = $props();

  onMount(() => {
    void settingsStore.load();
    markFirstPaint();
    const stopTheme = settingsStore.watchSystemTheme();
    const stopSpellcheck = muteSpellcheckOnBlur(document);
    // Alt+F4 or the taskbar close give no warning: leaving the window saves what is pending.
    const flush = () => void flushAllSavers();
    window.addEventListener("blur", flush);
    window.addEventListener("pagehide", flush);
    return () => {
      window.removeEventListener("blur", flush);
      window.removeEventListener("pagehide", flush);
      stopTheme();
      stopSpellcheck();
    };
  });
</script>

<div class="app-root" class:native-effect={windowEffect.isNative}>
  <TitleBar />
  {@render children()}
</div>
