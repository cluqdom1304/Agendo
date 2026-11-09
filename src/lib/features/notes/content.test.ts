import { describe, expect, it } from "vitest";

import { hasNotes, textToDoc } from "./content";

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
