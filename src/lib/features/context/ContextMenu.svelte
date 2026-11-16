<script lang="ts">
  import { tick } from "svelte";

  import { t } from "../../i18n";
  import { contextMenu, type ContextEntry } from "./context.svelte";
  import { buildEntries, type ContextDeps } from "./entries";
  import { clampStart, slideToFit } from "./placement";
  import { resolveTarget } from "./target";

  // The menu of the right click and of the Menu key / Shift+F10 (D-50). It is drawn by the
  // app, in the style of the other menus, and decides what to offer from the element that was
  // pointed at (see `target.ts`). The browser's own menu only remains in text fields.
  interface Props {
    deps: Omit<ContextDeps, "clipboard" | "menu">;
  }

  let { deps }: Props = $props();

  const current = $derived(contextMenu.current);
  const entries = $derived(current ? current.build() : []);
  /** The id of the open submenu at each depth. */
  let path = $state<string[]>([]);
  let opener: Element | null = null;

  // The clipboard belongs to the page: these act on the editor that had the focus.
  const clipboard = {
    cut: () => void document.execCommand("cut"),
    copy: () => void document.execCommand("copy"),
    paste: () =>
      void navigator.clipboard
        ?.readText()
        .then((text) => document.execCommand("insertText", false, text))
        .catch(() => undefined),
    selectAll: () => void document.execCommand("selectAll"),
  };

  const full = $derived<ContextDeps>({ ...deps, menu: contextMenu, clipboard });

  // What was pointed at may disappear while the menu is open (a task deleted elsewhere).
  $effect(() => {
    if (current && entries.length === 0) close();
  });

  function show(x: number, y: number, origin: Element | null) {
    const target = resolveTarget(origin);
    const build = () => buildEntries(target, full);
    if (build().length === 0) return false;
    opener = document.activeElement;
    path = [];
    contextMenu.show({ x, y, label: t.context.menu, build });
    return true;
  }

  function close() {
    if (!contextMenu.current) return;
    contextMenu.close();
    path = [];
    // The keyboard goes back to where it was, unless the action moved it somewhere else.
    const active = document.activeElement;
    if (
      opener instanceof HTMLElement &&
      opener.isConnected &&
      (!active || active === document.body || root?.contains(active))
    ) {
      opener.focus();
    }
  }

  let root = $state<HTMLElement>();

  function onContextMenu(event: MouseEvent) {
    const origin = event.target instanceof Element ? event.target : null;
    const target = resolveTarget(origin);
    // Text fields keep the browser's menu (copy, paste, spelling).
    if (target.kind === "field") {
      close();
      return;
    }
    event.preventDefault();
    if (root?.contains(origin)) return;
    if (!show(event.clientX, event.clientY, origin)) close();
  }

  function onKey(event: KeyboardEvent) {
    const opens =
      event.key === "ContextMenu" || (event.shiftKey && event.key === "F10");
    if (opens && !event.isComposing) {
      const origin = document.activeElement;
      if (!origin || resolveTarget(origin).kind === "field") return;
      event.preventDefault();
      event.stopPropagation();
      const box = origin.getBoundingClientRect();
      show(box.left + Math.min(40, box.width / 2), box.bottom - 4, origin);
      return;
    }
    if (!contextMenu.current) return;
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    if (event.key === "Tab") {
      close();
      return;
    }
    const active = document.activeElement;
    const panel = active?.closest<HTMLElement>("[role='menu']");
    if (!panel || !root?.contains(panel)) return;
    const items = Array.from(
      panel.querySelectorAll<HTMLButtonElement>(
        ":scope > .slot > button:not([disabled])",
      ),
    );
    const at = items.indexOf(active as HTMLButtonElement);
    const depth = Number(panel.dataset.depth ?? 0);
    let next: HTMLElement | undefined;
    if (event.key === "ArrowDown") next = items[(at + 1) % items.length];
    else if (event.key === "ArrowUp") {
      next = items[(at - 1 + items.length) % items.length];
    } else if (event.key === "Home") next = items[0];
    else if (event.key === "End") next = items.at(-1);
    else if (event.key === "ArrowRight" && active instanceof HTMLElement) {
      const id = active.dataset.entry;
      if (id && active.getAttribute("aria-haspopup") === "menu") {
        event.preventDefault();
        event.stopPropagation();
        void openSub(depth, id, true);
      }
      return;
    } else if (event.key === "ArrowLeft" && depth > 0) {
      event.preventDefault();
      event.stopPropagation();
      const parent =
        panel.parentElement?.querySelector<HTMLElement>(":scope > button");
      path = path.slice(0, depth - 1);
      parent?.focus();
      return;
    } else return;
    event.preventDefault();
    event.stopPropagation();
    next?.focus();
  }

  async function openSub(depth: number, id: string, focusFirst = false) {
    path = [...path.slice(0, depth), id];
    if (!focusFirst) return;
    await tick();
    root
      ?.querySelectorAll<HTMLElement>(`[data-depth="${depth + 1}"]`)
      .item(0)
      ?.querySelector<HTMLElement>(":scope > .slot > button:not([disabled])")
      ?.focus();
  }

  function pick(entry: ContextEntry, depth: number) {
    if (entry.disabled) return;
    if (entry.children) {
      void openSub(depth, entry.id);
      return;
    }
    if (entry.keepOpen) {
      entry.run?.();
      return;
    }
    close();
    entry.run?.();
  }

  function hover(entry: ContextEntry, depth: number) {
    path = entry.children
      ? [...path.slice(0, depth), entry.id]
      : path.slice(0, depth);
  }

  function onPointerDown(event: PointerEvent) {
    if (!contextMenu.current) return;
    if (event.target instanceof Node && root?.contains(event.target)) return;
    close();
  }

  // Keeps the menu inside the window, and the keyboard on its first entry.
  function place(node: HTMLElement, at: { x: number; y: number }) {
    const fit = (point: { x: number; y: number }) => {
      // `offset*` ignores the scale of the opening animation, which made the box look smaller.
      node.style.left = `${clampStart(point.x, node.offsetWidth, window.innerWidth)}px`;
      node.style.top = `${clampStart(point.y, node.offsetHeight, window.innerHeight)}px`;
    };
    fit(at);
    node
      .querySelector<HTMLElement>(
        ":scope > .panel > .slot > button:not([disabled])",
      )
      ?.focus();
    return { update: fit };
  }

  // A submenu opens to the right, or to the left when there is no room, and slides up if it
  // would fall off the bottom.
  function fitSub(node: HTMLElement) {
    const box = node.getBoundingClientRect();
    if (box.left + node.offsetWidth > window.innerWidth - 8) {
      node.classList.add("flip");
    }
    const shift = slideToFit(box.top, node.offsetHeight, window.innerHeight);
    if (shift !== 0) node.style.marginTop = `${shift}px`;
  }
