import { describe, expect, it } from "vitest";

import { FakeSummaryApi } from "../testing";
import { SummaryStore } from "./summary.svelte";

function make() {
  const api = new FakeSummaryApi();
  const store = new SummaryStore({
    api,
    today: () => "2026-03-10",
    firstDay: () => 1,
  });
  return { api, store };
}

describe("SummaryStore (F26-02)", () => {
  it("opens the week of today with the configured first day", async () => {
    const { api, store } = make();
    await store.show();
    expect(api.calls).toEqual([{ date: "2026-03-10", firstDay: 1 }]);
    expect(store.status).toBe("ready");
    expect(store.summary?.start).toBe("2026-03-09");
    expect(store.current).toBe(true);
  });

  it("pages by weeks and is no longer on the current week afterwards", async () => {
    const { api, store } = make();
    await store.show();
    api.summary = { ...api.summary, start: "2026-03-16", end: "2026-03-22" };
    await store.go(1);
    expect(api.calls.at(-1)?.date).toBe("2026-03-17");
    expect(store.current).toBe(false);
    await store.go(-2);
    expect(api.calls.at(-1)?.date).toBe("2026-03-03");
  });

  it("keeps the answer of the last request when two are in flight", async () => {
    const { api, store } = make();
    let release: () => void = () => undefined;
    const slow = new Promise<void>((resolve) => (release = resolve));
    const original = api.summaryWeek.bind(api);
    let first = true;
    api.summaryWeek = async (date, firstDay) => {
      if (first) {
        first = false;
        await slow;
        return { ...api.summary, completed: 1 };
      }
      return original(date, firstDay);
    };
    const older = store.show();
    api.summary = { ...api.summary, completed: 7 };
    await store.go(1);
    release();
    await older;
    expect(store.summary?.completed).toBe(7);
  });

  it("reports a failure and recovers on refresh", async () => {
    const { api, store } = make();
    api.failWith = "DB: sin acceso";
    await store.show();
    expect(store.status).toBe("error");
    expect(store.error).toContain("DB:");
    api.failWith = null;
    await store.refresh();
    expect(store.status).toBe("ready");
    expect(store.error).toBeNull();
  });

  it("exports the week shown and says whether it worked", async () => {
    const { api, store } = make();
    await store.show();
    expect(await store.exportMarkdown("C:/resumen.md")).toBe(true);
    expect(api.exported).toEqual([
      { date: "2026-03-10", firstDay: 1, path: "C:/resumen.md" },
    ]);
    expect(store.exportState).toBe("done");
    api.failWith = "IO: sin espacio";
    expect(await store.exportMarkdown("C:/resumen.md")).toBe(false);
    expect(store.exportState).toBe("error");
  });
});
