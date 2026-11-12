import { describe, expect, it } from "vitest";

import { virtualRange } from "./virtual";

describe("virtualRange", () => {
  it("renders nothing for an empty list", () => {
    expect(virtualRange(0, 600, 78, 0)).toEqual({
      start: 0,
      end: 0,
      before: 0,
      after: 0,
    });
  });

  it("renders the first rows plus the overscan at the top", () => {
    const range = virtualRange(0, 780, 78, 1000, 2);
    expect(range.start).toBe(0);
    expect(range.end).toBe(11 + 2);
    expect(range.before).toBe(0);
    expect(range.after).toBe((1000 - range.end) * 78);
  });

  it("keeps the space above and below equal to the rows left out", () => {
    const range = virtualRange(78 * 500, 600, 78, 1000, 3);
    expect(range.start).toBe(497);
    expect(range.before).toBe(497 * 78);
    expect(range.before + (range.end - range.start) * 78 + range.after).toBe(
      1000 * 78,
    );
  });

  it("clamps at the end and ignores a negative scroll", () => {
    const end = virtualRange(78 * 5000, 600, 78, 40);
    expect(end.end).toBe(40);
    expect(end.after).toBe(0);
    expect(end.start).toBeLessThan(40);
    expect(virtualRange(-50, 600, 78, 40).start).toBe(0);
  });

  it("never renders more than a window of rows out of a huge list", () => {
    const range = virtualRange(78 * 20000, 800, 78, 50000);
    expect(range.end - range.start).toBeLessThan(40);
  });
});
