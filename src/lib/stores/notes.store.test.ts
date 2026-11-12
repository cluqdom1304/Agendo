import { describe, expect, it } from "vitest";

import {
  FakeNotesApi as FakeApi,
  folder,
  note,
  summary,
} from "../features/notes/testing";
import type { NoteSummary } from "../types";
import { NotesStore } from "./notes.svelte";

async function loaded(setup: (api: FakeApi) => void = () => undefined) {
  const api = new FakeApi();
  setup(api);
  const store = new NotesStore(api);
  await store.load();
  return { api, store };
}

describe("NotesStore", () => {
  it("loads folders and every note by default", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("a")];
      api.notes = [note("n1")];
    });
    expect(store.status).toBe("ready");
    expect(store.scope).toBe("all");
    expect(store.notes.map((n) => n.id)).toEqual(["n1"]);
  });

  it("restores the scope and the open note of the last run", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("root"), folder("kid", "root")];
      api.notes = [note("n1", { folderId: "kid" })];
      api.settings.set("notes_scope", "folder:kid");
      api.settings.set("notes_open", "n1");
    });
    expect(store.scope).toBe("folder:kid");
    expect(store.open?.id).toBe("n1");
    expect(store.expanded.has("root")).toBe(true);
  });

  it("falls back when the saved folder or note no longer exists", async () => {
    const { api, store } = await loaded((fake) => {
      fake.settings.set("notes_scope", "folder:gone");
      fake.settings.set("notes_open", "gone");
    });
    expect(store.scope).toBe("all");
    expect(store.open).toBeNull();
    expect(store.status).toBe("ready");
    expect(api.settings.get("notes_open")).toBe("");
  });

  it("lists with the query of the scope, including subfolders of a folder", async () => {
    const { api, store } = await loaded((fake) => {
      fake.folders = [folder("a")];
    });
    await store.selectScope("folder:a");
    await store.selectScope("inbox");
    await store.selectScope("pinned");
    expect(api.queries.slice(1)).toEqual([
      { scope: "folder", folderId: "a", sort: "updated" },
      { scope: "inbox", sort: "updated" },
      { scope: "pinned", sort: "updated" },
    ]);
    expect(api.settings.get("notes_scope")).toBe("pinned");
  });

  it("shows the trash with its own list", async () => {
    const { store } = await loaded((api) => {
      api.trash = [note("old")];
    });
    await store.selectScope("trash");
    expect(store.notes.map((n) => n.id)).toEqual(["old"]);
  });

  it("creates a note inside the folder being shown and opens it", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("a")];
    });
    await store.selectScope("folder:a");
    const created = await store.createNote();
    expect(created?.folderId).toBe("a");
    expect(store.open?.id).toBe(created?.id);
    expect(store.notes).toHaveLength(1);
  });

  it("creates from pinned or trash in the full list", async () => {
    const { store } = await loaded();
    await store.selectScope("trash");
    await store.createNote();
    expect(store.scope).toBe("all");
  });

  it("keeps the open note in step with saves", async () => {
    const { store } = await loaded((api) => {
      api.notes = [note("n1")];
    });
    await store.openNote("n1");
    await store.saveTitle("n1", "Nuevo");
    await store.saveContent("n1", '{"type":"doc","content":[1]}');
    expect(store.open?.title).toBe("Nuevo");
    expect(store.open?.contentJson).toContain("1");
    expect(store.notes[0]?.title).toBe("Nuevo");
  });

  it("rejects a failed save so the saver can retry", async () => {
    const { api, store } = await loaded((fake) => {
      fake.notes = [note("n1")];
    });
    api.failWith = "DB: fallo";
    await expect(store.saveTitle("n1", "x")).rejects.toBe("DB: fallo");
  });

  it("pins, moves and duplicates", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("a")];
      api.notes = [note("n1", { title: "Idea" })];
    });
    await store.openNote("n1");
    await store.setPinned("n1", true);
    expect(store.open?.isPinned).toBe(true);
    await store.moveNote("n1", "a");
    expect(store.open?.folderId).toBe("a");
    const copy = await store.duplicateNote("n1");
    expect(copy?.title).toBe("Copia de Idea");
    expect(copy?.folderId).toBe("a");
    expect(store.open?.id).toBe(copy?.id);
  });

  it("trashes a note, closes it and brings it back with undo", async () => {
    const { store } = await loaded((api) => {
      api.notes = [note("n1")];
    });
    await store.openNote("n1");
    await store.trashNote("n1");
    expect(store.open).toBeNull();
    expect(store.notes).toHaveLength(0);
    expect(store.undoStack).toEqual(["n1"]);
    await store.undoTrash();
    expect(store.notes.map((n) => n.id)).toEqual(["n1"]);
    expect(store.undoStack).toEqual([]);
  });

  it("flattens the tree honouring what is expanded", async () => {
    const { store } = await loaded((api) => {
      api.folders = [
        folder("a"),
        folder("b", "a"),
        folder("c", "b"),
        folder("z"),
      ];
    });
    expect(store.rows.map((r) => r.folder.id)).toEqual(["a", "z"]);
    expect(store.rows[0]?.hasChildren).toBe(true);
    store.toggleExpanded("a");
    store.toggleExpanded("b");
    expect(store.rows.map((r) => [r.folder.id, r.depth])).toEqual([
      ["a", 0],
      ["b", 1],
      ["c", 2],
      ["z", 0],
    ]);
  });

  it("creates, renames and moves folders", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("a"), folder("b")];
    });
    expect(await store.createFolder("Nueva", "a")).toBe(true);
    expect(store.expanded.has("a")).toBe(true);
    expect(await store.renameFolder("b", "Beta")).toBe(true);
    expect(store.byId.get("b")?.name).toBe("Beta");
    expect(await store.moveFolder("b", "a")).toBe(true);
    expect(store.byId.get("b")?.parentId).toBe("a");
    expect(await store.moveFolder("b", "a")).toBe(false);
  });

  it("explains a repeated or invalid folder name", async () => {
    const { api, store } = await loaded();
    api.failWith = "CONFLICT: ya existe";
    expect(await store.createFolder("x")).toBe(false);
    expect(store.notice?.message).toMatch(/Ya existe/);
    api.failWith = "VALIDATION: nombre";
    await store.createFolder("");
    expect(store.notice?.message).toMatch(/no es válido/);
  });

  it("deleting a folder sends its notes to the trash and leaves the view", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("a")];
      api.notes = [note("n1", { folderId: "a" })];
      api.settings.set("notes_scope", "folder:a");
      api.settings.set("notes_open", "n1");
    });
    expect(store.open?.id).toBe("n1");
    await store.deleteFolder("a");
    expect(store.scope).toBe("all");
    expect(store.open).toBeNull();
    expect(store.byId.has("a")).toBe(false);
    expect(store.notice?.message).toMatch(/papelera/);
  });

  it("a slow answer never overwrites a newer list", async () => {
    const { api, store } = await loaded();
    let release: (value: NoteSummary[]) => void = () => undefined;
    const original = api.notesList.bind(api);
    api.notesList = (query) =>
      query.scope === "inbox"
        ? new Promise<NoteSummary[]>((resolve) => (release = resolve))
        : original(query);
    const slow = store.selectScope("inbox");
    api.notes = [note("fresh")];
    await store.selectScope("all");
    release([summary(note("stale"))]);
    await slow;
    expect(store.notes.map((n) => n.id)).toEqual(["fresh"]);
  });

  it("reports a failed load", async () => {
    const api = new FakeApi();
    api.folderList = async () => {
      throw "DB: fallo";
    };
    const store = new NotesStore(api);
    await store.load();
    expect(store.status).toBe("error");
  });
});
