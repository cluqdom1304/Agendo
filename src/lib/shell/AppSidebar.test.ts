// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { FakeNotesApi, folder, note, summary } from "../features/notes/testing";
import { NotesStore } from "../stores/notes.svelte";
import { TagsStore } from "../stores/tags.svelte";
import { TasksStore } from "../stores/tasks.svelte";
import type { Section } from "../stores/view.svelte";
import { FakeApi } from "../testing";
import type { NotesListQuery } from "../types";
import AppSidebar from "./AppSidebar.svelte";

/** Answers the pinned scope with the pinned notes only, like the backend. */
class PinnedNotesApi extends FakeNotesApi {
  override async notesList(query: NotesListQuery) {
    const all = await super.notesList(query);
    return query.scope === "pinned"
      ? this.notes.filter((n) => n.isPinned).map(summary)
      : all;
  }
}

const emptyTags = {
  tagList: async () => [],
  tagUpsert: async () => {
    throw new Error("unused");
  },
  tagDelete: async () => undefined,
  tagMerge: async () => undefined,
};

function mount(options: { open?: boolean; section?: Section } = {}) {
  const api = new PinnedNotesApi();
  api.notes = [
    note("plan", { title: "Plan de viaje", isPinned: true }),
    note("ideas", { title: "Ideas" }),
  ];
  api.folders = [folder("Trabajo")];
  api.settings.set("notes_recent", "ideas,plan");
  const notes = new NotesStore(api);
  const tasks = new TasksStore({ api: new FakeApi([]), settleMs: () => 0 });
  const onSection = vi.fn();
  const onOpenNote = vi.fn();
  render(AppSidebar, {
    open: options.open ?? true,
    section: options.section ?? "today",
    notes,
    tags: new TagsStore(emptyTags),
    tasks,
    onSection,
    onOpenNote,
    idleMs: 0,
  });
  return { notes, onSection, onOpenNote };
}

describe("AppSidebar (F19-02)", () => {
  it("lists the sections, marks the current one and switches on click", async () => {
    const { onSection } = mount({ section: "calendar" });
    const current = screen.getByRole("button", { name: "Calendario" });
    expect(current.getAttribute("aria-current")).toBe("page");
    expect(
      screen.getByRole("button", { name: "Hoy" }).getAttribute("aria-current"),
    ).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Notas" }));
    expect(onSection).toHaveBeenCalledWith("notes");
  });

  it("is hidden from assistive technology while folded", () => {
    mount({ open: false });
    expect(document.querySelector<HTMLElement>("aside")?.inert).toBe(true);
  });

  it("shows the pinned notes as favourites and the other recent notes, and opens them", async () => {
    const { onOpenNote } = mount();
    const favorite = await screen.findByRole("button", {
      name: "Plan de viaje",
    });
    const recent = await screen.findByRole("button", { name: "Ideas" });
    // A favourite is not repeated among the recent notes.
    expect(
      screen.getAllByRole("button", { name: "Plan de viaje" }),
    ).toHaveLength(1);
    await fireEvent.click(favorite);
    expect(onOpenNote).toHaveBeenCalledWith("plan");
    await fireEvent.click(recent);
    expect(onOpenNote).toHaveBeenCalledWith("ideas");

    await fireEvent.click(
      screen.getByRole("button", { name: "Plegar Favoritos" }),
    );
    expect(screen.queryByRole("button", { name: "Plan de viaje" })).toBeNull();
  });

  it("shows the folders everywhere and goes to the notes when one is chosen", async () => {
    const { notes, onSection } = mount({ section: "tasks" });
    await fireEvent.click(await screen.findByText("Trabajo"));
    expect(onSection).toHaveBeenCalledWith("notes");
    await waitFor(() => expect(notes.scope).toBe("folder:Trabajo"));
    // Smart folders and the trash belong to the notes section.
    expect(screen.queryByText("Papelera")).toBeNull();
    expect(screen.getByRole("heading", { name: "Etiquetas" })).toBeTruthy();
  });

  it("in the notes it shows the smart folders and leaves the task filter out", async () => {
    mount({ section: "notes" });
    expect(await screen.findByText("Papelera")).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Etiquetas" })).toBeNull();
  });

  it("takes the dark chrome tokens, so its text stays light in the light theme", () => {
    mount({ section: "today" });
    expect(
      screen.getByRole("complementary", { name: "Barra lateral" }).className,
    ).toContain("chrome-dark");
  });
});
