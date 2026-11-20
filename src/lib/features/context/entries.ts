// What each right click offers (D-50). Pure functions of the stores: the menu calls them
// again whenever the state changes, so ticks and labels never go stale.

import { t } from "../../i18n";
import type { ContextEntry } from "./context.svelte";
import type { ContextDeps } from "./deps";
import { editorEntries, removeEntry, taskRefEntries } from "./entriesEditor";
import { folderEntries, noteEntries, tagEntries } from "./entriesNotes";
import { agendaEntries, eventEntries, taskEntries } from "./entriesTasks";
import type { MenuTarget } from "./target";

export type { ContextDeps } from "./deps";

export function buildEntries(
  target: MenuTarget,
  deps: ContextDeps,
): ContextEntry[] {
  switch (target.kind) {
    case "task":
      return taskEntries(target.taskId, deps);
    case "event":
      return eventEntries(target.eventId, deps);
    case "events":
      return [
        {
          id: "new-event",
          label: t.context.newEvent,
          run: () => deps.newEvent(null),
        },
      ];
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
        {
          id: "templates",
          label: t.context.templates,
          run: () => void deps.notes.openTemplates(),
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
