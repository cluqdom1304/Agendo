// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import { TagsStore } from "../../stores/tags.svelte";
import { contextMenu } from "../context/context.svelte";
import FolderSidebar from "./FolderSidebar.svelte";
import NoteList from "./NoteList.svelte";
import { FakeNotesApi, folder, note } from "./testing";

// jsdom has no layout engine: the list only needs the observer to exist.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const emptyTags = {
  tagList: async () => [],
  tagUpsert: async () => {
    throw new Error("unused");
  },
  tagDelete: async () => undefined,
  tagMerge: async () => undefined,
};

async function mount(setup: (api: FakeNotesApi) => void = () => undefined) {
  const api = new FakeNotesApi();
  setup(api);
  const notes = new NotesStore(api);
  await notes.load();
  return { api, notes };
}

describe("FolderSidebar", () => {
  it("lists the smart folders and the tree with counts", async () => {
    const { notes } = await mount((api) => {
      api.folders = [folder("Grado", null, 3), folder("Mates", "Grado", 1)];
    });
    render(FolderSidebar, {
      notes,
      tags: new TagsStore(emptyTags),
      open: true,
    });
    expect(screen.getByText("Todas las notas")).toBeTruthy();
    expect(screen.getByText("Bandeja de entrada")).toBeTruthy();
    expect(screen.getByText("Fijadas")).toBeTruthy();
    expect(screen.getByText("Papelera")).toBeTruthy();
    expect(screen.getByText("Grado")).toBeTruthy();
    expect(screen.queryByText("Mates")).toBeNull();
    await fireEvent.click(
      screen.getByRole("button", { name: "Mostrar subcarpetas de «Grado»" }),
    );
    expect(screen.getByText("Mates")).toBeTruthy();
  });

  it("selects a folder and marks it as current", async () => {
    const { notes } = await mount((api) => {
      api.folders = [folder("Grado")];
    });
    render(FolderSidebar, {
      notes,
      tags: new TagsStore(emptyTags),
      open: true,
    });
    await fireEvent.click(screen.getByText("Grado"));
    await waitFor(() => expect(notes.scope).toBe("folder:Grado"));
    await fireEvent.click(screen.getByText("Papelera"));
    await waitFor(() => expect(notes.scope).toBe("trash"));
  });

  it("creates a folder from the header with Enter", async () => {
    const { notes } = await mount();
    render(FolderSidebar, {
      notes,
      tags: new TagsStore(emptyTags),
      open: true,
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Nueva carpeta" }),
    );
    const input = screen.getByRole("textbox", { name: "Nombre de la carpeta" });
    await fireEvent.input(input, { target: { value: "Ideas" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() =>
      expect(notes.folders.map((f) => f.name)).toEqual(["Ideas"]),
    );
  });

  it("answers the context menu: rename, new subfolder and delete ask for their fields", async () => {
    const { notes } = await mount((api) => {
      api.folders = [folder("Grado")];
    });
    render(FolderSidebar, {
      notes,
      tags: new TagsStore(emptyTags),
      open: true,
    });
    contextMenu.request({ kind: "folder", mode: "rename", id: "Grado" });
    const rename = await screen.findByRole("textbox", {
      name: "Nombre de la carpeta",
    });
    expect((rename as HTMLInputElement).value).toBe("Grado");
    expect(contextMenu.intent).toBeNull();
    await fireEvent.keyDown(rename, { key: "Escape" });

    contextMenu.request({ kind: "folder", mode: "new", id: "Grado" });
    await screen.findByRole("textbox", { name: "Nombre de la carpeta" });
    expect(notes.expanded.has("Grado")).toBe(true);
    await fireEvent.keyDown(
      screen.getByRole("textbox", { name: "Nombre de la carpeta" }),
      { key: "Escape" },
    );

    contextMenu.request({ kind: "folder", mode: "delete", id: "Grado" });
    await screen.findByRole("button", { name: "Eliminar «Grado»" });
  });

  it("asks before deleting a folder and then deletes it", async () => {
    const { notes } = await mount((api) => {
      api.folders = [folder("Grado")];
    });
    render(FolderSidebar, {
      notes,
      tags: new TagsStore(emptyTags),
      open: true,
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Eliminar «Grado»" }),
    );
    expect(notes.folders).toHaveLength(1);
    await fireEvent.click(
      screen.getByRole("button", { name: "Eliminar «Grado»" }),
    );
    await waitFor(() => expect(notes.folders).toHaveLength(0));
  });
});

describe("NoteList", () => {
  const tags = new TagsStore(emptyTags);

  it("shows each note with its title and opens it on click", async () => {
    const { notes } = await mount((api) => {
      api.notes = [note("n1", { title: "Apuntes" }), note("n2", { title: "" })];
    });
    render(NoteList, { notes, tags, onCreate: () => undefined });
    expect(screen.getByText("2 notas")).toBeTruthy();
    expect(screen.getByText("Sin título")).toBeTruthy();
    await fireEvent.click(
      screen.getByRole("option", { name: "Abrir «Apuntes»" }),
    );
    await waitFor(() => expect(notes.open?.id).toBe("n1"));
  });

  it("offers to create the first note when the list is empty", async () => {
    const { notes } = await mount();
    let created = 0;
    render(NoteList, { notes, tags, onCreate: () => (created += 1) });
    expect(screen.getByText("Aquí no hay notas")).toBeTruthy();
    await fireEvent.click(
      screen.getAllByRole("button", { name: /Nueva nota/ }).at(-1)!,
    );
    expect(created).toBe(1);
  });

  it("only renders the rows near the viewport of a long list", async () => {
    const { notes } = await mount((api) => {
      api.notes = Array.from({ length: 5000 }, (_, i) => note(`n${i}`));
    });
    render(NoteList, { notes, tags, onCreate: () => undefined });
    expect(screen.getAllByRole("option").length).toBeLessThan(60);
  });

  it("lets the trash restore a note instead of creating one", async () => {
    const { notes } = await mount((api) => {
      api.trash = [note("old", { title: "Vieja" })];
    });
    await notes.selectScope("trash");
    render(NoteList, { notes, tags, onCreate: () => undefined });
    expect(screen.queryByRole("button", { name: "Nueva nota" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Restaurar" }));
    await waitFor(() => expect(notes.notes.map((n) => n.id)).toEqual([]));
  });
});
