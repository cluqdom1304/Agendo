// What the command palette (Ctrl+K) and "Ir a…" (Ctrl+P) show for what is typed: pure
// functions, so the rules are easy to test apart from the dialog.

import type { FoundItem } from "../../types";

export type PaletteMode = "all" | "notes";

export interface PaletteCommand {
  id: string;
  label: string;
  /** Shortcut shown next to the command, when it has one. */
  shortcut?: string;
  run: () => void;
}

export type PaletteRow =
  | { kind: "command"; key: string; command: PaletteCommand }
  | {
      kind: "note";
      key: string;
      id: string;
      title: string;
      snippet: string;
      pinned: boolean;
    }
  | {
      kind: "task";
      key: string;
      id: string;
      title: string;
      snippet: string;
      dueDate: string | null;
      done: boolean;
    }
  | {
      kind: "tag";
      key: string;
      id: string;
      name: string;
      color: string;
      notes: number;
      pending: number;
    };

export type GroupId = "commands" | "recent" | "notes" | "tasks" | "tags";

export interface PaletteGroup {
  id: GroupId;
  rows: PaletteRow[];
}

export interface QueryPlan {
  /** `>` at the start: only commands. */
  commandsOnly: boolean;
  /** What is left to match commands against, without the `>`. */
  text: string;
  /** What to ask the search for; `null` when nothing should be searched. */
  search: string | null;
}

const COMMANDS_SHOWN_WITH_RESULTS = 4;
const RECENT_SHOWN = 5;
const NOTES_SHOWN = 10;

export function fold(text: string): string {
  return text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();
}

export function planQuery(raw: string, mode: PaletteMode): QueryPlan {
  const trimmed = raw.trim();
  if (mode === "notes") {
    return {
      commandsOnly: false,
      text: trimmed,
      search: `is:nota ${trimmed}`.trim(),
    };
  }
  if (trimmed.startsWith(">")) {
    return { commandsOnly: true, text: trimmed.slice(1).trim(), search: null };
  }
  return {
    commandsOnly: false,
    text: trimmed,
    search: trimmed === "" ? null : trimmed,
  };
}

/** Commands whose name starts with the text first, then the ones that contain it. */
export function matchCommands(
  commands: readonly PaletteCommand[],
  text: string,
  limit = commands.length,
): PaletteCommand[] {
  const wanted = fold(text);
  if (wanted === "") return commands.slice(0, limit);
  const starts: PaletteCommand[] = [];
  const inside: PaletteCommand[] = [];
  for (const command of commands) {
    const label = fold(command.label);
    if (label.startsWith(wanted)) starts.push(command);
    else if (label.includes(wanted)) inside.push(command);
  }
  return [...starts, ...inside].slice(0, limit);
}

function commandRows(commands: readonly PaletteCommand[]): PaletteRow[] {
  return commands.map((command) => ({
    kind: "command",
    key: `command:${command.id}`,
    command,
  }));
}

function noteRow(
  id: string,
  title: string,
  snippet: string,
  pinned: boolean,
  prefix: string,
): PaletteRow {
  return { kind: "note", key: `${prefix}:${id}`, id, title, snippet, pinned };
}

function foundRows(found: readonly FoundItem[]) {
  const notes: PaletteRow[] = [];
  const tasks: PaletteRow[] = [];
  const tags: PaletteRow[] = [];
  for (const item of found) {
    if (item.kind === "note") {
      notes.push(
        noteRow(item.noteId, item.title, item.snippet, item.isPinned, "note"),
      );
    } else if (item.kind === "task") {
      tasks.push({
        kind: "task",
        key: `task:${item.taskId}`,
        id: item.taskId,
        title: item.title,
        snippet: item.snippet,
        dueDate: item.dueDate,
        done: item.isCompleted,
      });
    } else {
      tags.push({
        kind: "tag",
        key: `tag:${item.tagId}`,
        id: item.tagId,
        name: item.name,
        color: item.colorHex,
        notes: item.noteCount,
        pending: item.pendingCount,
      });
    }
  }
  return { notes, tasks, tags };
}

export interface BuildInput {
  mode: PaletteMode;
  plan: QueryPlan;
  commands: readonly PaletteCommand[];
  /** Notes opened lately, most recent first. */
  recent: readonly { id: string; title: string }[];
  found: readonly FoundItem[];
}

/** The groups of rows to draw, in order, without empty ones. */
export function buildGroups(input: BuildInput): PaletteGroup[] {
  const { mode, plan, commands, recent, found } = input;
  const groups: PaletteGroup[] = [];
  const add = (id: GroupId, rows: PaletteRow[]) => {
    if (rows.length > 0) groups.push({ id, rows });
  };
  const recentRows = recent
    .slice(0, RECENT_SHOWN + (mode === "notes" ? 3 : 0))
    .map((item) => noteRow(item.id, item.title, "", false, "recent"));
  const { notes, tasks, tags } = foundRows(found);

  if (mode === "notes") {
    if (plan.text === "") {
      const shown = new Set(recent.map((item) => item.id));
      add("recent", recentRows);
      add(
        "notes",
        notes
          .filter((row) => row.kind === "note" && !shown.has(row.id))
          .slice(0, Math.max(0, NOTES_SHOWN - recentRows.length)),
      );
    } else {
      add("notes", notes);
    }
    return groups;
  }

  if (plan.commandsOnly) {
    add("commands", commandRows(matchCommands(commands, plan.text)));
    return groups;
  }
  if (plan.text === "") {
    add("recent", recentRows);
    add("commands", commandRows(commands));
    return groups;
  }
  add(
    "commands",
    commandRows(
      matchCommands(commands, plan.text, COMMANDS_SHOWN_WITH_RESULTS),
    ),
  );
  add("notes", notes);
  add("tasks", tasks);
  add("tags", tags);
  return groups;
}
