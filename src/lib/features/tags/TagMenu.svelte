<script lang="ts">
  import { t } from "../../i18n";
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

  // The menu stays open so several tags can be ticked one after another.
  function toggle(tagId: string) {
    onChange(
      values.includes(tagId)
        ? values.filter((id) => id !== tagId)
        : [...values, tagId],
    );
  }
</script>

<svelte:window onpointerdown={onPointerDown} onkeydown={onKeydown} />

<div bind:this={menu} class="menu" role="menu" aria-label={t.tags.menu}>
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
    {#if values.length === 0}<span class="check" aria-hidden="true">✓</span
      >{/if}
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
      {#if values.includes(tag.id)}<span class="check" aria-hidden="true"
          >✓</span
        >{/if}
    </button>
  {/each}
  {#if tagsStore.tags.length === 0}
    <p class="hint">{t.tags.createHint}</p>
  {/if}
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

  .item:hover {
    background: var(--bg-control-hover);
  }

  .item:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
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
    color: var(--accent);
    font-weight: 600;
  }

  .hint {
    margin: 0;
    padding: 4px var(--space-2);
    color: var(--text-secondary);
    font-size: 11px;
  }
</style>
