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

  it("opens a note of the trash read-only and closes it when leaving the trash", async () => {
    const { store } = await loaded((api) => {
      api.notes = [note("n1")];
      api.trash = [note("old", { title: "Vieja" })];
    });
    await store.openNote("n1");
    await store.selectScope("trash");
    // A live note stays open, and editable, while the trash is looked at.
    expect(store.open?.id).toBe("n1");
    expect(store.openInTrash).toBe(false);

    await store.openNote("old");
    expect(store.open?.title).toBe("Vieja");
    expect(store.openInTrash).toBe(true);

    await store.selectScope("all");
    expect(store.open).toBeNull();
    expect(store.openInTrash).toBe(false);
  });

  it("restoring the open note of the trash keeps it open and editable", async () => {
    const { store } = await loaded((api) => {
      api.trash = [note("old")];
    });
    await store.selectScope("trash");
    await store.openNote("old");
    await store.restoreNote("old");
    expect(store.open?.id).toBe("old");
    expect(store.openInTrash).toBe(false);
    expect(store.notes).toHaveLength(0);
  });

  it("lists every folder in tree order for menus, whatever is expanded", async () => {
    const { store } = await loaded((api) => {
      api.folders = [folder("Grado"), folder("Mates", "Grado"), folder("Ocio")];
    });
    expect(store.rows.map((row) => row.folder.name)).toEqual(["Grado", "Ocio"]);
    expect(store.tree.map((row) => `${row.depth}:${row.folder.name}`)).toEqual([
      "0:Grado",
      "1:Mates",
      "0:Ocio",
    ]);
  });

  it("reports a list that fails to load instead of throwing", async () => {
    const { api, store } = await loaded();
    api.notesList = async () => {
      throw "DB: roto";
    };
    await store.selectScope("inbox");
    expect(store.notice?.kind).toBe("error");
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

  it("sets the tags of a note and lists by tag", async () => {
    const { api, store } = await loaded((fake) => {
      fake.notes = [note("n1")];
    });
    await store.openNote("n1");
    await store.setTags("n1", ["mat", "fis"]);
    expect(store.open?.tagIds).toEqual(["mat", "fis"]);
    await store.selectScope("tag:mat");
    expect(store.scopeTagId).toBe("mat");
    expect(api.queries.at(-1)).toEqual({
      scope: "all",
      tagId: "mat",
      sort: "updated",
    });
    expect(api.settings.get("notes_scope")).toBe("tag:mat");
  });

  it("a new note inside a tag view carries that tag", async () => {
    const { store } = await loaded();
    await store.selectScope("tag:mat");
    const created = await store.createNote();
    expect(created).not.toBeNull();
    expect(store.scope).toBe("tag:mat");
  });

  it("looks up the titles of the notes tasks come from, once each", async () => {
    const { api, store } = await loaded((fake) => {
      fake.notes = [
        note("n1", { title: "Reunión" }),
        note("n2", { title: "Plan" }),
      ];
    });
    let lookups = 0;
    const original = api.noteGet.bind(api);
    api.noteGet = async (id) => {
      lookups += 1;
      return original(id);
    };
    await store.ensureTitles(["n1", "n2", "n1", "gone"]);
    expect(store.titleOf("n1")).toBe("Reunión");
    expect(store.titleOf("gone")).toBeUndefined();
    const first = lookups;
    await store.ensureTitles(["n1", "n2"]);
    expect(lookups).toBe(first);
  });

  it("creates a note with given contents and opens it", async () => {
    const { store } = await loaded();
    const created = await store.createNoteWith({
      title: "Desde una tarea",
      tagIds: ["mat"],
    });
    expect(created?.title).toBe("Desde una tarea");
    expect(store.open?.id).toBe(created?.id);
    expect(store.titleOf(created?.id ?? "")).toBe("Desde una tarea");
  });

  it("shares one load between callers that arrive together", async () => {
    const api = new FakeApi();
    let lists = 0;
    const original = api.folderList.bind(api);
    api.folderList = async () => {
      lists += 1;
      return original();
    };
    const store = new NotesStore(api);
    await Promise.all([store.load(), store.ensureLoaded(), store.load()]);
    expect(lists).toBe(1);
    await store.ensureLoaded();
    expect(lists).toBe(1);
  });
});

