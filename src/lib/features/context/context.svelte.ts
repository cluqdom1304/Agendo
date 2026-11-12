// State of the context menu (D-50): the one menu that is open, and the requests it makes to
// sidebars that own the editing fields (rename a folder, edit a tag).

export interface ContextEntry {
  id: string;
  label: string;
  run?: () => void;
  /** A choice among several shows a tick when it is the current one. */
  checked?: boolean;
  danger?: boolean;
  disabled?: boolean;
  /** Draws a hairline above the entry. */
  separated?: boolean;
  /** Runs without closing the menu, so several tags can be ticked in a row. */
  keepOpen?: boolean;
  /** A colour dot, for tags. */
  dot?: string;
  /** Opens a submenu. */
  children?: ContextEntry[];
  /** Indentation, for folder trees. */
  depth?: number;
}

export interface OpenMenu {
  x: number;
  y: number;
  label: string;
  /** Called again whenever the stores change, so ticks and labels stay current. */
  build: () => ContextEntry[];
}

export type ContextIntent =
  | { kind: "folder"; mode: "new" | "rename" | "delete"; id: string }
  | { kind: "tag-edit"; id: string };

export class ContextMenuStore {
  current = $state.raw<OpenMenu | null>(null);
  intent = $state.raw<ContextIntent | null>(null);

  show(menu: OpenMenu): void {
    this.current = menu;
  }

  close(): void {
    this.current = null;
  }

  request(intent: ContextIntent): void {
    this.intent = intent;
  }

  /** The sidebar that owns the field calls this once it has acted on the request. */
  take<K extends ContextIntent["kind"]>(
    kind: K,
  ): Extract<ContextIntent, { kind: K }> | null {
    const intent = this.intent;
    if (!intent || intent.kind !== kind) return null;
    this.intent = null;
    return intent as Extract<ContextIntent, { kind: K }>;
  }
}

export const contextMenu = new ContextMenuStore();
