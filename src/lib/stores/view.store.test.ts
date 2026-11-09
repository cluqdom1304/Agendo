import { describe, expect, it } from "vitest";

import { ViewStore, type ViewApi } from "./view.svelte";

class FakeViewApi implements ViewApi {
  saved: string | null;
  writes: string[] = [];
  failReads = false;
  failWrites = false;

  constructor(saved: string | null = null) {
    this.saved = saved;
  }

  async settingsGet() {
    if (this.failReads) throw "DB: fallo simulado";
    return this.saved;
  }

  async settingsSet(_key: "view_mode", value: string) {
    if (this.failWrites) throw "DB: fallo simulado";
    this.writes.push(value);
    this.saved = value;
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
    expect(api.writes).toEqual(["month", "day"]);

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
    store.mode = "month";
    store.showDay("2026-03-18");
    expect(store.mode).toBe("day");
    expect(store.anchor).toBe("2026-03-18");
    expect(api.writes).toEqual(["day"]);
  });
});
