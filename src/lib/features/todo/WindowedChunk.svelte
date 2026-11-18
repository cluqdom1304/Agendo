<script lang="ts">
  import { onMount, type Snippet } from "svelte";

  interface Props {
    /** Mounted from the start (the first chunks of a list). */
    eager: boolean;
    /** Must stay mounted, e.g. it holds the selected row. */
    forced: boolean;
    /** Height to reserve until the chunk has been measured. */
    estimate: number;
    children: Snippet;
  }

  let { eager, forced, estimate, children }: Props = $props();

  // Where the chunk starts is the parent's decision; later it follows the viewport.
  // svelte-ignore state_referenced_locally
  let near = $state(eager);
  // svelte-ignore state_referenced_locally
  let height = $state(estimate);
  let host: HTMLElement | undefined;

  const mounted = $derived(near || forced);

  onMount(() => {
    if (typeof IntersectionObserver === "undefined" || !host) {
      near = true;
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        const entry = entries.at(-1);
        if (!entry) return;
        // The real height is kept when the rows leave, so the scrollbar does not jump.
        if (!entry.isIntersecting && host && host.offsetHeight > 0) {
          height = host.offsetHeight;
        }
        near = entry.isIntersecting;
      },
      { rootMargin: "1500px 0px" },
    );
    observer.observe(host);
    return () => observer.disconnect();
  });
</script>

<div bind:this={host} style:height={mounted ? undefined : `${height}px`}>
  {#if mounted}
    {@render children()}
  {/if}
</div>
