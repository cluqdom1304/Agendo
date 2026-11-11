<script lang="ts">
  import WindowControls from "./WindowControls.svelte";
</script>

<!-- Only the caption buttons live here. The page toolbar and the head of the sidebar are the
     drag regions, so the window reads as a single bar (D-39). -->
<header class="titlebar">
  <WindowControls />
</header>

<style>
  .titlebar {
    position: absolute;
    top: 0;
    right: 0;
    /* Above the dialog backdrops: the window can always be minimised or closed. */
    z-index: 30;
    height: 32px;
  }
</style>
