import { describe, expect, it, vi } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import { TagsStore } from "../../stores/tags.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeApi } from "../../testing";
import { FakeNotesApi, note } from "../notes/testing";
import { buildCommands } from "./commands";

function setup() {
  const settings = new Map<string, string>();
  const view = new ViewStore({
    api: {
      settingsGet: async (key) => settings.get(key) ?? null,
      settingsSet: async (key, value) => void settings.set(key, value),
    },
    now: () => new Date("2026-03-10T10:00:00"),
    firstDay: () => 1,
  });
  const notesApi = new FakeNotesApi();
  notesApi.notes = [note("a")];
  const notes = new NotesStore(notesApi);
  const tasks = new TasksStore({ api: new FakeApi([]), settleMs: () => 0 });
  const tags = new TagsStore({
    tagList: async () => [],
    tagUpsert: async () => {
      throw new Error("unused");
    },
    tagDelete: async () => undefined,
    tagMerge: async () => undefined,
  });
  const commands = buildCommands({ view, notes, tasks, tags });
  const run = (id: string) => {
    const command = commands.find((candidate) => candidate.id === id);
    if (!command) throw new Error(`no command ${id}`);
    command.run();
  };
  return { view, notes, tasks, tags, commands, run, notesApi };
}

describe("palette commands", () => {
  it("has unique ids and shows the shortcut the app really has", () => {
    const { commands } = setup();
    const ids = commands.map((command) => command.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(commands.find((c) => c.id === "new-task")?.shortcut).toBe("Ctrl+N");
    expect(commands.find((c) => c.id === "go-notes")?.shortcut).toBe(
      "Ctrl+Mayús+2",
    );
  });

  it("changes section and view", () => {
    const { view, run } = setup();
    run("go-notes");
    expect(view.section).toBe("notes");
    run("view-week");
    expect(view.section).toBe("calendar");
    expect(view.mode).toBe("week");
    run("go-agenda");
    expect(view.section).toBe("tasks");
  });

  it("starts a task draft in the agenda", () => {
    const { view, tasks, run } = setup();
    view.setSection("notes");
    run("new-task");
    expect(view.section).toBe("tasks");
    expect(tasks.draft).not.toBeNull();
  });

  it("creates a note and opens the smart folders in the notes", async () => {
    const { view, notes, run, notesApi } = setup();
    run("new-note");
    await vi.waitFor(() => expect(notes.open).not.toBeNull());
    expect(view.section).toBe("notes");
    expect(notesApi.notes).toHaveLength(2);

    run("notes-orphans");
    await vi.waitFor(() => expect(notes.scope).toBe("orphans"));
  });

  it("shows every tag again", () => {
    const { tags, run } = setup();
    tags.hideAll();
    run("show-all-tags");
    expect(tags.filtered).toBe(false);
  });
});
