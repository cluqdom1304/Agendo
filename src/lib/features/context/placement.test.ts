import { describe, expect, it } from "vitest";

import {
  clampStart,
  dropdownPosition,
  slideToFit,
  subSidePlacement,
} from "./placement";

describe("clampStart", () => {
  it("keeps a menu that fits where it was asked", () => {
    expect(clampStart(100, 200, 720)).toBe(100);
  });

  it("pulls a menu that would fall off the bottom or the right edge back in", () => {
    expect(clampStart(700, 200, 720)).toBe(512);
  });

  it("never goes above or left of the margin, even if the window is smaller than the menu", () => {
    expect(clampStart(-30, 200, 720)).toBe(8);
    expect(clampStart(50, 900, 720)).toBe(8);
  });
});

describe("slideToFit", () => {
  it("does not move a submenu that fits", () => {
    expect(slideToFit(100, 200, 720)).toBe(0);
  });

  it("slides one that falls off the bottom up, by exactly what is missing", () => {
    expect(slideToFit(600, 200, 720)).toBe(-88);
  });

  it("never slides it above the top margin", () => {
    expect(slideToFit(40, 900, 720)).toBe(-32);
  });

  it("moves down a submenu that starts above the window", () => {
    expect(slideToFit(-20, 100, 720)).toBe(28);
  });
});

describe("subSidePlacement", () => {
  it("opens on the right when it fits", () => {
    expect(subSidePlacement(100, 300, 200, 1100)).toEqual({
      flip: false,
      shift: 0,
    });
  });

  it("opens on the left when the right edge is too close", () => {
    expect(subSidePlacement(700, 900, 250, 1000)).toEqual({
      flip: true,
      shift: 0,
    });
  });

  it("slides back in when neither side has room, never leaving the window", () => {
    const { flip, shift } = subSidePlacement(100, 300, 400, 600);
    expect(flip).toBe(false);
    expect(300 + 2 + shift + 400).toBeLessThanOrEqual(600 - 8);
    const narrow = subSidePlacement(300, 500, 420, 600);
    expect(narrow.flip).toBe(true);
    expect(300 - 2 - 420 + narrow.shift).toBeGreaterThanOrEqual(8);
  });
});

describe("dropdownPosition", () => {
  const limit = { width: 1000, height: 700 };
  const size = { width: 200, height: 200 };

  it("hangs below the button, aligned to its right edge", () => {
    expect(
      dropdownPosition({ top: 100, bottom: 130, right: 600 }, size, limit),
    ).toEqual({ left: 400, top: 136 });
  });

  it("never starts left of the window margin, even from a button near the left edge", () => {
    const { left } = dropdownPosition(
      { top: 100, bottom: 130, right: 90 },
      size,
      limit,
    );
    expect(left).toBe(8);
  });

  it("opens above the button when there is no room below", () => {
    expect(
      dropdownPosition({ top: 600, bottom: 630, right: 600 }, size, limit),
    ).toEqual({ left: 400, top: 394 });
  });

  it("stays inside the window when there is no room above or below", () => {
    const { top } = dropdownPosition(
      { top: 100, bottom: 130, right: 600 },
      { width: 200, height: 650 },
      limit,
    );
    expect(top).toBe(42);
  });
});
