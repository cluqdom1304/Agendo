/** What is being dragged towards a tag of the sidebar. */
export interface DragSubject {
  id: string;
  title: string;
}

export interface ActiveDrag extends DragSubject {
  x: number;
  y: number;
  /** The `data-tag-drop` key under the pointer, if any. */
  overKey: string | null;
}

const THRESHOLD_PX = 5;
const DROP_ATTRIBUTE = "data-tag-drop";

/**
 * Pointer-based drag of a task onto a sidebar tag. HTML5 drag and drop is not used: the
 * Tauri window intercepts it for file drops. A press only becomes a drag after moving a few
 * pixels, so plain clicks keep working, and the click that ends a drag is swallowed.
 */
export class TagDrag {
  active = $state.raw<ActiveDrag | null>(null);

  #stop: (() => void) | null = null;

  begin(
    event: PointerEvent,
    subject: DragSubject,
    onDrop: (key: string) => void,
  ): void {
    if (event.button !== 0 || this.#stop) return;
    const startX = event.clientX;
    const startY = event.clientY;
    let dragging = false;

    const keyAt = (x: number, y: number): string | null => {
      if (typeof document.elementFromPoint !== "function") return null;
      const target = document
        .elementFromPoint(x, y)
        ?.closest(`[${DROP_ATTRIBUTE}]`);
      return target?.getAttribute(DROP_ATTRIBUTE) ?? null;
    };

    const onMove = (move: PointerEvent) => {
      if (
        !dragging &&
        Math.hypot(move.clientX - startX, move.clientY - startY) < THRESHOLD_PX
      ) {
        return;
      }
      dragging = true;
      document.body.classList.add("tag-dragging");
      this.active = {
        ...subject,
        x: move.clientX,
        y: move.clientY,
        overKey: keyAt(move.clientX, move.clientY),
      };
    };

    const finish = (drop: boolean) => {
      const key = this.active?.overKey ?? null;
      cleanup();
      if (!dragging) return;
      // The click that follows the release must not open or toggle anything.
      const swallow = (click: Event) => {
        click.stopPropagation();
        click.preventDefault();
      };
      window.addEventListener("click", swallow, { capture: true, once: true });
      setTimeout(() => window.removeEventListener("click", swallow, true), 0);
      if (drop && key !== null) onDrop(key);
    };

    const onUp = () => finish(true);
    const onKey = (key: KeyboardEvent) => {
      if (key.key !== "Escape") return;
      key.stopPropagation();
      finish(false);
    };

    const cleanup = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onCancel);
      window.removeEventListener("keydown", onKey, true);
      document.body.classList.remove("tag-dragging");
      this.active = null;
      this.#stop = null;
    };
    const onCancel = () => finish(false);

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onCancel);
    window.addEventListener("keydown", onKey, true);
    this.#stop = cleanup;
  }
}

export const tagDrag = new TagDrag();
