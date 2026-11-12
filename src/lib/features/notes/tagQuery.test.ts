import { describe, expect, it } from "vitest";

import type { Tag } from "../../types";
import { findTagQuery, matchTags } from "./tagQuery";

const tag = (id: string, name: string): Tag => ({
  id,
  name,
  colorHex: "#4C8DFF",
  parentId: null,
  position: 0,
});

describe("findTagQuery", () => {
  it("finds the letters after a # at the start or after a space", () => {
    expect(findTagQuery("#")).toBe("");
    expect(findTagQuery("#mat")).toBe("mat");
    expect(findTagQuery("repasar #fís")).toBe("fís");
    expect(findTagQuery("a #b-1_c")).toBe("b-1_c");
  });

  it("ignores a # inside a word, a finished tag or no #", () => {
    expect(findTagQuery("a#b")).toBeNull();
    expect(findTagQuery("#mat ")).toBeNull();
    expect(findTagQuery("sin etiqueta")).toBeNull();
    expect(findTagQuery("#a#b")).toBeNull();
  });
});

describe("matchTags", () => {
  const tags = [
    tag("1", "Matemáticas"),
    tag("2", "Física"),
    tag("3", "Automatización"),
    tag("4", "Mates"),
  ];

  it("lists names that start with the query first, then those that contain it", () => {
    expect(matchTags(tags, "mat").map((t) => t.name)).toEqual([
      "Matemáticas",
      "Mates",
      "Automatización",
    ]);
  });

  it("ignores case and accents", () => {
    expect(matchTags(tags, "FISI").map((t) => t.id)).toEqual(["2"]);
    expect(matchTags(tags, "fís").map((t) => t.id)).toEqual(["2"]);
  });

  it("offers the first tags when nothing is typed, up to the limit", () => {
    expect(matchTags(tags, "", 2).map((t) => t.id)).toEqual(["1", "2"]);
  });

  it("returns nothing when no tag matches", () => {
    expect(matchTags(tags, "zzz")).toEqual([]);
  });
});
