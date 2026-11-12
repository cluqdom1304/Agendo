// What each right click offers (D-50). Pure functions of the stores: the menu calls them
// again whenever the state changes, so ticks and labels never go stale.

import { formatDisplayDate } from "../../dates";
import { addDays } from "../calendar/grid";
import { t } from "../../i18n";
import type { NotesStore } from "../../stores/notes.svelte";
import type { TagsStore } from "../../stores/tags.svelte";
import type { TasksStore } from "../../stores/tasks.svelte";
import type { ViewStore } from "../../stores/view.svelte";
import type { Note, NoteSummary, Task, ViewMode } from "../../types";
import type { TableAction } from "../notes/editor";
import { editorActions } from "./editorActions";
import { nodeRemovers } from "./nodeRemovers";
import type { ContextEntry, ContextMenuStore } from "./context.svelte";
import type { MenuTarget } from "./target";

export interface ContextDeps {
  view: ViewStore;
  tasks: TasksStore;
  tags: TagsStore;
  notes: NotesStore;
  menu: Pick<ContextMenuStore, "request">;
  /** Opens the creation dialog, on a day if one is given. */
  newTask(date: string | null): void;
  editTask(task: Task): void;
  /** Shows the view of one task. */
  openTask(taskId: string): void;
  /** Switches to the notes and opens the note. */
  showNote(noteId: string): Promise<void>;
  noteForTask(task: Task): Promise<Note | null>;
  taskToNote(task: Task): Promise<Note | null>;
  /** Cut, copy and paste in the editor (the clipboard belongs to the browser). */
  clipboard: {
    cut(): void;
    copy(): void;
    paste(): void;
    selectAll(): void;
  };
}

const VIEW_MODES: { mode: ViewMode; label: string }[] = [
  { mode: "list", label: t.views.list },
  { mode: "day", label: t.views.day },
  { mode: "week", label: t.views.week },
  { mode: "month", label: t.views.month },
];

/** The entries for what was pointed at; none when nothing applies, and the menu stays shut. */
export function buildEntries(
  target: MenuTarget,
  deps: ContextDeps,
): ContextEntry[] {
  switch (target.kind) {
    case "task":
      return taskEntries(target.taskId, deps);
    case "note":
      return noteEntries(target.noteId, deps);
    case "folder":
      return folderEntries(target.folderId, deps);
    case "tag":
      return tagEntries(target.tagId, deps, "agenda");
    case "note-tag":
      return tagEntries(target.tagId, deps, "notes");
    case "agenda":
      return agendaEntries(target.date, deps);
    case "notes":
      return [
        {
          id: "new-note",
          label: t.context.newNote,
          run: () => void deps.notes.createNote(),
        },
      ];
    case "editor":
      return editorEntries(deps, target.node, target.inTable);
    case "note-link":
      return [
        {
          id: "open",
          label: t.context.openLinkedNote,
          run: () => void deps.showNote(target.noteId),
        },
        removeEntry(target.node, t.context.removeLink),
      ];
    case "task-ref":
      return taskRefEntries(target.taskId, target.node, deps);
    default:
      return [];
  }
}

// ── Tasks ────────────────────────────────────────────────────────────────

function tagChoices(
  deps: ContextDeps,
  current: () => string[],
  toggle: (next: string[]) => void,
): ContextEntry[] {
  if (deps.tags.tags.length === 0) {
    return [{ id: "no-tags", label: t.context.noTags, disabled: true }];
  }
  const ids = current();
  return deps.tags.tags.map((tag) => ({
    id: `tag:${tag.id}`,
    label: tag.name,
    dot: tag.colorHex,
    checked: ids.includes(tag.id),
    keepOpen: true,
    run: () =>
      toggle(
        ids.includes(tag.id)
          ? ids.filter((id) => id !== tag.id)
          : [...ids, tag.id],
      ),
  }));
}

