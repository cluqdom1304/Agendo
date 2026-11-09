import { describe, expect, it } from "vitest";

import { isHexColor, normalizeHex, TAG_PRESETS } from "./tagColor";

describe("tagColor", () => {
  it("only offers colours the backend accepts", () => {
    expect(TAG_PRESETS).toHaveLength(8);
    expect(TAG_PRESETS.every(isHexColor)).toBe(true);
    expect(new Set(TAG_PRESETS).size).toBe(8);
  });

  it("normalises what the user types into #RRGGBB", () => {
    expect(normalizeHex("#4c8dff")).toBe("#4C8DFF");
    expect(normalizeHex("  4c8dff ")).toBe("#4C8DFF");
    expect(normalizeHex("#abc")).toBe("#AABBCC");
  });

  it("rejects anything that is not a colour", () => {
    expect(normalizeHex("")).toBeNull();
    expect(normalizeHex("#12345")).toBeNull();
    expect(normalizeHex("#12345g")).toBeNull();
    expect(normalizeHex("#1234567")).toBeNull();
    expect(normalizeHex("rojo")).toBeNull();
  });
});