</script>

<svelte:window
  oncontextmenu={onContextMenu}
  onkeydowncapture={onKey}
  onpointerdown={onPointerDown}
  onblur={close}
  onresize={close}
/>

{#if current}
  <div
    bind:this={root}
    class="root"
    data-context-menu
    use:place={{ x: current.x, y: current.y }}
  >
    {@render panel(entries, 0)}
  </div>
{/if}

{#snippet panel(list: ContextEntry[], depth: number)}
  <div
    class="panel"
    class:sub={depth > 0}
    role="menu"
    aria-label={current?.label}
    data-depth={depth}
    use:fitSub
  >
    {#each list as entry (entry.id)}
      <div class="slot">
        <button
          type="button"
          class="item"
          class:danger={entry.danger}
          class:separated={entry.separated}
          class:open={path[depth] === entry.id}
          role={entry.checked === undefined
            ? "menuitem"
            : entry.dot !== undefined
              ? "menuitemcheckbox"
              : "menuitemradio"}
          aria-checked={entry.checked}
          aria-haspopup={entry.children ? "menu" : undefined}
          aria-expanded={entry.children ? path[depth] === entry.id : undefined}
          disabled={entry.disabled}
          data-entry={entry.id}
          style:padding-left="{10 + (entry.depth ?? 0) * 12}px"
          onpointerenter={() => hover(entry, depth)}
          onclick={() => pick(entry, depth)}
        >
          {#if entry.dot}
            <span class="dot" style:background={entry.dot}></span>
          {/if}
          <span class="name">{entry.label}</span>
          {#if entry.checked}
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
          {#if entry.children}
            <svg
              class="more"
              viewBox="0 0 12 12"
              width="10"
              height="10"
              aria-hidden="true"
            >
              <path
                d="M4.5 2.5L8 6l-3.5 3.5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          {/if}
        </button>
        {#if entry.children && path[depth] === entry.id}
          {@render panel(entry.children, depth + 1)}
        {/if}
      </div>
    {/each}
  </div>
{/snippet}

<style>
  .root {
    position: fixed;
    z-index: 30;
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 200px;
    max-width: 300px;
    padding: 5px;
    border-radius: var(--radius-card);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    animation: pop-in var(--motion-fast) var(--ease-out);
    transform-origin: top left;
  }

  .panel.sub {
    position: absolute;
    top: -5px;
    left: calc(100% + 2px);
    max-height: min(360px, calc(100vh - 16px));
    overflow-y: auto;
  }

  :global(.panel.sub.flip) {
    right: calc(100% + 2px);
    left: auto;
  }

  .slot {
    position: relative;
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    min-height: 26px;
    padding: 0 var(--space-2) 0 10px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: default;
  }

  /* Solid accent, as in a system menu: the entry under the pointer is unmistakable. */
  .item:hover:not(:disabled),
  .item:focus-visible,
  .item.open {
    background: var(--accent);
    color: var(--on-accent);
    outline: none;
  }

  .item.danger {
    color: var(--danger);
  }

  .item.danger:hover:not(:disabled),
  .item.danger:focus-visible {
    background: var(--danger);
    color: var(--on-accent);
  }

  .item:disabled {
    opacity: 0.45;
  }

  .item.separated {
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

  .dot {
    flex: none;
    width: 9px;
    height: 9px;
    border-radius: var(--radius-chip);
  }

  svg {
    flex: none;
  }

  .more {
    opacity: 0.7;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel {
      animation: none;
    }
  }
</style>