function taskEntries(taskId: string, deps: ContextDeps): ContextEntry[] {
  const task = deps.tasks.find(taskId);
  if (!task) return [];
  const { tasks, view } = deps;
  const today = view.today();
  const tomorrow = addDays(today, 1);

  const dates: ContextEntry[] = [
    {
      id: "today",
      label: t.context.today,
      checked: task.dueDate === today,
      run: () => void tasks.setDueDate(task.id, today),
    },
    {
      id: "tomorrow",
      label: t.context.tomorrow,
      checked: task.dueDate === tomorrow,
      run: () => void tasks.setDueDate(task.id, tomorrow),
    },
    {
      id: "pick",
      label: t.context.pickDate,
      run: () => deps.editTask(task),
    },
  ];
  if (task.dueDate) {
    dates.push({
      id: "clear-date",
      label: t.context.clearDate,
      separated: true,
      run: () => void tasks.setDueDate(task.id, null),
    });
  }

  const entries: ContextEntry[] = [
    {
      id: "toggle",
      label: task.isCompleted ? t.context.reopen : t.context.done,
      run: () => void tasks.setCompleted(task.id, !task.isCompleted),
    },
    { id: "open", label: t.context.open, run: () => deps.openTask(task.id) },
    { id: "edit", label: t.context.edit, run: () => deps.editTask(task) },
    {
      id: "date",
      label: task.dueDate
        ? `${t.context.date} · ${formatDisplayDate(task.dueDate)}`
        : t.context.date,
      separated: true,
      children: dates,
    },
  ];
  if (!task.isCompleted) {
    entries.push({
      id: "reminder",
      label: t.context.reminder,
      // The reminder panel lives under the row of the list; elsewhere the edit dialog has them.
      run: () =>
        view.mode === "list"
          ? tasks.toggleReminders(task.id)
          : deps.editTask(task),
    });
  }
  entries.push(
    {
      id: "tags",
      label: t.context.tags,
      children: tagChoices(
        deps,
        () => deps.tasks.find(task.id)?.tagIds ?? [],
        (next) => void tasks.setTags(task.id, next),
      ),
    },
    {
      id: "duplicate",
      label: t.context.duplicate,
      run: () => void duplicate(task, deps),
    },
  );
  if (task.sourceNoteId) {
    const noteId = task.sourceNoteId;
    entries.push({
      id: "source",
      label: t.context.openNote,
      separated: true,
      run: () => void deps.showNote(noteId),
    });
  } else {
    entries.push({
      id: "create-note",
      label: t.context.createNote,
      separated: true,
      run: () => void deps.noteForTask(task),
    });
  }
  entries.push(
    {
      id: "convert",
      label: t.context.convertNote,
      run: () => void deps.taskToNote(task),
    },
    {
      id: "delete",
      label: t.context.delete,
      danger: true,
      separated: true,
      run: () => void tasks.remove(task.id),
    },
  );
  return entries;
}

async function duplicate(task: Task, deps: ContextDeps): Promise<void> {
  await deps.tasks.create({
    title: task.title,
    ...(task.dueDate ? { dueDate: task.dueDate } : {}),
    ...(task.dueTime ? { dueTime: task.dueTime } : {}),
    ...(task.durationMinutes !== null
      ? { durationMinutes: task.durationMinutes }
      : {}),
    ...(task.tagIds.length > 0 ? { tagIds: task.tagIds } : {}),
    ...(task.contentJson.trim().startsWith("{")
      ? { contentJson: task.contentJson }
      : {}),
  });
}

// ── Notes, folders and tags ──────────────────────────────────────────────

