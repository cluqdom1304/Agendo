// @vitest-environment jsdom
import { render } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import CalendarView from "../features/calendar/CalendarView.svelte";
import TodoView from "../features/todo/TodoView.svelte";
import Spotlight from "../shell/Spotlight.svelte";
import WindowControls from "../shell/WindowControls.svelte";
import EmptyState from "../shell/EmptyState.svelte";
import { TasksStore } from "../stores/tasks.svelte";
import { ViewStore } from "../stores/view.svelte";
import { FakeApi, installLayoutStubs, makeTask } from "../testing";
import type { ViewMode } from "../types";
import { unnamedControls } from "./names";

installLayoutStubs();

async function tasksStore() {
  const api = new FakeApi([
    makeTask({ id: "a", title: "Sin fecha" }),
    makeTask({
      id: "b",
      title: "Con hora",
      dueDate: "2026-03-10",
      dueTime: "09:30",
    }),
    makeTask({
      id: "c",
      title: "Hecha",
      dueDate: "2026-03-10",
      isCompleted: true,
    }),
  ]);
  const store = new TasksStore({
    api,
    now: () => new Date("2026-03-10T10:00:00"),
  });
  await store.load();
  return store;
}

describe("accessible names", () => {
  it("the audit itself finds an unnamed button", () => {
    const host = document.createElement("div");
    host.innerHTML = '<button></button><button aria-label="Ok"></button>';
    expect(unnamedControls(host)).toHaveLength(1);
  });

  it("every control of the list view has a name", async () => {
    const { container } = render(TodoView, { store: await tasksStore() });
    expect(unnamedControls(container)).toEqual([]);
  });

  it.each<Exclude<ViewMode, "list">>(["day", "week", "month"])(
    "every control of the %s view has a name",
    async (mode) => {
      const view = new ViewStore({
        api: {
          settingsGet: async () => null,
          settingsSet: async () => undefined,
        },
        now: () => new Date("2026-03-10T10:00:00"),
        firstDay: () => 1,
      });
      view.mode = mode;
      const { container } = render(CalendarView, {
        store: await tasksStore(),
        view,
        onOpenTask: () => undefined,
      });
      expect(unnamedControls(container)).toEqual([]);
    },
  );

  it("the shell components have names", () => {
    const search = render(Spotlight, {
      search: async () => [],
      onSelect: () => undefined,
      onClose: () => undefined,
    });
    expect(unnamedControls(search.container)).toEqual([]);
    search.unmount();

    const controls = render(WindowControls);
    expect(unnamedControls(controls.container)).toEqual([]);
    controls.unmount();

    const empty = render(EmptyState, {
      title: "Todo en orden",
      description: "Sin tareas",
      actionLabel: "Nueva tarea",
      onAction: () => undefined,
    });
    expect(unnamedControls(empty.container)).toEqual([]);
  });
});
