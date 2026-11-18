import { describe, expect, it, vi } from "vitest";

import { MAX_SAVED, SavedSearches } from "./savedSearches.svelte";

function setup(stored: string | null = null) {
  const api = {
    settingsGet: vi.fn(async () => stored),
    settingsSet: vi.fn(async () => undefined),
  };
  return { api, store: new SavedSearches(api) };
}

describe("saved searches (F17-08)", () => {
  it("loads the stored list and ignores what is not a list of texts", async () => {
    const { store } = setup('["is:pendiente","desde:2026-01-01"]');
    await store.load();
    expect(store.items).toEqual(["is:pendiente", "desde:2026-01-01"]);
    for (const bad of ["no json", '{"a":1}', "[1,2]"]) {
      const { store: other } = setup(bad);
      await other.load();
      expect(other.items).toEqual([]);
    }
  });

  it("adds trimmed queries once, refuses commands and empty text, and removes", async () => {
    const { store, api } = setup();
    await store.load();
    expect(await store.add("  tag:SMA  ")).toBe(true);
    expect(api.settingsSet).toHaveBeenLastCalledWith(
      "saved_searches",
      '["tag:SMA"]',
    );
    expect(await store.add("tag:SMA")).toBe(false);
    expect(await store.add("   ")).toBe(false);
    expect(await store.add(">nueva tarea")).toBe(false);
    expect(await store.remove("tag:SMA")).toBe(true);
    expect(store.items).toEqual([]);
  });

  it("stops at the limit and puts the list back when saving fails", async () => {
    const { store, api } = setup(
      JSON.stringify(Array.from({ length: MAX_SAVED }, (_, i) => `q${i}`)),
    );
    await store.load();
    expect(store.canSave("otra")).toBe(false);
    expect(await store.remove("q0")).toBe(true);
    api.settingsSet.mockRejectedValueOnce("VALIDATION: no");
    expect(await store.add("nueva")).toBe(false);
    expect(store.items).not.toContain("nueva");
  });
});
