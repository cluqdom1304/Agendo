// Context menu entries for notes, folders and tags.

import { t } from "../../i18n";
import type { NotesStore } from "../../stores/notes.svelte";
import { exportSelection } from "../notes/exportNotes";
import type { ContextEntry } from "./context.svelte";
import type { ContextDeps } from "./deps";
import { tagChoices } from "./tagChoices";
import type { ExportFormat, NoteSummary } from "../../types";

export function noteEntries(noteId: string, deps: ContextDeps): ContextEntry[] {
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

export async function exportFolder(
  format: ExportFormat,
  folderId: string,
  notes: NotesStore,
): Promise<void> {
  const notice = await exportSelection(format, "folder", folderId);
  if (notice) notes.notice = notice;
}

export function folderEntries(
  folderId: string,
  deps: ContextDeps,
): ContextEntry[] {
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
      id: "export-markdown",
      label: t.context.exportFolderMarkdown,
      separated: true,
      run: () => void exportFolder("markdown", folderId, notes),
    },
    {
      id: "export-html",
      label: t.context.exportFolderHtml,
      run: () => void exportFolder("html", folderId, notes),
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

export function mergeChoices(tagId: string, deps: ContextDeps): ContextEntry[] {
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

export function tagEntries(
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
