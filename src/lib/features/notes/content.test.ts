import { describe, expect, it } from "vitest";

import { hasNotes, taskRefDoc, taskRefIds, textToDoc } from "./content";

describe("textToDoc", () => {
  it("returns null for an empty or blank description", () => {
    expect(textToDoc("")).toBeNull();
    expect(textToDoc("  \n \r\n ")).toBeNull();
  });

  it("makes one paragraph per non-empty line", () => {
    const doc = JSON.parse(textToDoc("uno\r\n\r\ndos  \ntres") ?? "");
    expect(doc.type).toBe("doc");
    expect(
      doc.content.map(
        (paragraph: { content: { text: string }[] }) =>
          paragraph.content[0].text,
      ),
    ).toEqual(["uno", "dos", "tres"]);
  });

  it("produces a document that counts as having notes", () => {
    expect(hasNotes(textToDoc("algo") ?? "")).toBe(true);
  });

  it("keeps quotes and accents intact", () => {
    const doc = JSON.parse(textToDoc('Dijo "hola" — ¿qué tal?') ?? "");
    expect(doc.content[0].content[0].text).toBe('Dijo "hola" — ¿qué tal?');
  });
});

describe("taskRefIds", () => {
  const doc = (content: object[]) => JSON.stringify({ type: "doc", content });

  it("lists the referenced tasks in order, without repeats, at any depth", () => {
    const json = doc([
      { type: "taskRef", attrs: { taskId: "a" } },
      {
        type: "bulletList",
        content: [{ type: "taskRef", attrs: { taskId: "b" } }],
      },
      { type: "taskRef", attrs: { taskId: "a" } },
    ]);
    expect(taskRefIds(json)).toEqual(["a", "b"]);
  });

  it("ignores references without an id, text that is not a document and bad JSON", () => {
    expect(taskRefIds(doc([{ type: "taskRef", attrs: {} }]))).toEqual([]);
    expect(taskRefIds("")).toEqual([]);
    expect(taskRefIds("texto suelto")).toEqual([]);
    expect(taskRefIds("{roto")).toEqual([]);
  });

  it("round-trips with taskRefDoc", () => {
    expect(taskRefIds(taskRefDoc("t9"))).toEqual(["t9"]);
  });
});
