import type { Task, TasksSections } from "../types";

function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  return (
    typeof a === "object" &&
    a !== null &&
    typeof b === "object" &&
    b !== null &&
    JSON.stringify(a) === JSON.stringify(b)
  );
}

function sameTask(a: Task, b: Task): boolean {
  return (Object.keys(a) as (keyof Task)[]).every((key) =>
    sameValue(a[key], b[key]),
  );
}

/** The tasks of both sections as one list: POR HACER first, then each day. */
export function flattenSections(sections: TasksSections): Task[] {
  return [...sections.todo, ...sections.agenda.flatMap((day) => day.tasks)];
}

/**
 * The fresh list reusing the object of every task that did not change, so the rows (and what
 * they derive) that are intact are not invalidated. Returns `current` itself when nothing changed.
 */
export function mergeTasks(current: readonly Task[], fresh: Task[]): Task[] {
  const known = new Map(current.map((task) => [task.id, task]));
  const merged = fresh.map((task) => {
    const old = known.get(task.id);
    return old && sameTask(old, task) ? old : task;
  });
  const unchanged =
    merged.length === current.length &&
    merged.every((task, at) => task === current[at]);
  return unchanged ? (current as Task[]) : merged;
}