function noteEntries(noteId: string, deps: ContextDeps): ContextEntry[] {
  const { notes } = deps;
  const note: NoteSummary | undefined = notes.notes.find(
    (candidate) => candidate.id === noteId,
  );
  if (!note) return [];
  if (notes.scope === "trash") {
    return [
      {
        id: "open",
        label: t.context.open,
        run: () => void notes.openNote(noteId),
      },
      {
        id: "restore",
        label: t.context.restore,
        run: () => void notes.restoreNote(noteId),
      },
    ];
  }
  const folders: ContextEntry[] = [
    {
      id: "move:inbox",
      label: t.context.inbox,
      checked: note.folderId === null,
      run: () => void notes.moveNote(noteId, null),
    },
    ...notes.tree.map((row) => ({
      id: `move:${row.folder.id}`,
      label: row.folder.name,
      depth: row.depth,
      checked: note.folderId === row.folder.id,
      run: () => void notes.moveNote(noteId, row.folder.id),
    })),
  ];
  return [
    {
      id: "open",
      label: t.context.open,
      run: () => void notes.openNote(noteId),
    },
    {
      id: "pin",
      label: note.isPinned ? t.context.unpin : t.context.pin,
      run: () => void notes.setPinned(noteId, !note.isPinned),
    },
    { id: "move", label: t.context.moveTo, separated: true, children: folders },
    {
      id: "tags",
      label: t.context.tags,
      children: tagChoices(
        deps,
        () =>
          deps.notes.notes.find((candidate) => candidate.id === noteId)
            ?.tagIds ?? [],
        (next) => void notes.setTags(noteId, next),
      ),
    },
    {
      id: "duplicate",
      label: t.context.duplicate,
      run: () => void notes.duplicateNote(noteId),
    },
    {
      id: "delete",
      label: t.context.delete,
      danger: true,
      separated: true,
      run: () => void notes.trashNote(noteId),
    },
  ];
}

function folderEntries(folderId: string, deps: ContextDeps): ContextEntry[] {
  const { notes, menu } = deps;
  if (!notes.byId.has(folderId)) return [];
  return [
    {
      id: "new-note",
      label: t.context.newNoteHere,
      run: () =>
        void notes
          .selectScope(`folder:${folderId}`)
          .then(() => notes.createNote()),
    },
    {
      id: "subfolder",
      label: t.context.newSubfolder,
      run: () => menu.request({ kind: "folder", mode: "new", id: folderId }),
    },
    {
      id: "rename",
      label: t.context.rename,
      separated: true,
      run: () => menu.request({ kind: "folder", mode: "rename", id: folderId }),
    },
    {
      id: "delete",
      label: t.context.deleteFolder,
      danger: true,
      run: () => menu.request({ kind: "folder", mode: "delete", id: folderId }),
    },
  ];
}

function mergeChoices(tagId: string, deps: ContextDeps): ContextEntry[] {
  return deps.tags.tags
    .filter((tag) => tag.id !== tagId)
    .map((tag) => ({
      id: `merge:${tag.id}`,
      label: tag.name,
      dot: tag.colorHex,
      run: () =>
        void deps.tags.merge(tagId, tag.id).then(async (merged) => {
          if (!merged) return;
          await deps.tasks.load();
          await deps.notes.load();
        }),
    }));
}

function tagEntries(
  tagId: string,
  deps: ContextDeps,
  place: "agenda" | "notes",
): ContextEntry[] {
  if (!deps.tags.byId.has(tagId)) return [];
  const merge = mergeChoices(tagId, deps);
  const mergeEntry: ContextEntry | null =
    merge.length > 0
      ? { id: "merge", label: t.context.mergeInto, children: merge }
      : null;
  const entries: ContextEntry[] =
    place === "agenda"
      ? [
          {
            id: "only",
            label: t.context.showOnly,
            run: () => deps.tags.only(tagId),
          },
          {
            id: "toggle",
            label: t.context.toggleVisible,
            checked: deps.tags.isChecked(tagId),
            run: () => deps.tags.toggle(tagId),
          },
          {
            id: "edit",
            label: t.context.editTag,
            separated: true,
            run: () => deps.menu.request({ kind: "tag-edit", id: tagId }),
          },
        ]
      : [
          {
            id: "see",
            label: t.context.seeNotes,
            run: () => void deps.notes.selectScope(`tag:${tagId}`),
          },
          {
            id: "new-note",
            label: t.context.newNoteTagged,
            run: () =>
              void deps.notes
                .selectScope(`tag:${tagId}`)
                .then(() => deps.notes.createNote()),
          },
        ];
  if (mergeEntry) entries.push({ ...mergeEntry, separated: true });
  return entries;
}

