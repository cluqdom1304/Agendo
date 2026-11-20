// Front/back parity (A9-01): the same vectors that src-tauri/src/parity_tests.rs checks against the
// Rust domain code are run here against the TypeScript re-implementations.
import { readFileSync } from "node:fs";
import process from "node:process";
import { beforeAll, describe, expect, it } from "vitest";
import { sortDay, sortTodo, groupByDay } from "./ordering";
import { buildRule, parseRule, type RepeatForm } from "./recurrence";
import { dueInstant } from "./reminders";
import { maxDuration } from "./schedule";
import { toInstant, toLocalIsoDate } from "./dates";
import type { Task } from "./types";

// The vectors are written for Europe/Madrid, which is also what the Rust test simulates.
process.env.TZ = "Europe/Madrid";

type Row = Pick<
  Task,
  "id" | "dueDate" | "dueTime" | "isCompleted" | "completedAt" | "position"
> &
  Partial<Pick<Task, "priority">>;

function load<T>(name: string): T {
  const url = new URL(
    `../../src-tauri/fixtures/parity/${name}`,
    import.meta.url,
  );
  return JSON.parse(readFileSync(url, "utf8")) as T;
}

const ids = (rows: readonly Row[]) => rows.map((row) => row.id);

beforeAll(() => {
  // Guards against a runtime that ignores the TZ change: DST vectors would then be meaningless.
  expect(new Date("2026-07-15T09:30").toISOString()).toBe(
    "2026-07-15T07:30:00.000Z",
  );
});

describe("recurrence parity", () => {
  const data = load<{
    parse: { rule: string; form: RepeatForm }[];
    invalid: string[];
    build: { form: RepeatForm; rule: string }[];
  }>("recurrence.json");

  it.each(data.parse)("parses $rule", ({ rule, form }) => {
    expect(parseRule(rule)).toEqual(form);
  });

  it.each(data.invalid)("rejects %j like the backend", (rule) => {
    expect(parseRule(rule)).toBeNull();
  });

  it.each(data.build)("builds $rule", ({ form, rule }) => {
    expect(buildRule(form)).toBe(rule);
  });
});

describe("ordering parity", () => {
  const data = load<{
    todo: { name: string; tasks: Row[]; expected: string[] }[];
    day: { name: string; tasks: Row[]; expected: string[] }[];
    groups: {
      name: string;
      tasks: Row[];
      expected: { date: string; ids: string[] }[];
    }[];
  }>("ordering.json");

  it.each(data.todo)("todo: $name", ({ tasks, expected }) => {
    expect(ids(sortTodo(tasks))).toEqual(expected);
  });

  it.each(data.day)("day: $name", ({ tasks, expected }) => {
    expect(ids(sortDay(tasks))).toEqual(expected);
  });

  it.each(data.groups)("groups: $name", ({ tasks, expected }) => {
    const actual = groupByDay(tasks).map((group) => ({
      date: group.date,
      ids: ids(group.tasks),
    }));
    expect(actual).toEqual(expected);
  });
});

describe("date and instant parity", () => {
  const data = load<{
    localToUtc: { name: string; local: string; utc: string }[];
    utcToLocalDate: { utc: string; date: string; time: string }[];
  }>("dates.json");

  it.each(data.localToUtc)("local to UTC: $name", ({ local, utc }) => {
    expect(toInstant(new Date(local))).toBe(utc);
  });

  it.each(data.utcToLocalDate)("UTC $utc to local", ({ utc, date, time }) => {
    const moment = new Date(utc);
    expect(toLocalIsoDate(moment)).toBe(date);
    const clock = `${String(moment.getHours()).padStart(2, "0")}:${String(moment.getMinutes()).padStart(2, "0")}`;
    expect(clock).toBe(time);
  });
});

describe("reminder instant parity", () => {
  const data = load<{
    offsetNotifyAt: {
      dueDate: string;
      dueTime: string;
      offset: number;
      notifyAt: string;
    }[];
  }>("reminders.json");

  it.each(data.offsetNotifyAt)(
    "$dueDate $dueTime minus $offset min",
    ({ dueDate, dueTime, offset, notifyAt }) => {
      const due = dueInstant(dueDate, dueTime);
      expect(due).not.toBeNull();
      const at = new Date((due as Date).getTime() - offset * 60_000);
      expect(toInstant(at)).toBe(notifyAt);
    },
  );
});

describe("duration parity", () => {
  const data = load<{
    cases: { start: string; duration: number; valid: boolean }[];
  }>("duration.json");

  it.each(data.cases)(
    "$start + $duration min",
    ({ start, duration, valid }) => {
      const fits = duration >= 1 && duration <= maxDuration(start);
      expect(fits).toBe(valid);
    },
  );
});
