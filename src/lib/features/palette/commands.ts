// The commands of the palette (F11-04). Each one drives the same stores the rest of the app
// uses, so a command never does anything a button could not.

import { toLocalIsoDate } from "../../dates";
import { t } from "../../i18n";
import type { NotesStore, ScopeKey } from "../../stores/notes.svelte";
import type { TagsStore } from "../../stores/tags.svelte";
import type { TasksStore } from "../../stores/tasks.svelte";
import type { ViewStore } from "../../stores/view.svelte";
import type { ViewMode } from "../../types";
import type { PaletteCommand } from "./palette";

export interface CommandDeps {
  view: ViewStore;
  notes: NotesStore;
  tasks: TasksStore;
  tags: TagsStore;
  /** Opens the full creation dialog (the one Ctrl+N opens). */
  newTask?: () => void;
  /** Opens the event dialog. */
  newEvent?: () => void;
  /** Opens the habit dialog. */
  newHabit?: () => void;
  /** Opens the «Planificar mi día» assistant. */
  planDay?: () => void;
}

export function buildCommands(deps: CommandDeps): PaletteCommand[] {
  const { view, notes, tasks, tags, newTask, newEvent, newHabit, planDay } =
    deps;
  const l = t.palette.commands;

  const inAgenda = (action: () => void) => () => {
    view.showAgenda();
    action();
  };
  const showView = (mode: ViewMode) => inAgenda(() => view.setMode(mode));
  const showNotes = (scope: ScopeKey) => () => {
    view.setSection("notes");
    void notes.ensureLoaded().then(() => notes.selectScope(scope));
  };

  return [
    {
      id: "new-task",
      label: l.newTask,
      shortcut: "Ctrl+N",
      run: inAgenda(() =>
        newTask ? newTask() : tasks.startDraft(tasks.defaultTarget()),
      ),
    },
    {
      id: "new-task-today",
      label: l.newTaskToday,
      shortcut: "Ctrl+Mayús+N",
      run: inAgenda(() => tasks.startDraft(tasks.todayTarget())),
    },
    {
      id: "new-event",
      label: l.newEvent,
      run: () => newEvent?.(),
    },
    {
      id: "new-habit",
      label: l.newHabit,
      run: () => newHabit?.(),
    },
    {
      id: "plan-day",
      label: l.planDay,
      run: () => {
        view.setSection("today");
        planDay?.();
      },
    },
    {
      id: "go-habits",
      label: l.goHabits,
      run: () => view.setSection("habits"),
    },
    {
      id: "go-summary",
      label: l.goSummary,
      run: () => view.setSection("summary"),
    },
    {
      id: "go-events",
      label: l.goEvents,
      run: () => view.setSection("events"),
    },
    {
      id: "new-note",
      label: l.newNote,
      run: () => {
        view.setSection("notes");
        void notes.ensureLoaded().then(() => notes.createNote());
      },
    },
    {
      id: "daily-note",
      label: l.dailyNote,
      run: () => {
        view.setSection("notes");
        void notes
          .ensureLoaded()
          .then(() => notes.openDaily(toLocalIsoDate(new Date())));
      },
    },
    {
      id: "templates",
      label: l.templates,
      run: () => {
        view.setSection("notes");
        void notes.ensureLoaded().then(() => notes.openTemplates());
      },
    },
    {
      id: "go-agenda",
      label: l.goAgenda,
      shortcut: "Ctrl+Mayús+1",
      run: () => view.setSection("tasks"),
    },
    {
      id: "go-today",
      label: l.goToday,
      shortcut: "Ctrl+Mayús+3",
      run: () => view.setSection("today"),
    },
    {
      id: "go-notes",
      label: l.goNotes,
      shortcut: "Ctrl+Mayús+2",
      run: () => {
        view.setSection("notes");
        void notes.ensureLoaded();
      },
    },
    {
      id: "view-list",
      label: l.viewList,
      shortcut: "Ctrl+1",
      run: showView("list"),
    },
    {
      id: "view-day",
      label: l.viewDay,
      shortcut: "Ctrl+2",
      run: showView("day"),
    },
    {
      id: "view-week",
      label: l.viewWeek,
      shortcut: "Ctrl+3",
      run: showView("week"),
    },
    {
      id: "view-month",
      label: l.viewMonth,
      shortcut: "Ctrl+4",
      run: showView("month"),
    },
    { id: "notes-all", label: l.notesAll, run: showNotes("all") },
    { id: "notes-inbox", label: l.notesInbox, run: showNotes("inbox") },
    { id: "notes-pinned", label: l.notesPinned, run: showNotes("pinned") },
    { id: "notes-orphans", label: l.notesOrphans, run: showNotes("orphans") },
    { id: "notes-trash", label: l.notesTrash, run: showNotes("trash") },
    {
      id: "show-all-tags",
      label: l.showAllTags,
      run: () => tags.showAll(),
    },
  ];
}
