import { describe, expect, it } from "vitest";

import { clampStart, slideToFit } from "./placement";

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
