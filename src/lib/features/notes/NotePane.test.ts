// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, installLayoutStubs } from "../../testing";
import NoteList from "./NoteList.svelte";
import NotePane from "./NotePane.svelte";
import { FakeNotesApi, folder, note } from "./testing";
import { TagsStore } from "../../stores/tags.svelte";

installLayoutStubs();

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
};

const emptyTags = new TagsStore({
  tagList: async () => [],
  tagUpsert: async () => {
    throw new Error("unused");
  },
  tagDelete: async () => undefined,
  tagMerge: async () => undefined,
});

async function mount(setup: (api: FakeNotesApi) => void) {
  const api = new FakeNotesApi();
  setup(api);
  const notes = new NotesStore(api);
  await notes.load();
  const tasks = new TasksStore({ api: new FakeApi([]), settleMs: () => 0 });
  return { api, notes, tasks };
}

function pane(notes: NotesStore, tasks: TasksStore) {
  const open = notes.open;
  if (!open) throw new Error("no note is open");
  return render(NotePane, {
    notes,
    note: open,
    tasks,
    onPreviewTask: () => undefined,
  });
}

describe("NotePane", () => {
  it("shows the format bar in its header and the note's own actions", async () => {
    const { notes, tasks } = await mount((api) => {
      api.notes = [note("n1", { title: "Apuntes" })];
    });
    await notes.openNote("n1");
    pane(notes, tasks);
    await screen.findByRole("textbox", { name: "Texto de la nota" });
    expect(screen.getByRole("toolbar")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Fijar" })).toBeTruthy();
    expect(
      (
        screen.getByRole("textbox", {
          name: "Título de la nota",
        }) as HTMLInputElement
      ).value,
    ).toBe("Apuntes");
  });

  it("moves the note from the folder menu, which lists subfolders too", async () => {
    const { notes, tasks } = await mount((api) => {
      api.folders = [folder("Grado"), folder("Mates", "Grado")];
      api.notes = [note("n1")];
    });
    await notes.openNote("n1");
    pane(notes, tasks);
    await fireEvent.click(screen.getByRole("button", { name: "Mover a…" }));
    expect(
      screen
        .getByRole("menuitemradio", { name: "Bandeja de entrada" })
        .getAttribute("aria-checked"),
    ).toBe("true");
    await fireEvent.click(screen.getByRole("menuitemradio", { name: "Mates" }));
    await waitFor(() => expect(notes.open?.folderId).toBe("Mates"));
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("deletes and duplicates from the menu of more actions", async () => {
    const { notes, tasks } = await mount((api) => {
      api.notes = [note("n1", { title: "Apuntes" })];
    });
    await notes.openNote("n1");
    pane(notes, tasks);
    await fireEvent.click(screen.getByRole("button", { name: "Más acciones" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Eliminar" }));
    await waitFor(() => expect(notes.undoStack).toEqual(["n1"]));
    expect(notes.open).toBeNull();
  });

  it("shows a note of the trash read-only, with a way back", async () => {
    const { notes, tasks } = await mount((api) => {
      api.trash = [note("old", { title: "Vieja" })];
    });
    await notes.selectScope("trash");
    await notes.openNote("old");
    pane(notes, tasks);
    const body = await screen.findByRole("textbox", {
      name: "Texto de la nota",
    });
    expect(body.getAttribute("contenteditable")).toBe("false");
    expect(screen.queryByRole("toolbar")).toBeNull();
    expect(screen.queryByRole("button", { name: "Más acciones" })).toBeNull();
    expect(
      (
        screen.getByRole("textbox", {
          name: "Título de la nota",
        }) as HTMLInputElement
      ).readOnly,
    ).toBe(true);
    await fireEvent.click(screen.getByRole("button", { name: "Restaurar" }));
    await waitFor(() => expect(notes.openInTrash).toBe(false));
    expect(notes.open?.id).toBe("old");
  });
});

describe("NoteList header", () => {
  it("changes the order from its menu", async () => {
    const { notes } = await mount((api) => {
      api.notes = [note("n1"), note("n2")];
    });
    render(NoteList, { notes, tags: emptyTags, onCreate: () => undefined });
    await fireEvent.click(screen.getByRole("button", { name: "Ordenar por" }));
    await fireEvent.click(
      screen.getByRole("menuitemradio", { name: "Título" }),
    );
    await waitFor(() => expect(notes.sort).toBe("title"));
    expect(screen.queryByRole("menu")).toBeNull();
  });
});
