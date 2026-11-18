<script lang="ts">
  import { t } from "../../i18n";
  import { dropdownPosition } from "../context/placement";
  import { tagsStore } from "../../stores/tags.svelte";

  interface Props {
    values: string[];
    onChange: (tagIds: string[]) => void;
    onClose: () => void;
  }

  let { values, onChange, onClose }: Props = $props();

  let menu = $state<HTMLElement>();

  // Clicking elsewhere closes the menu, except the button that toggles it (it would close
  // and immediately reopen).
  function onPointerDown(event: PointerEvent) {
    const target = event.target;
    if (!(target instanceof Element) || menu?.contains(target)) return;
    if (target.closest("[data-tag-toggle]")) return;
    onClose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.stopPropagation();
    onClose();
  }

  // Fixed to the window and moved under <body>: inside the row it was still cut by the scrolling,
  // animated ancestors of the list (a fixed box is clipped or trapped by any of them), and the
  // sidebar painted over its left edge.
  function place(node: HTMLElement) {
    const anchor = node.parentElement?.getBoundingClientRect();
    if (!anchor) return;
    document.body.appendChild(node);
    const { left, top } = dropdownPosition(
      anchor,
      { width: node.offsetWidth, height: node.offsetHeight },
      { width: window.innerWidth, height: window.innerHeight },
    );
    node.style.left = `${left}px`;
    node.style.top = `${top}px`;
    // A fixed menu does not follow its row when the list scrolls.
    const onScroll = (event: Event) => {
      if (event.target instanceof Node && node.contains(event.target)) return;
      onClose();
    };
    window.addEventListener("scroll", onScroll, true);
    return {
      destroy: () => {
        window.removeEventListener("scroll", onScroll, true);
        node.remove();
      },
    };
  }

  // The menu stays open so several tags can be ticked one after another.
  function toggle(tagId: string) {
    onChange(
      values.includes(tagId)
        ? values.filter((id) => id !== tagId)
        : [...values, tagId],
    );
  }
</script>

<svelte:window
  onpointerdown={onPointerDown}
  onkeydown={onKeydown}
  onresize={onClose}
/>

<div
  bind:this={menu}
  use:place
  class="menu"
  role="menu"
  aria-label={t.tags.menu}
>
  <button
    type="button"
    role="menuitem"
    class="item"
    onclick={() => {
      onChange([]);
      onClose();
    }}
  >
    <span class="dot none"></span>
    <span class="name">{t.tags.noneOption}</span>
    {#if values.length === 0}{@render tick()}{/if}
  </button>
  {#each tagsStore.tags as tag (tag.id)}
    <button
      type="button"
      role="menuitemcheckbox"
      aria-checked={values.includes(tag.id)}
      class="item"
      onclick={() => toggle(tag.id)}
    >
      <span class="dot" style:background={tag.colorHex}></span>
      <span class="name">{tag.name}</span>
      {#if values.includes(tag.id)}{@render tick()}{/if}
    </button>
  {/each}
  {#if tagsStore.tags.length === 0}
    <p class="hint">{t.tags.createHint}</p>
  {/if}
</div>

{#snippet tick()}
  <svg
    class="check"
    viewBox="0 0 12 12"
    width="11"
    height="11"
    aria-hidden="true"
  >
    <path
      d="M2.5 6.4l2.3 2.3 4.7-5.2"
      fill="none"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
{/snippet}

<style>
  .menu {
    position: fixed;
    z-index: 40;
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 168px;
    max-width: 240px;
    max-height: 260px;
    padding: 5px;
    overflow-y: auto;
    border-radius: var(--radius-card);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    transform-origin: top right;
    animation: pop-in var(--motion-fast) var(--ease-out);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 26px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .item:hover,
  .item:focus-visible {
    background: var(--accent);
    color: var(--on-accent);
    outline: none;
  }

  /* On the accent the dot keeps its edge. */
  .item:hover .dot,
  .item:focus-visible .dot {
    box-shadow: 0 0 0 1px var(--on-accent);
  }

  .item:hover .dot.none,
  .item:focus-visible .dot.none {
    border-color: var(--on-accent);
    box-shadow: none;
  }

  .dot {
    flex: none;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-chip);
  }

  .dot.none {
    border: 1.5px solid var(--text-secondary);
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .check {
    flex: none;
    color: var(--accent);
  }

  .item:hover .check,
  .item:focus-visible .check {
    color: inherit;
  }

  .hint {
    margin: 0;
    padding: 4px var(--space-2);
    color: var(--text-secondary);
    font-size: 11px;
  }
</style>
