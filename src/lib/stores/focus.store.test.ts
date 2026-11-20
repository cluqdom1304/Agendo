import { describe, expect, it, vi } from "vitest";

import { FocusStore, formatCountdown, remainingMs } from "./focus.svelte";
import type { FocusApi } from "./focus.svelte";
import type { TimeEntry } from "../types";

const START = Date.parse("2026-11-20T10:00:00Z");

function entry(overrides: Partial<TimeEntry> = {}): TimeEntry {
  return {
    id: "e1",
    taskId: "t1",
    startedAt: "2026-11-20T10:00:00Z",
    endedAt: null,
    plannedMinutes: 25,
    ...overrides,
  };
}

function setup(current: TimeEntry | null = null) {
  let clock = START;
  let running = current;
  const api: FocusApi = {
    focusCurrent: vi.fn(async () => running),
    focusStart: vi.fn(async (taskId: string, plannedMinutes: number) => {
      running = entry({ taskId, plannedMinutes });
      return running;
    }),
    focusStop: vi.fn(async () => {
      const closed = running && { ...running, endedAt: "2026-11-20T10:05:00Z" };
      running = null;
      return closed;
    }),
  };
  const store = new FocusStore({ api, now: () => clock, tickMs: 0 });
  return {
    api,
    store,
    advance: (ms: number) => (clock += ms),
    finish: () => (running = null),
  };
}

describe("focus countdown (F25-01)", () => {
  it("measures what is left of a session and formats it", () => {
    expect(remainingMs(entry(), START + 60_000)).toBe(24 * 60_000);
    expect(remainingMs(entry(), START + 99 * 60_000)).toBe(0);
    expect(formatCountdown(24 * 60_000 + 30)).toBe("24:01");
    expect(formatCountdown(3_725_000)).toBe("1:02:05");
    expect(formatCountdown(0)).toBe("00:00");
  });

  it("starts a session and counts down", async () => {
    const { store, advance, api } = setup();
    await store.start("t1", 25);
    expect(api.focusStart).toHaveBeenCalledWith("t1", 25);
    expect(store.label).toBe("25:00");
    advance(90_000);
    await store.tick();
    expect(store.label).toBe("23:30");
  });

  it("asks the backend again once the time is over and clears the session", async () => {
    const { store, advance, finish, api } = setup(entry());
    await store.load();
    advance(25 * 60_000);
    await store.tick();
    expect(api.focusCurrent).toHaveBeenCalledTimes(2);
    expect(store.current).not.toBeNull();
    finish();
    await store.tick();
    expect(store.current).toBeNull();
    expect(store.label).toBe("00:00");
  });

  it("stops the session and says so", async () => {
    const { store } = setup(entry());
    await store.load();
    await store.stop();
    expect(store.current).toBeNull();
    expect(store.notice?.message).toBe("Sesión de enfoque terminada");
  });

  it("reports a refused start without keeping a session", async () => {
    const { api, store } = setup();
    vi.mocked(api.focusStart).mockRejectedValueOnce(
      "CONFLICT: ya hay una sesión de enfoque en marcha",
    );
    await store.start("t1", 25);
    expect(store.current).toBeNull();
    expect(store.notice?.kind).toBe("error");
  });
});
