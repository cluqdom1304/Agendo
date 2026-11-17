// Debounced, serialised saving of one note (SPEC F-08: 400 ms).

export const NOTE_SAVE_DELAY_MS = 400;

/** Savers with text not yet sent, so the window can flush them all before it closes. */
const dirtySavers = new Set<NoteSaver>();

/** Saves every pending note text now; resolves when all the saves finished. */
export async function flushAllSavers(): Promise<void> {
  await Promise.all([...dirtySavers].map((saver) => saver.flush()));
}

export class NoteSaver {
  readonly #save: (contentJson: string) => Promise<void>;
  readonly #delayMs: number;
  #pending: string | null = null;
  #timer: ReturnType<typeof setTimeout> | null = null;
  #running: Promise<void> = Promise.resolve();

  constructor(
    save: (contentJson: string) => Promise<void>,
    delayMs = NOTE_SAVE_DELAY_MS,
  ) {
    this.#save = save;
    this.#delayMs = delayMs;
  }

  /** True while there is text the backend has not been asked to save yet. */
  get dirty(): boolean {
    return this.#pending !== null;
  }

  /** Remembers the latest text and saves it once typing pauses for the delay. */
  schedule(contentJson: string): void {
    this.#pending = contentJson;
    dirtySavers.add(this);
    if (this.#timer !== null) clearTimeout(this.#timer);
    this.#timer = setTimeout(() => {
      this.#timer = null;
      void this.#drain();
    }, this.#delayMs);
  }

  /** Saves right now whatever is pending (closing the note, leaving the app). */
  flush(): Promise<void> {
    if (this.#timer !== null) clearTimeout(this.#timer);
    this.#timer = null;
    return this.#drain();
  }

  // Saves never overlap, so an older answer cannot overwrite a newer text.
  #drain(): Promise<void> {
    this.#running = this.#running.then(async () => {
      const contentJson = this.#pending;
      if (contentJson === null) return;
      this.#pending = null;
      try {
        await this.#save(contentJson);
      } catch {
        // Keep the text for the next attempt unless newer text has replaced it.
        this.#pending ??= contentJson;
      }
    });
    this.#running = this.#running.then(() => {
      if (this.#pending === null) dirtySavers.delete(this);
    });
    return this.#running;
  }
}
