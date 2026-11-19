// Context menu entries inside the note editor.

import { t } from "../../i18n";
import type { TableAction } from "../notes/editor";
import { editorActions } from "./editorActions";
import { nodeRemovers } from "./nodeRemovers";
import type { ContextEntry } from "./context.svelte";
import type { ContextDeps } from "./deps";

export function tableEntries(node: Element): ContextEntry[] {
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

export function editorEntries(
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

export function removeEntry(node: Element, label: string): ContextEntry {
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

export function taskRefEntries(
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