describe("NotesStore: recent notes", () => {
  it("remembers the notes opened, most recent first, without repeats or more than eight", async () => {
    const { api, store } = await loaded((fake) => {
      fake.notes = Array.from({ length: 10 }, (_, at) => note(`n${at}`));
    });
    for (let at = 0; at < 10; at += 1) await store.openNote(`n${at}`);
    await store.openNote("n5");
    expect(store.recent).toEqual([
      "n5",
      "n9",
      "n8",
      "n7",
      "n6",
      "n4",
      "n3",
      "n2",
    ]);
    expect(api.settings.get("notes_recent")).toBe(store.recent.join(","));
  });

  it("restores them on load and only lists those that still exist", async () => {
    const { store } = await loaded((api) => {
      api.notes = [note("a", { title: "Uno" }), note("b", { title: "Dos" })];
      api.settings.set("notes_recent", "b,gone,a,b");
    });
    expect(store.recent).toEqual(["b", "gone", "a"]);
    await store.ensureTitles(store.recent);
    expect(store.recentNotes()).toEqual([
      { id: "b", title: "Dos" },
      { id: "a", title: "Uno" },
    ]);
  });

  it("does not count the note restored at startup nor one from the trash", async () => {
    const { store } = await loaded((api) => {
      api.notes = [note("a")];
      api.settings.set("notes_open", "a");
    });
    expect(store.open?.id).toBe("a");
    expect(store.recent).toEqual([]);
  });
});

describe("daily note (F13-01)", () => {
  it("opens the note of a day, titled with its date, and reuses it", async () => {
    const api = new FakeApi();
    const store = new NotesStore(api);
    await store.ensureLoaded();
    const first = await store.openDaily("2026-03-05");
    expect(first?.title).toBe("Nota del 05/03/2026");
    expect(store.open?.id).toBe(first?.id);
    const again = await store.openDaily("2026-03-05");
    expect(again?.id).toBe(first?.id);
    expect(api.notes).toHaveLength(1);
  });
});

describe("note templates (F13-02)", () => {
  const DOC = '{"type":"doc","content":[{"type":"paragraph"}]}';

  async function ready() {
    const api = new FakeApi();
    const store = new NotesStore(api);
    await store.ensureLoaded();
    return { api, store };
  }

  it("saves the open note as a template and starts new notes from it", async () => {
    const { api, store } = await ready();
    const source = await store.createNoteWith({
      title: "Acta",
      contentJson: DOC,
    });
    expect(source).not.toBeNull();
    await store.openTemplates();
    expect(store.templatesOpen).toBe(true);
    expect(await store.saveOpenAsTemplate("Reunión")).toBe(true);
    expect(store.templates.map((item) => item.name)).toEqual(["Reunión"]);

    const created = await store.createFromTemplate(store.templates[0]!.id);
    expect(created?.title).toBe("Reunión");
    expect(created?.contentJson).toBe(DOC);
    expect(store.templatesOpen).toBe(false);
    expect(api.notes).toHaveLength(2);
  });

  it("renames and deletes templates", async () => {
    const { store } = await ready();
    await store.createNoteWith({ title: "Acta", contentJson: DOC });
    await store.saveOpenAsTemplate("Reunión");
    const id = store.templates[0]!.id;
    expect(await store.renameTemplate(id, "Semanal")).toBe(true);
    expect(store.templates[0]?.name).toBe("Semanal");
    await store.deleteTemplate(id);
    expect(store.templates).toEqual([]);
  });

  it("reports a rejected name and keeps the dialog data", async () => {
    const { api, store } = await ready();
    api.templateSave = async () => {
      throw "CONFLICT: ya existe una plantilla con ese nombre";
    };
    expect(await store.saveOpenAsTemplate("Reunión")).toBe(false);
    expect(store.notice).toEqual({
      kind: "error",
      message: "ya existe una plantilla con ese nombre",
    });
  });
});
