import { describe, expect, it, vi } from "vitest";

import type { FoundItem } from "../../types";
import {
  buildGroups,
  matchCommands,
  planQuery,
  type PaletteCommand,
} from "./palette";

const command = (id: string, label: string): PaletteCommand => ({
  id,
  label,
  run: vi.fn(),
});

const commands = [
  command("a", "Nueva tarea"),
  command("b", "Nueva nota"),
  command("c", "Ir a las notas"),
  command("d", "Agenda: vista de mes"),
];

const note = (noteId: string, title: string): FoundItem => ({
  kind: "note",
  noteId,
  title,
  snippet: "",
  folderId: null,
  isPinned: false,
});
const task = (taskId: string, title: string): FoundItem => ({
  kind: "task",
  taskId,
  title,
  snippet: "",
  dueDate: null,
  isCompleted: false,
});
const tag = (tagId: string, name: string): FoundItem => ({
  kind: "tag",
  tagId,
  name,
  colorHex: "#4C8DFF",
  noteCount: 1,
  pendingCount: 2,
});

const ids = (groups: ReturnType<typeof buildGroups>) =>
  groups.map((group) => [
    group.id,
    group.rows.map((row) =>
      row.kind === "command" ? row.command.id : (row as { id: string }).id,
    ),
  ]);

describe("planQuery", () => {
  it("searches what is typed, or only commands after >", () => {
    expect(planQuery("  álgebra ", "all")).toEqual({
      commandsOnly: false,
      text: "álgebra",
      search: "álgebra",
    });
    expect(planQuery("", "all").search).toBeNull();
    expect(planQuery(" > nueva", "all")).toEqual({
      commandsOnly: true,
      text: "nueva",
      search: null,
    });
  });

  it("go-to mode only looks for notes, even before anything is typed", () => {
    expect(planQuery("", "notes").search).toBe("is:nota");
    expect(planQuery("tag:x álg", "notes").search).toBe("is:nota tag:x álg");
  });
});

describe("matchCommands", () => {
  it("ignores case and accents, prefixes first", () => {
    expect(matchCommands(commands, "NUEVA").map((c) => c.id)).toEqual([
      "a",
      "b",
    ]);
    expect(matchCommands(commands, "notas").map((c) => c.id)).toEqual(["c"]);
    expect(matchCommands(commands, "mes").map((c) => c.id)).toEqual(["d"]);
    expect(matchCommands(commands, "", 2)).toHaveLength(2);
    expect(matchCommands(commands, "zzz")).toEqual([]);
  });
});

describe("buildGroups", () => {
  const recent = [
    { id: "r1", title: "Reciente 1" },
    { id: "r2", title: "Reciente 2" },
  ];

  it("shows the recent notes and every command before anything is typed", () => {
    const groups = buildGroups({
      mode: "all",
      plan: planQuery("", "all"),
      commands,
      recent,
      found: [],
    });
    expect(ids(groups)).toEqual([
      ["recent", ["r1", "r2"]],
      ["commands", ["a", "b", "c", "d"]],
    ]);
  });

  it("puts the matching commands first, then notes, tasks and tags", () => {
    const groups = buildGroups({
      mode: "all",
      plan: planQuery("nueva", "all"),
      commands,
      recent,
      found: [tag("g", "Nuevas"), task("t", "Nueva idea"), note("n", "Nueva")],
    });
    expect(ids(groups)).toEqual([
      ["commands", ["a", "b"]],
      ["notes", ["n"]],
      ["tasks", ["t"]],
      ["tags", ["g"]],
    ]);
  });

  it("with > lists only commands and leaves out empty groups", () => {
    const groups = buildGroups({
      mode: "all",
      plan: planQuery(">agenda", "all"),
      commands,
      recent,
      found: [note("n", "Agenda")],
    });
    expect(ids(groups)).toEqual([["commands", ["d"]]]);
    expect(
      buildGroups({
        mode: "all",
        plan: planQuery("zzz", "all"),
        commands,
        recent,
        found: [],
      }),
    ).toEqual([]);
  });

  it("go-to mode lists recent notes then the others, without repeating", () => {
    const groups = buildGroups({
      mode: "notes",
      plan: planQuery("", "notes"),
      commands,
      recent,
      found: [
        note("r2", "Reciente 2"),
        note("x", "Otra"),
        tag("g", "Etiqueta"),
      ],
    });
    expect(ids(groups)).toEqual([
      ["recent", ["r1", "r2"]],
      ["notes", ["x"]],
    ]);
  });

  it("go-to mode shows only the matching notes once something is typed", () => {
    const groups = buildGroups({
      mode: "notes",
      plan: planQuery("otra", "notes"),
      commands,
      recent,
      found: [note("x", "Otra")],
    });
    expect(ids(groups)).toEqual([["notes", ["x"]]]);
  });
});
