import { describe, expect, it } from "vitest";

import { FakeEventsApi } from "../testing";
import { EventsStore } from "./events.svelte";

function setup() {
  const api = new FakeEventsApi();
  const store = new EventsStore({ api, noticeMs: 60_000 });
  return { api, store };
}

describe("EventsStore (D-94)", () => {
  it("loads the occurrences of a window and spreads them by day", async () => {
    const { api, store } = setup();
    api.makeEvent({
      id: "a",
      title: "A",
      startDate: "2026-03-10",
      endDate: "2026-03-10",
    });
    api.makeEvent({
      id: "b",
      title: "Fuera",
      startDate: "2026-04-01",
      endDate: "2026-04-01",
    });
    await store.show("2026-03-09", "2026-03-11");
    expect(store.occurrences.map((item) => item.eventId)).toEqual(["a"]);
    expect([...store.byDate.keys()]).toEqual(["2026-03-10"]);
  });

  it("hides what the tag filter hides", async () => {
    const { api, store } = setup();
    api.makeEvent({ id: "a", tagIds: ["t1"] });
    api.makeEvent({ id: "b", tagIds: [] });
    store.filter = (occurrence) => occurrence.tagIds.length === 0;
    await store.show("2026-03-10", "2026-03-10");
    expect(store.visible.map((item) => item.eventId)).toEqual(["b"]);
  });

  it("creates an event and shows it in the window", async () => {
    const { store } = setup();
    await store.show("2026-03-10", "2026-03-10");
    const created = await store.create({
      title: "Dentista",
      startDate: "2026-03-10",
      startTime: "09:00",
      endTime: "09:30",
    });
    expect(created?.title).toBe("Dentista");
    expect(store.occurrences).toHaveLength(1);
  });

  it("reports an error and keeps going when the backend refuses", async () => {
    const { api, store } = setup();
    await store.show("2026-03-10", "2026-03-10");
    api.failWith = "VALIDATION: fecha no válida";
    expect(await store.create({ startDate: "x" })).toBeNull();
    expect(store.notice?.kind).toBe("error");
    expect(store.notice?.message).toContain("fecha no válida");
  });

  it("moves an occurrence to another day and time keeping its length", async () => {
    const { api, store } = setup();
    const event = api.makeEvent({ id: "a" });
    await store.show("2026-03-09", "2026-03-13");
    const occurrence = store.occurrences[0];
    expect(await store.move(occurrence, "2026-03-12", 14 * 60)).toBe(true);
    expect(event).toMatchObject({
      startDate: "2026-03-12",
      endDate: "2026-03-12",
      startTime: "14:00",
      endTime: "15:00",
    });
    expect(store.occurrences[0].startDate).toBe("2026-03-12");
  });

  it("resizes an occurrence by its end", async () => {
    const { api, store } = setup();
    const event = api.makeEvent({ id: "a" });
    await store.show("2026-03-10", "2026-03-10");
    await store.resize(store.occurrences[0], 12 * 60 + 30);
    expect(event.endTime).toBe("12:30");
    expect(event.startTime).toBe("10:00");
  });

  it("deletes with the chance to undo", async () => {
    const { api, store } = setup();
    const event = api.makeEvent({ id: "a" });
    await store.show("2026-03-10", "2026-03-10");
    expect(await store.remove(event)).toBe(true);
    expect(store.occurrences).toHaveLength(0);
    expect(store.deleted?.id).toBe("a");
    await store.undoDelete();
    expect(store.occurrences).toHaveLength(1);
    expect(store.deleted).toBeNull();
  });

  it("does not offer to undo a deletion that is part of a conversion", async () => {
    const { api, store } = setup();
    const event = api.makeEvent({ id: "a" });
    await store.show("2026-03-10", "2026-03-10");
    await store.remove(event, false);
    expect(store.deleted).toBeNull();
  });

  it("the last window asked for wins", async () => {
    const { api, store } = setup();
    api.makeEvent({ id: "a", startDate: "2026-03-10", endDate: "2026-03-10" });
    api.makeEvent({ id: "b", startDate: "2026-05-10", endDate: "2026-05-10" });
    const first = store.show("2026-03-01", "2026-03-31");
    const second = store.show("2026-05-01", "2026-05-31");
    await Promise.all([first, second]);
    expect(store.occurrences.map((item) => item.eventId)).toEqual(["b"]);
  });
});
