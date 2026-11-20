import { describe, expect, it } from "vitest";

import { INBOX_HIDE_WIDTH, InboxState } from "./inbox.svelte";

describe("InboxState (F20-04)", () => {
  it("is open in a wide window and hidden in a narrow one", () => {
    expect(new InboxState(INBOX_HIDE_WIDTH).open).toBe(true);
    const narrow = new InboxState(INBOX_HIDE_WIDTH - 1);
    expect(narrow.open).toBe(false);
    expect(narrow.available).toBe(false);
  });

  it("comes back after a narrow spell unless the user folded it", () => {
    const state = new InboxState(1400);
    state.setWidth(900);
    expect(state.open).toBe(false);
    state.setWidth(1400);
    expect(state.open).toBe(true);
    state.toggle();
    state.setWidth(900);
    state.setWidth(1400);
    expect(state.open).toBe(false);
  });

  it("ignores the toggle while the window is narrow", () => {
    const state = new InboxState(800);
    state.toggle();
    state.setWidth(1400);
    expect(state.open).toBe(true);
  });
});
