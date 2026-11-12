// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import { installLayoutStubs } from "../../testing";
import { noteLinkIds } from "./content";
import type { NoteEditorHandle, NoteLinkHost, TagQuery } from "./editor";
import NoteBacklinks from "./NoteBacklinks.svelte";
import NoteEditor from "./NoteEditor.svelte";
import { FakeNotesApi, note } from "./testing";

installLayoutStubs();

afterEach(() => vi.useRealTimers());

const link = (noteId: string) => ({ type: "noteLink", attrs: { noteId } });
const para = (...content: object[]) => ({ type: "paragraph", content });
const text = (value: string) => ({ type: "text", text: value });
const doc = (...content: object[]) => JSON.stringify({ type: "doc", content });

function makeHost(initial: Record<string, string | null | undefined>) {
  const titles = { ...initial };
  const host = {
    titles,
    open: vi.fn<(id: string) => void>(),
    lookup: (id: string) => titles[id],
  };
  return host satisfies NoteLinkHost & { titles: typeof titles };
}

async function open(
  contentJson: string,
  host: NoteLinkHost,
  onLinkQuery?: (query: TagQuery | null) => void,
) {
  const { createNoteEditor } = await import("./editor");
  const element = document.createElement("div");
  document.body.append(element);
  const changes: string[] = [];
  const handle: NoteEditorHandle = createNoteEditor({
    element,
    contentJson,
    label: "nota",
    placeholder: "",
    onChange: (json) => changes.push(json),
    onState: () => {},
    onLinkShortcut: () => {},
    onOpenLink: () => {},
    linkHost: host,
    ...(onLinkQuery ? { onLinkQuery } : {}),
  });
  return { element, handle, changes };
}

describe("links between notes in the editor", () => {
  it("draws the title read live, so a rename shows without touching the document", async () => {
    const host = makeHost({ a: "Álgebra" });
    const { element, handle, changes } = await open(
      doc(para(text("ver "), link("a"))),
      host,
    );
    expect(element.querySelector(".note-link")?.textContent).toBe("Álgebra");

    host.titles.a = "Álgebra lineal";
    handle.refreshNoteLinks();
    expect(element.querySelector(".note-link")?.textContent).toBe(
      "Álgebra lineal",
    );
    expect(changes).toEqual([]);
    handle.destroy();
  });

  it("opens the note on click and shows a removed note as such", async () => {
    const host = makeHost({ a: "Álgebra", b: null, c: undefined });
    const { element, handle } = await open(
      doc(para(link("a"), text(" "), link("b"), text(" "), link("c"))),
      host,
    );
    const [first, gone, loading] = [
      ...element.querySelectorAll<HTMLElement>(".note-link"),
    ];
    first?.click();
    expect(host.open).toHaveBeenCalledWith("a");

    expect(gone?.textContent).toBe("Nota eliminada");
    expect(gone?.classList).toContain("is-gone");
    gone?.click();
    expect(host.open).toHaveBeenCalledTimes(1);

    expect(loading?.classList).toContain("is-loading");
    handle.destroy();
  });

  it("reports the [[text being typed and replaces it with a link", async () => {
    const host = makeHost({ a: "Álgebra" });
    const queries: (TagQuery | null)[] = [];
    const { element, handle, changes } = await open(
      doc(para(text("ver [[alg"))),
      host,
      (query) => queries.push(query),
    );
    handle.focus("end");
    expect(queries.at(-1)?.query).toBe("alg");

    handle.insertNoteLink("a");
    expect(noteLinkIds(changes.at(-1) ?? "")).toEqual(["a"]);
    expect(element.textContent).toBe("ver Álgebra ");
    expect(queries.at(-1)).toBeNull();
    handle.destroy();
  });
});

describe("noteLinkIds", () => {
  it("lists the linked notes once, in order, and survives bad documents", () => {
    expect(
      noteLinkIds(doc(para(link("b"), link("a")), para(link("b")))),
    ).toEqual(["b", "a"]);
    expect(noteLinkIds("")).toEqual([]);
    expect(noteLinkIds("{no")).toEqual([]);
  });
});

