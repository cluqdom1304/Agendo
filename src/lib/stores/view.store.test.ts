import { describe, expect, it } from "vitest";

import { ViewStore, type ViewApi } from "./view.svelte";

type Key = "view_mode" | "app_section";

class FakeViewApi implements ViewApi {
  saved: Record<Key, string | null>;
  writes: string[] = [];
  failReads = false;
  failWrites = false;

  constructor(mode: string | null = null, section: string | null = null) {
    this.saved = { view_mode: mode, app_section: section };
  }

  async settingsGet(key: Key) {
    if (this.failReads) throw "DB: fallo simulado";
    return this.saved[key];
  }

  async settingsSet(key: Key, value: string) {
    if (this.failWrites) throw "DB: fallo simulado";
    this.writes.push(value);
    this.saved[key] = value;
  }
}

const make = (api: ViewApi) =>
  new ViewStore({
    api,
    now: () => new Date("2026-03-10T10:00:00"),
    firstDay: () => 1,
  });

describe("ViewStore", () => {
  it("starts on the list, centred on today, and is not ready until it has read the setting", async () => {
    const store = make(new FakeViewApi());
    expect(store.ready).toBe(false);
    expect(store.mode).toBe("list");
    expect(store.anchor).toBe("2026-03-10");
    await store.load();
    expect(store.ready).toBe(true);
  });

  it("starts on «Hoy» and remembers the notes section", async () => {
    const api = new FakeViewApi(null);
    const store = make(api);
    await store.load();
    expect(store.section).toBe("today");
    store.setSection("notes");
    expect(store.section).toBe("notes");
    expect(api.writes).toEqual(["notes"]);
    store.setSection("notes");
    expect(api.writes).toHaveLength(1);

    const reopened = make(new FakeViewApi(null, "notes"));
    await reopened.load();
    expect(reopened.section).toBe("notes");
  });

  it("restores the saved view", async () => {
    const store = make(new FakeViewApi("week"));
    await store.load();
    expect(store.mode).toBe("week");
  });

  it("falls back to the list for an unknown value or an unreadable setting", async () => {
    const unknown = make(new FakeViewApi("year"));
    await unknown.load();
    expect(unknown.mode).toBe("list");

    const broken = new FakeViewApi("week");
    broken.failReads = true;
    const store = make(broken);
    await store.load();
    expect(store.mode).toBe("list");
    expect(store.ready).toBe(true);
  });

  it("saves each change once and survives a failed save", async () => {
    const api = new FakeViewApi();
    const store = make(api);
    store.setMode("month");
    store.setMode("month");
    store.setMode("day");
    expect(api.writes).toEqual(["calendar", "month", "day"]);

    api.failWrites = true;
    store.setMode("week");
    await Promise.resolve();
    expect(store.mode).toBe("week");
  });

  it("shifts by a day, a week or a month depending on the view", () => {
    const store = make(new FakeViewApi());
    store.mode = "day";
    store.shift(1);
    expect(store.anchor).toBe("2026-03-11");
    store.mode = "week";
    store.shift(-1);
    expect(store.anchor).toBe("2026-03-04");
    store.mode = "month";
    store.shift(1);
    expect(store.anchor).toBe("2026-04-04");
    store.mode = "list";
    store.shift(1);
    expect(store.anchor).toBe("2026-04-04");
    store.goToToday();
    expect(store.anchor).toBe("2026-03-10");
  });

  it("remembers the last calendar view to come back to from the list", async () => {
    const store = make(new FakeViewApi("month"));
    expect(store.calendarMode).toBe("week");
    await store.load();
    expect(store.calendarMode).toBe("month");
    store.setMode("day");
    store.setMode("list");
    expect(store.calendarMode).toBe("day");
  });

  it("showDay centres on that day and switches to the day view", () => {
    const api = new FakeViewApi();
    const store = make(api);
    store.section = "calendar";
    store.mode = "month";
    store.showDay("2026-03-18");
    expect(store.mode).toBe("day");
    expect(store.anchor).toBe("2026-03-18");
    expect(api.writes).toEqual(["day"]);
  });
});

describe("sections of the sidebar (F19-04)", () => {
  it("restores the saved section and falls back to «Hoy»", async () => {
    for (const [saved, expected] of [
      ["today", "today"],
      ["notes", "notes"],
      ["tasks", "tasks"],
      ["calendar", "calendar"],
      ["algo raro", "today"],
      [null, "today"],
    ] as const) {
      const store = new ViewStore({ api: new FakeViewApi(null, saved) });
      await store.load();
      expect(store.section).toBe(expected);
    }
  });

  it("migrates the 2.0.x `agenda` to «Tareas» or «Calendario» by the saved view and saves it", async () => {
    const list = new FakeViewApi("list", "agenda");
    const store = make(list);
    await store.load();
    expect(store.section).toBe("tasks");
    expect(list.saved.app_section).toBe("tasks");

    const week = new FakeViewApi("week", "agenda");
    const calendar = make(week);
    await calendar.load();
    expect(calendar.section).toBe("calendar");
    expect(calendar.mode).toBe("week");
    expect(week.saved.app_section).toBe("calendar");
  });

  it("keeps the list in «Tareas» and the calendar views in «Calendario»", async () => {
    const api = new FakeViewApi("month", "tasks");
    const store = make(api);
    await store.load();
    expect(store.mode).toBe("list");
    expect(store.calendarMode).toBe("month");

    store.setSection("calendar");
    expect(store.mode).toBe("month");
    store.setMode("list");
    expect(store.section).toBe("tasks");
    store.setSection("notes");
    store.setMode("day");
    expect(store.section).toBe("calendar");
    expect(store.inAgenda).toBe(true);
  });

  it("showAgenda goes to the list only from outside the agenda", () => {
    const store = make(new FakeViewApi());
    store.setSection("notes");
    store.showAgenda();
    expect(store.section).toBe("tasks");
    store.setSection("calendar");
    store.showAgenda();
    expect(store.section).toBe("calendar");
  });

  it("saves the change of section", () => {
    const api = new FakeViewApi();
    const store = new ViewStore({ api });
    store.setSection("notes");
    store.setSection("today");
    expect(store.section).toBe("today");
    expect(api.writes).toContain("today");
  });
});

describe("back and forward (F19-03)", () => {
  it("walks the visited sections both ways and a new visit drops what was ahead", () => {
    const store = make(new FakeViewApi());
    expect(store.canGoBack).toBe(false);
    store.setSection("notes");
    store.setSection("calendar");
    store.goBack();
    expect(store.section).toBe("notes");
    expect(store.canGoForward).toBe(true);
    store.goBack();
    expect(store.section).toBe("today");
    expect(store.canGoBack).toBe(false);
    store.goForward();
    expect(store.section).toBe("notes");

    store.setSection("tasks");
    expect(store.canGoForward).toBe(false);
    store.goForward();
    expect(store.section).toBe("tasks");
  });

  it("does not record a change of calendar view inside the calendar", () => {
    const store = make(new FakeViewApi());
    store.setSection("calendar");
    store.setMode("month");
    store.setMode("day");
    store.goBack();
    expect(store.section).toBe("today");
  });
});
