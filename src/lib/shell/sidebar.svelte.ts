/** Below this window width the sidebar folds by itself so the panel keeps its room (F19-05). */
export const SIDEBAR_FOLD_WIDTH = 900;

/**
 * Whether the sidebar shows. The user's choice is kept apart from the automatic fold: a narrow
 * window folds it without forgetting that the user wanted it, and widening the window brings
 * it back. Opening it by hand while narrow lasts until the window crosses the limit again.
 */
export class SidebarState {
  #wanted = $state(true);
  #narrow = $state(false);
  #peek = $state(false);

  constructor(width: number) {
    this.#narrow = width < SIDEBAR_FOLD_WIDTH;
  }

  get open(): boolean {
    return this.#narrow ? this.#peek : this.#wanted;
  }

  get narrow(): boolean {
    return this.#narrow;
  }

  toggle(): void {
    if (this.#narrow) this.#peek = !this.#peek;
    else this.#wanted = !this.#wanted;
  }

  /** Opens it because a drag needs somewhere to drop. */
  reveal(): void {
    if (this.#narrow) this.#peek = true;
    else this.#wanted = true;
  }

  setWidth(width: number): void {
    const narrow = width < SIDEBAR_FOLD_WIDTH;
    if (narrow === this.#narrow) return;
    this.#narrow = narrow;
    this.#peek = false;
  }
}
