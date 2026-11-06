// Single table of keyboard shortcuts (SPEC §9). Components ask `findShortcut`; no key is
// hard-coded anywhere else.

export type ShortcutId =
  | "newTask"
  | "newTaskToday"
  | "confirmTitle"
  | "cancelTitle"
  | "toggleComplete"
  | "deleteTask"
  | "undo"
  | "selectPrevious"
  | "selectNext";

/** Where the keyboard focus is: a title input, the list itself (body or a row), or elsewhere. */
export type FocusContext = "title" | "list" | "other";

interface Binding {
  id: ShortcutId;
  /** `KeyboardEvent.key`, compared case-insensitively. */
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  /** `anywhere` also fires while typing; the others only in their focus context. */
  scope: FocusContext | "anywhere";
}

export const BINDINGS: readonly Binding[] = [
  { id: "newTask", key: "n", ctrl: true, scope: "anywhere" },
  { id: "newTaskToday", key: "n", ctrl: true, shift: true, scope: "anywhere" },
  { id: "confirmTitle", key: "Enter", scope: "title" },
  { id: "cancelTitle", key: "Escape", scope: "title" },
  { id: "toggleComplete", key: " ", scope: "list" },
  { id: "deleteTask", key: "Delete", scope: "list" },
  { id: "undo", key: "z", ctrl: true, scope: "list" },
  { id: "selectPrevious", key: "ArrowUp", scope: "list" },
  { id: "selectNext", key: "ArrowDown", scope: "list" },
];

export const ROW_ATTRIBUTE = "data-task-row";
export const TITLE_ATTRIBUTE = "data-title-input";

export function focusContextOf(target: EventTarget | null): FocusContext {
  if (!(target instanceof Element)) return "list";
  if (target.hasAttribute(TITLE_ATTRIBUTE)) return "title";
  if (target.hasAttribute(ROW_ATTRIBUTE)) return "list";
  if (target === document.body || target === document.documentElement) {
    return "list";
  }
  return "other";
}

export function findShortcut(
  event: KeyboardEvent,
  context: FocusContext,
): ShortcutId | null {
  const ctrl = event.ctrlKey || event.metaKey;
  const binding = BINDINGS.find(
    (candidate) =>
      candidate.key.toLowerCase() === event.key.toLowerCase() &&
      Boolean(candidate.ctrl) === ctrl &&
      Boolean(candidate.shift) === event.shiftKey &&
      !event.altKey &&
      (candidate.scope === "anywhere" || candidate.scope === context),
  );
  return binding?.id ?? null;
}
