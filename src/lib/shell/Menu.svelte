<script lang="ts" module>
  export interface MenuItem {
    id: string;
    label: string;
    /** A choice among several shows a tick when it is the current one; leave it out for actions. */
    checked?: boolean;
    danger?: boolean;
    /** Indentation level, for trees. */
    depth?: number;
    /** Draws a hairline above the item. */
    separated?: boolean;
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";

  // A small popover menu. It is drawn under the element that holds it, which must be
  // positioned; the button that opens it carries `data-menu-toggle` and owns the open state.
  interface Props {
    label: string;
    items: MenuItem[];
    /** The side of the holder the menu lines up with. */
    align?: "left" | "right";
    onSelect: (id: string) => void;
    onClose: () => void;
  }

  let { label, items, align = "right", onSelect, onClose }: Props = $props();

  let menu = $state<HTMLElement>();

  const buttons = () =>
    Array.from(menu?.querySelectorAll<HTMLButtonElement>("button") ?? []);

  // The keyboard lands on the current choice, and goes back to the opener afterwards.
  onMount(() => {
    const opener = document.activeElement;
    const all = buttons();
    (
      all.find((item) => item.getAttribute("aria-checked") === "true") ?? all[0]
    )?.focus();
    return () => {
      const active = document.activeElement;
      if (
        opener instanceof HTMLElement &&
        opener.isConnected &&
        (!active || active === document.body || menu?.contains(active))
      ) {
        opener.focus();
      }
    };
  });

  function onPointerDown(event: PointerEvent) {
    const target = event.target;
    if (!(target instanceof Element) || menu?.contains(target)) return;
    // The opener closes the menu itself; closing here too would reopen it at once.
    if (target.closest("[data-menu-toggle]")) return;
    onClose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key === "Tab") {
      onClose();
      return;
    }
    const all = buttons();
    if (all.length === 0) return;
    const at = all.indexOf(document.activeElement as HTMLButtonElement);
    let next: number;
    if (event.key === "ArrowDown") next = (at + 1) % all.length;
    else if (event.key === "ArrowUp") next = (at - 1 + all.length) % all.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = all.length - 1;
    else return;
    event.preventDefault();
    event.stopPropagation();
    all[next]?.focus();
  }

  function choose(id: string) {
    onClose();
    onSelect(id);
  }
</script>

<svelte:window onpointerdown={onPointerDown} onkeydowncapture={onKeydown} />

<div
  bind:this={menu}
  class="menu"
  class:left={align === "left"}
  role="menu"
  aria-label={label}
>
  {#each items as item (item.id)}
    <button
      type="button"
      class="item"
      class:danger={item.danger}
      class:separated={item.separated}
      role={item.checked === undefined ? "menuitem" : "menuitemradio"}
      aria-checked={item.checked}
      style:padding-left="{8 + (item.depth ?? 0) * 12}px"
      onclick={() => choose(item.id)}
    >
      <span class="name">{item.label}</span>
      {#if item.checked}
        <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
          <path
            d="M2.5 6.4l2.3 2.3 4.7-5.2"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      {/if}
    </button>
  {/each}
</div>

<style>
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 20;
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 184px;
    max-width: 260px;
    max-height: 300px;
    padding: 5px;
    overflow-y: auto;
    border-radius: var(--radius-card);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    transform-origin: top right;
    animation: pop-in var(--motion-fast) var(--ease-out);
  }

  .menu.left {
    right: auto;
    left: 0;
    transform-origin: top left;
  }

  .item {
    display: flex;
    flex: none;
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

  /* Solid accent, as in a system menu and in the context menu (D-50). */
  .item:hover,
  .item:focus-visible {
    background: var(--accent);
    color: var(--on-accent);
    outline: none;
  }

  .item.danger {
    color: var(--danger);
  }

  .item.danger:hover,
  .item.danger:focus-visible {
    background: var(--danger);
    color: var(--on-danger);
  }

  /* The hairline sits in the gap above the item, clear of its hover background. */
  .item.separated {
    position: relative;
    margin-top: 9px;
  }

  .item.separated::before {
    content: "";
    position: absolute;
    top: -5px;
    right: 4px;
    left: 4px;
    height: 1px;
    background: var(--separator);
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  svg {
    flex: none;
    color: var(--accent);
  }

  .item:hover svg,
  .item:focus-visible svg {
    color: inherit;
  }
</style>
