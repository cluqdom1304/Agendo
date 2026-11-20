/** Below this window width the inbox panel hides so the content keeps its room (F20-04). */
export const INBOX_HIDE_WIDTH = 1100;

/**
 * Whether the inbox panel shows. The user's choice survives a narrow spell: the panel hides
 * while the window is narrow and comes back when it widens, unless the user folded it.
 */
export class InboxState {
  #wanted = $state(true);
  #narrow = $state(false);

  constructor(width: number) {
    this.#narrow = width < INBOX_HIDE_WIDTH;
  }

  /** The window is wide enough to offer the panel at all. */
  get available(): boolean {
    return !this.#narrow;
  }

  get open(): boolean {
    return this.#wanted && !this.#narrow;
  }

  toggle(): void {
    if (!this.#narrow) this.#wanted = !this.#wanted;
  }

  setWidth(width: number): void {
    this.#narrow = width < INBOX_HIDE_WIDTH;
  }
}