// ── Section backgrounds ──────────────────────────────────────────────────

function agendaEntries(date: string | null, deps: ContextDeps): ContextEntry[] {
  const { view } = deps;
  const entries: ContextEntry[] = [
    {
      id: "new-task",
      label: date
        ? t.context.newTaskOn(formatDisplayDate(date))
        : t.context.newTask,
      run: () => deps.newTask(date),
    },
  ];
  if (date) {
    entries.push({
      id: "go-day",
      label: t.context.goToDay,
      run: () => view.showDay(date),
    });
  }
  entries.push(
    {
      id: "today",
      label: t.context.goToToday,
      run: () => view.goToToday(),
    },
    {
      id: "view",
      label: t.context.view,
      separated: true,
      children: VIEW_MODES.map(({ mode, label }) => ({
        id: `view:${mode}`,
        label,
        checked: view.mode === mode,
        run: () => view.setMode(mode),
      })),
    },
  );
  return entries;
}

// ── Editor ───────────────────────────────────────────────────────────────

function tableEntries(node: Element): ContextEntry[] {
  const actions = editorActions.get(node);
  if (!actions) return [];
  const row = (
    id: TableAction,
    label: string,
    extra: Partial<ContextEntry> = {},
  ): ContextEntry => ({
    id,
    label,
    run: () => actions.tableAction(id),
    ...extra,
  });
  return [
    row("addRowBefore", t.context.rowBefore, { separated: true }),
    row("addRowAfter", t.context.rowAfter),
    row("addColumnBefore", t.context.columnBefore),
    row("addColumnAfter", t.context.columnAfter),
    row("toggleHeaderRow", t.context.headerRow),
    row("deleteRow", t.context.deleteRow, { separated: true }),
    row("deleteColumn", t.context.deleteColumn),
    row("deleteTable", t.context.deleteTable, { danger: true }),
  ];
}

function editorEntries(
  deps: ContextDeps,
  node: Element,
  inTable: boolean,
): ContextEntry[] {
  const { clipboard } = deps;
  return [
    { id: "cut", label: t.context.cut, run: () => clipboard.cut() },
    { id: "copy", label: t.context.copy, run: () => clipboard.copy() },
    { id: "paste", label: t.context.paste, run: () => clipboard.paste() },
    {
      id: "select-all",
      label: t.context.selectAll,
      separated: true,
      run: () => clipboard.selectAll(),
    },
    ...(inTable ? tableEntries(node) : []),
  ];
}

function removeEntry(node: Element, label: string): ContextEntry {
  const remove = nodeRemovers.get(node);
  return {
    id: "remove",
    label,
    danger: true,
    separated: true,
    disabled: !remove,
    run: () => remove?.(),
  };
}

function taskRefEntries(
  taskId: string,
  node: Element,
  deps: ContextDeps,
): ContextEntry[] {
  const task = deps.tasks.find(taskId);
  const entries: ContextEntry[] = [
    {
      id: "open",
      label: t.context.openTask,
      run: () => deps.openTask(taskId),
    },
  ];
  if (task) {
    entries.push({
      id: "toggle",
      label: task.isCompleted ? t.context.reopen : t.context.done,
      run: () => void deps.tasks.setCompleted(taskId, !task.isCompleted),
    });
  }
  entries.push(removeEntry(node, t.context.removeReference));
  return entries;
}
