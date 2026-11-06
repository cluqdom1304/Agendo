import { invoke } from "@tauri-apps/api/core";

import type {
  EffectApplied,
  EffectMode,
  ErrorCode,
  Section,
  Tag,
  TagUpsertInput,
  TagWithCount,
  Task,
  TaskCreateInput,
  TaskPatch,
  TasksListQuery,
  TasksSections,
} from "./types";

export type * from "./types";

const ERROR_CODES: readonly ErrorCode[] = [
  "NOT_FOUND",
  "VALIDATION",
  "CONFLICT",
  "DB",
  "IO",
  "NOTIFY",
  "INTERNAL",
];

export interface ParsedError {
  code: ErrorCode;
  message: string;
}

/** Splits the `CODE: message` string the backend rejects with (SPEC §7.1). */
export function parseError(raw: unknown): ParsedError {
  const text = typeof raw === "string" ? raw : String(raw);
  const separator = text.indexOf(": ");
  if (separator > 0) {
    const code = text.slice(0, separator);
    const known = ERROR_CODES.find((candidate) => candidate === code);
    if (known) return { code: known, message: text.slice(separator + 2) };
  }
  return { code: "INTERNAL", message: text };
}

export function startupMark(label: string): Promise<void> {
  return invoke<void>("startup_mark", { label });
}

export function windowSetEffect(mode: EffectMode): Promise<EffectApplied> {
  return invoke<EffectApplied>("window_set_effect", { mode });
}

/** Section views return both sections; the range view returns a flat list. */
export function tasksList(
  query: TasksListQuery & { view: "range" },
): Promise<Task[]>;
export function tasksList(query: TasksListQuery): Promise<TasksSections>;
export function tasksList(
  query: TasksListQuery,
): Promise<TasksSections | Task[]> {
  return invoke("tasks_list", { ...query });
}

export function taskCreate(input: TaskCreateInput = {}): Promise<Task> {
  return invoke<Task>("task_create", { ...input });
}

export function taskUpdate(id: string, patch: TaskPatch): Promise<Task> {
  return invoke<Task>("task_update", { id, patch });
}

export function taskSetCompleted(
  id: string,
  completed: boolean,
): Promise<Task> {
  return invoke<Task>("task_set_completed", { id, completed });
}

export function taskMove(
  id: string,
  toSection: Section,
  options: { date?: string; beforeId?: string } = {},
): Promise<Task> {
  return invoke<Task>("task_move", { id, toSection, ...options });
}

export function taskDelete(id: string): Promise<void> {
  return invoke<void>("task_delete", { id });
}

export function taskRestore(id: string): Promise<Task> {
  return invoke<Task>("task_restore", { id });
}

export function trashList(): Promise<Task[]> {
  return invoke<Task[]>("trash_list");
}

export function tagList(): Promise<TagWithCount[]> {
  return invoke<TagWithCount[]>("tag_list");
}

export function tagUpsert(input: TagUpsertInput): Promise<Tag> {
  return invoke<Tag>("tag_upsert", { ...input });
}

export function tagDelete(id: string): Promise<void> {
  return invoke<void>("tag_delete", { id });
}

export function taskTagsSet(taskId: string, tagIds: string[]): Promise<Task> {
  return invoke<Task>("task_tags_set", { taskId, tagIds });
}
