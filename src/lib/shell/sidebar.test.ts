import { describe, expect, it } from "vitest";

import { SIDEBAR_FOLD_WIDTH, SidebarState } from "./sidebar.svelte";

describe("SidebarState (F19-05)", () => {
  it("starts folded in a narrow window and open in a wide one", () => {
    expect(new SidebarState(SIDEBAR_FOLD_WIDTH - 1).open).toBe(false);
    expect(new SidebarState(SIDEBAR_FOLD_WIDTH).open).toBe(true);
  });

  it("folds when the window narrows and comes back when it widens", () => {
    const state = new SidebarState(1200);
    state.setWidth(800);
    expect(state.open).toBe(false);
    state.setWidth(1100);
    expect(state.open).toBe(true);
  });

  it("keeps a fold chosen by the user across a narrow spell", () => {
    const state = new SidebarState(1200);
    state.toggle();
    state.setWidth(800);
    state.setWidth(1200);
    expect(state.open).toBe(false);
  });

  it("opens by hand while narrow until the window crosses the limit", () => {
    const state = new SidebarState(800);
    state.toggle();
    expect(state.open).toBe(true);
    state.setWidth(850);
    expect(state.open).toBe(true);
    state.setWidth(1000);
    state.setWidth(800);
    expect(state.open).toBe(false);
  });

  it("reveal opens it for a drag in wide and narrow windows", () => {
    const state = new SidebarState(1200);
    state.toggle();
    state.reveal();
    expect(state.open).toBe(true);
    const narrow = new SidebarState(700);
    narrow.reveal();
    expect(narrow.open).toBe(true);
  });
});