describe("[[ suggestions in the editor", () => {
  const setup = (found: { id: string; title: string }[]) => {
    const save = vi.fn<(json: string) => Promise<void>>(async () => {});
    const create = vi.fn<(title: string) => Promise<string | null>>(
      async () => "new",
    );
    const search = vi.fn(async () => found);
    const host = makeHost({ a: "Álgebra", new: "alg" });
    render(NoteEditor, {
      contentJson: doc(para(text("ver [[alg"))),
      save,
      autofocus: true,
      links: { host, search, create },
    });
    return { save, create, search };
  };

  it("offers the matching notes and a way to create the one being typed", async () => {
    const { search } = setup([{ id: "a", title: "Álgebra" }]);
    const list = await screen.findByRole("listbox");
    expect(search).toHaveBeenCalledWith("alg");
    expect(list.textContent).toContain("Álgebra");
    expect(list.textContent).toContain("Crear la nota «alg»");
  });

  it("does not offer to create a note that already has that title", async () => {
    setup([{ id: "a", title: "ALG" }]);
    const list = await screen.findByRole("listbox");
    expect(list.textContent).toContain("ALG");
    expect(list.textContent).not.toContain("Crear");
  });

  it("creates the note being typed and links it", async () => {
    const { save, create } = setup([{ id: "a", title: "Álgebra" }]);
    await fireEvent.mouseDown(
      await screen.findByRole("option", { name: /Crear/ }),
    );
    await waitFor(() => expect(create).toHaveBeenCalledWith("alg"));
    await waitFor(() => expect(save).toHaveBeenCalled(), { timeout: 3000 });
    expect(noteLinkIds(save.mock.calls.at(-1)?.[0] ?? "")).toEqual(["new"]);
  });
});

describe("NoteBacklinks", () => {
  const setup = () => {
    const api = new FakeNotesApi();
    api.notes = [
      note("target", { title: "Destino" }),
      note("s1", { title: "Origen uno" }),
      note("s2", { title: "Origen dos" }),
    ];
    api.links = [
      ["s1", "target"],
      ["s2", "target"],
    ];
    return new NotesStore(api);
  };

  it("lists the notes that link here and opens one", async () => {
    const notes = setup();
    render(NoteBacklinks, { notes, noteId: "target" });
    expect(await screen.findByText("Origen uno")).toBeTruthy();
    expect(screen.getByText("2 notas enlazan aquí")).toBeTruthy();
    await fireEvent.click(screen.getByText("Origen dos"));
    await waitFor(() => expect(notes.open?.id).toBe("s2"));
  });

  it("stays out of the way when nothing links to the note", async () => {
    const notes = setup();
    render(NoteBacklinks, { notes, noteId: "s1" });
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(screen.queryByRole("region")).toBeNull();
  });
});

describe("notes store: links", () => {
  it("shows a note as removed once it is trashed and again when restored", async () => {
    const api = new FakeNotesApi();
    api.notes = [note("a", { title: "Uno" })];
    const notes = new NotesStore(api);
    await notes.load();
    expect(notes.linkTitle("a")).toBe("Uno");
    expect(notes.linkTitle("zzz")).toBeUndefined();

    await notes.trashNote("a");
    expect(notes.linkTitle("a")).toBeNull();
    await notes.restoreNote("a");
    expect(notes.linkTitle("a")).toBe("Uno");

    await notes.ensureTitles(["zzz"]);
    expect(notes.linkTitle("zzz")).toBeNull();
  });

  it("suggests by title without letting filters through, and creates a linked note quietly", async () => {
    const api = new FakeNotesApi();
    api.notes = [note("a", { title: "Álgebra" }), note("b", { title: "Otra" })];
    const notes = new NotesStore(api);
    await notes.load();
    expect(await notes.suggest("Álg")).toEqual([{ id: "a", title: "Álgebra" }]);
    await notes.suggest("tag:x");
    expect(api.searches.at(-1)).toBe("is:nota tag x");

    await notes.openNote("b");
    const created = await notes.createLinkedNote("Nueva", null);
    expect(created?.title).toBe("Nueva");
    expect(notes.open?.id).toBe("b");
    expect(notes.notes.some((item) => item.id === created?.id)).toBe(true);
    expect(notes.linkTitle(created?.id ?? "")).toBe("Nueva");
  });

  it("the orphans scope lists notes with no links", async () => {
    const api = new FakeNotesApi();
    api.notes = [note("a"), note("b"), note("c")];
    api.links = [["a", "b"]];
    const notes = new NotesStore(api);
    await notes.load();
    await notes.selectScope("orphans");
    expect(notes.notes.map((item) => item.id)).toEqual(["c"]);
    expect(notes.scopeTitle).toBe("Notas sin enlaces");
  });
});
