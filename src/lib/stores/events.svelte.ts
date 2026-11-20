// The events (D-94) the views are showing: the occurrences of one window of days, and the
// operations on events. Series are edited as a whole: moving one occurrence moves the series.
import * as backend from "../api";
import {
  movePatch,
  occurrencesByDate,
  resizePatch,
} from "../features/events/occurrences";
import { TaskNotices } from "./taskNotices.svelte";
import type {
  CalendarEvent,
  EventInput,
  EventOccurrence,
  EventPatch,
} from "../types";
import type { Notice } from "./tasksTypes";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface EventsApi {
  eventsList(from: string, to: string): Promise<EventOccurrence[]>;
  eventGet(id: string): Promise<CalendarEvent>;
  eventCreate(input: EventInput): Promise<CalendarEvent>;
  eventUpdate(id: string, patch: EventPatch): Promise<CalendarEvent>;
  eventDelete(id: string): Promise<void>;
  eventRestore(id: string): Promise<CalendarEvent>;
}

export interface EventsStoreOptions {
  api?: EventsApi;
  /** Milliseconds a notice stays on screen. */
  noticeMs?: number;
}

export class EventsStore {
  /** Occurrences of the window, as the backend sent them. */
  occurrences = $state.raw<EventOccurrence[]>([]);
  /** First and last local day of the window; empty until a view asks for one. */
  from = $state("");
  to = $state("");
  /** The tag filter of the sidebar, set by the page. */
  filter = $state.raw<(occurrence: EventOccurrence) => boolean>(() => true);
  /** The event just deleted, for «Deshacer». */
  deleted = $state.raw<CalendarEvent | null>(null);

  readonly visible = $derived(this.occurrences.filter(this.filter));
  readonly byDate = $derived(
    occurrencesByDate(this.visible, this.from, this.to),
  );

  readonly #api: EventsApi;
  readonly #notices: TaskNotices;
  #request = 0;

  constructor(options: EventsStoreOptions = {}) {
    this.#api = options.api ?? backend;
    this.#notices = new TaskNotices(options.noticeMs ?? 6000);
  }

  get notice(): Notice | null {
    return this.#notices.notice;
  }

  dismissNotice(): void {
    this.#notices.dismiss();
  }

  /** Loads the occurrences between two local days, inclusive. The last request wins. */
  async show(from: string, to: string): Promise<void> {
    this.from = from;
    this.to = to;
    await this.#load();
  }

  /** Reads the window again, after something changed it. */
  refresh(): Promise<void> {
    return this.from ? this.#load() : Promise.resolve();
  }

  async #load(): Promise<void> {
    const request = ++this.#request;
    try {
      const list = await this.#api.eventsList(this.from, this.to);
      if (request === this.#request) this.occurrences = list;
    } catch (error) {
      if (request === this.#request) this.#notices.report(error);
    }
  }

  /** The whole event, repeat rule and reminders included (an occurrence has neither). */
  async get(id: string): Promise<CalendarEvent | null> {
    try {
      return await this.#api.eventGet(id);
    } catch (error) {
      this.#notices.report(error);
      return null;
    }
  }

  async create(input: EventInput): Promise<CalendarEvent | null> {
    try {
      const created = await this.#api.eventCreate(input);
      await this.refresh();
      return created;
    } catch (error) {
      this.#notices.report(error);
      return null;
    }
  }

  async update(id: string, patch: EventPatch): Promise<CalendarEvent | null> {
    try {
      const saved = await this.#api.eventUpdate(id, patch);
      await this.refresh();
      return saved;
    } catch (error) {
      this.#notices.report(error);
      return null;
    }
  }

  /** Drags an occurrence to `date` at `startMinutes`; a series moves as a whole. */
  async move(
    occurrence: EventOccurrence,
    date: string,
    startMinutes: number,
  ): Promise<boolean> {
    const master = await this.get(occurrence.eventId);
    if (!master) return false;
    const saved = await this.update(
      master.id,
      movePatch(master, occurrence.startDate, date, startMinutes),
    );
    return saved !== null;
  }

  /** Drags the lower edge of an occurrence to `endMinutes`. */
  async resize(
    occurrence: EventOccurrence,
    endMinutes: number,
  ): Promise<boolean> {
    const master = await this.get(occurrence.eventId);
    if (!master) return false;
    const saved = await this.update(master.id, resizePatch(master, endMinutes));
    return saved !== null;
  }

  /** Moves the event to the trash and, unless `offerUndo` is off, offers to bring it back. */
  async remove(event: CalendarEvent, offerUndo = true): Promise<boolean> {
    try {
      await this.#api.eventDelete(event.id);
    } catch (error) {
      this.#notices.report(error);
      return false;
    }
    if (offerUndo) this.deleted = event;
    await this.refresh();
    return true;
  }

  async undoDelete(): Promise<void> {
    const event = this.deleted;
    if (!event) return;
    this.deleted = null;
    try {
      await this.#api.eventRestore(event.id);
      await this.refresh();
    } catch (error) {
      this.#notices.report(error);
    }
  }

  dismissDeleted(): void {
    this.deleted = null;
  }

  inform(message: string): void {
    this.#notices.inform(message);
  }
}

export const eventsStore = new EventsStore();
