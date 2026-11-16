// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeApi, makeTask } from "../../testing";
import { FakeNotesApi, note } from "../notes/testing";
import TodayView from "./TodayView.svelte";

const NOW = new Date("2026-03-10T10:00:00");

async function mount(
  tasks = [makeTask({ id: "a", title: "Entregar", dueDate: "2026-03-10" })],
  pinned: string[] = [],
) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
  await store.load();
  const notesApi = new FakeNotesApi();
  notesApi.notes = pinned.map((title, index) =>
    note(`p${index}`, { title, isPinned: true }),
  );
  const notes = new NotesStore(notesApi);
  const view = new ViewStore({
    api: { settingsGet: async () => null, settingsSet: async () => undefined },
    now: () => NOW,
  });
  const opened: string[] = [];
  render(TodayView, {
    store,
    notes,
    view,
    onOpenTask: (id: string) => opened.push(`task:${id}`),
    onOpenNote: (id: string) => opened.push(`note:${id}`),
    onOpenDaily: (date: string) => opened.push(`daily:${date}`),
  });
  return { api, opened };
}

describe("TodayView", () => {
  it("lists only the tasks of today, open ones first, and opens one", async () => {
    const { opened } = await mount([
      makeTask({
        id: "done",
        title: "Hecha",
        dueDate: "2026-03-10",
        isCompleted: true,
      }),
      makeTask({
        id: "late",
        title: "Tarde",
        dueDate: "2026-03-10",
        dueTime: "18:00",
      }),
      makeTask({
        id: "early",
        title: "Pronto",
        dueDate: "2026-03-10",
        dueTime: "08:00",
      }),
      makeTask({ id: "other", title: "Mañana", dueDate: "2026-03-11" }),
    ]);
    const titles = screen
      .getAllByRole("button")
      .map((button) => button.textContent ?? "");
    const shown = ["Pronto", "Tarde", "Hecha"].map((title) =>
      titles.findIndex((text) => text.includes(title)),
    );
    expect(shown.every((index) => index >= 0)).toBe(true);
    expect(shown).toEqual([...shown].sort((a, b) => a - b));
    expect(screen.queryByText("Mañana")).toBeNull();
    expect(screen.getByText("2 pendientes")).toBeTruthy();
    await fireEvent.click(screen.getByText("Pronto"));
    expect(opened).toEqual(["task:early"]);
  });

  it("completes a task from its checkbox", async () => {
    const { api } = await mount();
    await fireEvent.click(
      screen.getByRole("checkbox", { name: "Marcar «Entregar» como hecha" }),
    );
    await waitFor(() =>
      expect(
        api.calls.some((call) => call.startsWith("taskSetCompleted")),
      ).toBe(true),
    );
  });

  it("says when the day is empty and opens the daily note", async () => {
    const { opened } = await mount([]);
    expect(screen.getByText("Nada para hoy")).toBeTruthy();
    await fireEvent.click(
      screen.getByRole("button", { name: "Abrir la nota de hoy" }),
    );
    expect(opened).toEqual(["daily:2026-03-10"]);
  });

  it("lists the pinned notes, or says there are none", async () => {
    const { opened } = await mount([], ["Plan semanal"]);
    await waitFor(() => expect(screen.getByText("Plan semanal")).toBeTruthy());
    await fireEvent.click(screen.getByText("Plan semanal"));
    expect(opened).toEqual(["note:p0"]);
  });

  it("shows an empty message without pinned notes", async () => {
    await mount([], []);
    await waitFor(() =>
      expect(screen.getByText("No tienes notas fijadas.")).toBeTruthy(),
    );
  });
});
