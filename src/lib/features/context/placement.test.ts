import { describe, expect, it } from "vitest";

import { clampStart, slideToFit, subSidePlacement } from "./placement";

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
