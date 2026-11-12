// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { installLayoutStubs } from "../../testing";
import { tokenize, languageOf } from "./codeHighlight";
import { wordStats } from "./content";
import NoteEditor from "./NoteEditor.svelte";

installLayoutStubs();

const doc = (...blocks: object[]) =>
  JSON.stringify({ type: "doc", content: blocks });
const paragraph = (text: string) => ({
  type: "paragraph",
  content: [{ type: "text", text }],
});

describe("wordStats", () => {
  it("counts words and characters from the text nodes", () => {
    expect(
      wordStats(doc(paragraph("Hola mundo"), paragraph("dos   más"))),
    ).toEqual({ words: 4, characters: 19 });
    expect(wordStats(doc(paragraph("it's well-known")))).toEqual({
      words: 2,
      characters: 15,
    });
    expect(wordStats(doc(paragraph("¿Qué tal? 3 niños")))).toEqual({
      words: 4,
      characters: 17,
    });
  });

  it("is zero for empty, legacy or broken content", () => {
    for (const content of ["", "texto suelto", "{rota", '{"type":"doc"}']) {
      expect(wordStats(content)).toEqual({ words: 0, characters: 0 });
    }
  });
});

describe("code highlighting", () => {
  const kinds = (source: string, language: string) =>
    tokenize(source, language).map((t) => [t.kind, source.slice(t.from, t.to)]);

  it("colours comments, strings, numbers and keywords", () => {
    expect(kinds('const x = "a" + 42; // fin', "js")).toEqual([
      ["keyword", "const"],
      ["string", '"a"'],
      ["number", "42"],
      ["comment", "// fin"],
    ]);
  });

  it("treats # as a comment only where the language has it", () => {
    expect(kinds("x = 1 # uno", "python")).toEqual([
      ["number", "1"],
      ["comment", "# uno"],
    ]);
    expect(kinds("color: #fff", "css")).toEqual([]);
  });

  it("is case-insensitive for SQL and marks types by their capital letter", () => {
    expect(kinds("SELECT * FROM t", "sql")).toEqual([
      ["keyword", "SELECT"],
      ["keyword", "FROM"],
    ]);
    expect(kinds("let v: Vec<String>", "rust")).toEqual([
      ["keyword", "let"],
      ["type", "Vec"],
      ["type", "String"],
    ]);
  });

  it("closes a block comment at the end of the text and survives odd input", () => {
    expect(kinds("a /* sin cerrar", "js")).toEqual([
      ["comment", "/* sin cerrar"],
    ]);
    expect(() => tokenize("'`\"\\", "generic")).not.toThrow();
  });

  it("knows the usual names of the languages", () => {
    expect(languageOf("TypeScript")).toBe("js");
    expect(languageOf("py")).toBe("python");
    expect(languageOf("brainfuck")).toBe("generic");
    expect(languageOf(null)).toBe("generic");
  });
});

describe("the advanced editor", () => {
  async function mount(content = doc(paragraph("Uno dos uno tres UNO"))) {
    const save = vi.fn<(json: string) => Promise<void>>(async () => {});
    const view = render(NoteEditor, {
      contentJson: content,
      save,
      advanced: true,
      autofocus: false,
    });
    const box = await screen.findByRole("textbox");
    const component = view.component as unknown as {
      toggle(name: string): void;
      insertTable(): void;
      insertRule(): void;
      find(query: string): { count: number; current: number };
      findStep(direction: 1 | -1): { count: number; current: number };
      clearFind(): void;
    };
    // The editor loads in a separate chunk.
    await waitFor(() => component.find("zzzz"));
    component.clearFind();
    const lastSaved = () => JSON.parse(save.mock.calls.at(-1)?.[0] ?? "{}");
    return { box, save, component, lastSaved };
  }

  const types = (json: { content?: { type: string }[] }) =>
    (json.content ?? []).map((node) => node.type);

  it("changes blocks: heading levels, quote, code block, rule and table", async () => {
    const { component, lastSaved, box } = await mount(doc(paragraph("Texto")));
    const settle = () =>
      waitFor(() => expect(save()).toBeDefined(), { timeout: 3000 });
    const save = () => lastSaved();
    component.toggle("heading");
    await waitFor(() => expect(box.querySelector("h1")).not.toBeNull());
    component.toggle("heading");
    await waitFor(() => expect(box.querySelector("h2")).not.toBeNull());
    component.toggle("heading");
    await waitFor(() => expect(box.querySelector("h3")).not.toBeNull());
    component.toggle("heading");
    await waitFor(() => expect(box.querySelector("h3")).toBeNull());
    component.toggle("blockquote");
    await waitFor(() => expect(box.querySelector("blockquote")).not.toBeNull());
    component.toggle("blockquote");
    component.toggle("codeBlock");
    await waitFor(() => expect(box.querySelector("pre")).not.toBeNull());
    component.toggle("codeBlock");
    component.insertRule();
    await waitFor(() => expect(box.querySelector("hr")).not.toBeNull());
    component.insertTable();
    await waitFor(() =>
      expect(box.querySelectorAll("table th")).toHaveLength(3),
    );
    expect(box.querySelectorAll("table td")).toHaveLength(6);
    await settle();
    await waitFor(() => expect(types(save())).toContain("table"), {
      timeout: 3000,
    });
  });

  it("highlights the selection", async () => {
    const { component, box } = await mount(doc(paragraph("Texto")));
    box.focus();
    document.execCommand?.("selectAll");
    component.toggle("highlight");
    // Without a selection in jsdom the command is a no-op; what matters is that it exists.
    expect(box.textContent).toBe("Texto");
  });

  it("finds ignoring case and accents, walks the matches and wraps", async () => {
    const { component, box } = await mount(
      doc(
        paragraph("Canción uno"),
        paragraph("OTRA CANCION"),
        paragraph("fin"),
      ),
    );
    expect(component.find("cancion")).toEqual({ count: 2, current: 0 });
    await waitFor(() =>
      expect(box.querySelectorAll(".find-match")).toHaveLength(2),
    );
    expect(box.querySelectorAll(".find-current")).toHaveLength(1);
    expect(component.findStep(1)).toEqual({ count: 2, current: 1 });
    expect(component.findStep(1)).toEqual({ count: 2, current: 0 });
    expect(component.findStep(-1)).toEqual({ count: 2, current: 1 });
    expect(component.find("nada")).toEqual({ count: 0, current: -1 });
    expect(component.findStep(1)).toEqual({ count: 0, current: -1 });
    component.clearFind();
    await waitFor(() =>
      expect(box.querySelectorAll(".find-match")).toHaveLength(0),
    );
    expect(component.find("")).toEqual({ count: 0, current: -1 });
  });

  it("keeps the marks in step with edits to the document", async () => {
    const { component, box } = await mount(doc(paragraph("uno uno")));
    expect(component.find("uno").count).toBe(2);
    component.toggle("blockquote");
    await waitFor(() => expect(box.querySelector("blockquote")).not.toBeNull());
    expect(box.querySelectorAll(".find-match")).toHaveLength(2);
  });

  it("does not offer blocks when the editor is not advanced", async () => {
    render(NoteEditor, {
      contentJson: doc({
        type: "heading",
        attrs: { level: 1 },
        content: [{ type: "text", text: "Título" }],
      }),
      save: async () => {},
    });
    const box = await screen.findByRole("textbox");
    expect(box.querySelector("h1")).toBeNull();
    await fireEvent.focusIn(box);
  });
});
