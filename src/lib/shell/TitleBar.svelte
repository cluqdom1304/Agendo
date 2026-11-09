<script lang="ts">
  import { t } from "../i18n";
  import WindowControls from "./WindowControls.svelte";
</script>

<header
  class="flex h-9 shrink-0 items-center justify-between select-none"
  data-tauri-drag-region
>
  <span class="title pl-3 text-xs" data-tauri-drag-region>{t.app.name}</span>
  <WindowControls />
</header>

<style>
  .title {
    color: var(--text-secondary);
  }
</style>
