// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";

import type { Tag } from "../../types";
import type { NoteEditorHandle } from "./editorTypes";
import { LinkSuggest, TagSuggest } from "./noteSuggestions.svelte";

const where = { query: "", left: 0, top: 0, bottom: 0 };

function key(name: string): KeyboardEvent {
  return new KeyboardEvent("keydown", { key: name });
}

function tag(id: string, name: string): Tag {
  return { id, name, colorHex: "#000000", parentId: null, position: 0 };
}

describe("TagSuggest", () => {
  it("moves the highlight, picks with Enter and hides with Escape", () => {
    const onPick = vi.fn();
    const removeTagQuery = vi.fn();
    const menu = new TagSuggest(
      () => ({ tags: [tag("a", "alfa"), tag("b", "alto")], onPick }),
      () => ({ removeTagQuery }) as unknown as NoteEditorHandle,
    );
    menu.setQuery({ ...where, query: "al" });
    expect(menu.suggestions).toHaveLength(2);

    expect(menu.onKey(key("ArrowDown"))).toBe(true);
    expect(menu.highlighted).toBe(1);
    expect(menu.onKey(key("Enter"))).toBe(true);
    expect(removeTagQuery).toHaveBeenCalled();
    expect(onPick).toHaveBeenCalledWith(expect.objectContaining({ id: "b" }));

    expect(menu.onKey(key("Escape"))).toBe(true);
    expect(menu.suggestions).toHaveLength(0);
    expect(menu.onKey(key("x"))).toBe(false);
  });
});

describe("LinkSuggest", () => {
  it("offers to create a note when no title matches exactly", async () => {
    const menu = new LinkSuggest(
      () => ({
        search: async () => [{ id: "n1", title: "Otra" }],
        create: async () => null,
      }),
      () => null,
    );
    menu.setQuery({ ...where, query: "Nueva" });
    await vi.waitFor(() => expect(menu.choices).toHaveLength(2));
    expect(menu.choices[1]).toEqual({ create: "Nueva" });
    expect(menu.visible).toBe(true);
  });

  it("does not offer to create when a title matches", async () => {
    const menu = new LinkSuggest(
      () => ({
        search: async () => [{ id: "n1", title: "nueva" }],
        create: async () => null,
      }),
      () => null,
    );
    menu.setQuery({ ...where, query: "Nueva" });
    await vi.waitFor(() => expect(menu.choices).toHaveLength(1));
  });

  it("inserts the link of the chosen note", async () => {
    const insertNoteLink = vi.fn();
    const menu = new LinkSuggest(
      () => ({ search: async () => [], create: async () => "new-id" }),
      () => ({ insertNoteLink }) as unknown as NoteEditorHandle,
    );
    await menu.pick({ create: "Algo" });
    expect(insertNoteLink).toHaveBeenCalledWith("new-id");
  });
});
