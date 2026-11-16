import { describe, expect, it } from "vitest";

import { blocksOf, diffBlocks } from "./versionDiff";

const doc = (...paragraphs: string[]) =>
  JSON.stringify({
    type: "doc",
    content: paragraphs.map((text) => ({
      type: "paragraph",
      content: text ? [{ type: "text", text }] : [],
    })),
  });

describe("blocksOf", () => {
  it("returns one entry per non-empty block, including list items and cells", () => {
    const json = JSON.stringify({
      type: "doc",
      content: [
        { type: "heading", content: [{ type: "text", text: "Título" }] },
        {
          type: "bulletList",
          content: [
            {
              type: "listItem",
              content: [
                {
                  type: "paragraph",
                  content: [
                    { type: "text", text: "uno" },
                    { type: "hardBreak" },
                    { type: "text", text: "dos" },
                  ],
                },
              ],
            },
          ],
        },
        { type: "paragraph" },
      ],
    });
    expect(blocksOf(json)).toEqual(["Título", "uno dos"]);
  });

  it("tolerates documents that are not JSON", () => {
    expect(blocksOf("texto suelto")).toEqual([]);
    expect(blocksOf("{roto")).toEqual([]);
  });
});

describe("diffBlocks", () => {
  it("marks what was removed and what was added, keeping the common blocks in order", () => {
    const lines = diffBlocks(["a", "b", "c"], ["a", "c", "d"]);
    expect(lines).toEqual([
      { kind: "same", text: "a" },
      { kind: "removed", text: "b" },
      { kind: "same", text: "c" },
      { kind: "added", text: "d" },
    ]);
  });

  it("is all same for equal documents and all added or removed against an empty one", () => {
    expect(diffBlocks(["a"], ["a"])).toEqual([{ kind: "same", text: "a" }]);
    expect(diffBlocks([], ["a"])).toEqual([{ kind: "added", text: "a" }]);
    expect(diffBlocks(["a"], [])).toEqual([{ kind: "removed", text: "a" }]);
    expect(blocksOf(doc("x"))).toEqual(["x"]);
  });
});
