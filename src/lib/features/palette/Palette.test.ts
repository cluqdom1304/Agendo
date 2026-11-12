// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import type { FoundItem } from "../../types";
import Palette from "./Palette.svelte";
import type { PaletteMode } from "./palette";

const note = (noteId: string, title: string, snippet = ""): FoundItem => ({
  kind: "note",
  noteId,
  title,
  snippet,
  folderId: null,
  isPinned: false,
});
const task = (taskId: string, title: string): FoundItem => ({
  kind: "task",
  taskId,
  title,
  snippet: "",
  dueDate: "2026-03-12",
  isCompleted: false,
});
const tag = (tagId: string, name: string): FoundItem => ({
  kind: "tag",
  tagId,
  name,
  colorHex: "#4C8DFF",
  noteCount: 2,
  pendingCount: 1,
});

function mount(
  search: (query: string) => Promise<FoundItem[]>,
  mode: PaletteMode = "all",
) {
  const run = vi.fn();
  const handlers = {
    onNote: vi.fn(),
    onTask: vi.fn(),
    onTag: vi.fn(),
    onClose: vi.fn(),
  };
  render(Palette, {
    mode,
    search,
    commands: [
      { id: "new", label: "Nueva tarea", shortcut: "Ctrl+N", run },
      { id: "notes", label: "Ir a las notas", run: vi.fn() },
    ],
    recent: [{ id: "r1", title: "Apuntes recientes" }],
    ...handlers,
  });
  const input = screen.getByRole("combobox");
  return { input, run, ...handlers };
}

const type = (input: HTMLElement, value: string) =>
  fireEvent.input(input, { target: { value } });

describe("Palette", () => {
  it("focuses the field and, before typing, offers recent notes and commands without searching", () => {
    const search = vi.fn(async () => []);
    const { input } = mount(search);
    expect(document.activeElement).toBe(input);
    expect(screen.getByText("Apuntes recientes")).toBeTruthy();
    expect(screen.getByText("Nueva tarea")).toBeTruthy();
    expect(screen.getByText("Ctrl+N")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toContain("Escribe");
    expect(search).not.toHaveBeenCalled();
  });

  it("groups notes, tasks and tags and reports the count", async () => {
    const { input } = mount(async () => [
      tag("g", "Álgebra"),
      note("n", "Apuntes de álgebra", "tema 3"),
      task("t", "Estudiar álgebra"),
    ]);
    await type(input, "alg");
    await screen.findByText("Apuntes de álgebra");
    expect(screen.getAllByRole("option")).toHaveLength(3);
    for (const group of ["Notas", "Tareas", "Etiquetas"]) {
      expect(screen.getByText(group)).toBeTruthy();
    }
    expect(screen.getByText("tema 3")).toBeTruthy();
    expect(screen.getByText("12/03/2026")).toBeTruthy();
    expect(screen.getByText("2 notas · 1 pendiente")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("3 resultados");
  });

  it("opens each kind of result through its own handler and closes", async () => {
    const { input, onNote, onTask, onTag, onClose } = mount(async () => [
      note("n", "Nota"),
      task("t", "Tarea"),
      tag("g", "Etiqueta"),
    ]);
    await type(input, "x");
    await screen.findByText("Nota");
    await fireEvent.click(screen.getByText("Nota"));
    await fireEvent.click(screen.getByText("Tarea"));
    await fireEvent.click(screen.getByText("Etiqueta"));
    expect(onNote).toHaveBeenCalledWith("n");
    expect(onTask).toHaveBeenCalledWith("t");
    expect(onTag).toHaveBeenCalledWith("g");
    expect(onClose).toHaveBeenCalledTimes(3);
  });

  it("runs a command with Enter after closing the palette", async () => {
    const { input, run, onClose } = mount(async () => []);
    await type(input, ">nueva");
    expect(screen.getAllByRole("option")).toHaveLength(1);
    await fireEvent.keyDown(input, { key: "Enter" });
    expect(onClose).toHaveBeenCalled();
    expect(run).toHaveBeenCalledTimes(1);
  });

  it("navigates with the arrows, wrapping, and chooses the active row", async () => {
    const { input, onNote } = mount(async () => [
      note("a", "Uno"),
      note("b", "Dos"),
      note("c", "Tres"),
    ]);
    await type(input, "x");
    await screen.findAllByRole("option");
    const selected = () =>
      screen
        .getAllByRole("option")
        .findIndex((option) => option.getAttribute("aria-selected") === "true");
    expect(selected()).toBe(0);
    await fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(selected()).toBe(2);
    expect(input.getAttribute("aria-activedescendant")).toBe("row-2");
    await fireEvent.keyDown(input, { key: "ArrowDown" });
    await fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(selected()).toBe(1);
    await fireEvent.keyDown(input, { key: "Enter" });
    expect(onNote).toHaveBeenCalledWith("b");
  });

  it("closes with Escape, Ctrl+K and Ctrl+P, and keeps keys from the document", async () => {
    const { input, onClose } = mount(async () => []);
    const outer = vi.fn();
    window.addEventListener("keydown", outer);
    await fireEvent.keyDown(input, { key: "Escape" });
    await fireEvent.keyDown(input, { key: "k", ctrlKey: true });
    await fireEvent.keyDown(input, { key: "p", ctrlKey: true });
    await fireEvent.keyDown(input, { key: "n", ctrlKey: true });
    window.removeEventListener("keydown", outer);
    expect(onClose).toHaveBeenCalledTimes(3);
    expect(outer).not.toHaveBeenCalled();
  });

  it("ignores a slow answer that arrives after a newer one", async () => {
    let releaseFirst!: (items: FoundItem[]) => void;
    const search = vi.fn((query: string) =>
      query === "a"
        ? new Promise<FoundItem[]>((resolve) => (releaseFirst = resolve))
        : Promise.resolve([note("n", "Nuevo")]),
    );
    const { input } = mount(search);
    await type(input, "a");
    await type(input, "ab");
    await screen.findByText("Nuevo");
    releaseFirst([note("o", "Viejo")]);
    await Promise.resolve();
    expect(screen.queryByText("Viejo")).toBeNull();
  });

  it("says so when there is nothing, and when the search fails", async () => {
    const { input } = mount(async () => []);
    await type(input, "zzz");
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toBe("Sin resultados"),
    );
  });

  it("shows a message when the search fails, still offering the commands", async () => {
    const { input } = mount(async () => {
      throw "DB: fallo";
    });
    await type(input, "nueva");
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toBe(
        "La búsqueda ha fallado.",
      ),
    );
    expect(screen.getByText("Nueva tarea")).toBeTruthy();
  });

  it("go-to mode asks for notes at once and lists them with the recent ones first", async () => {
    const search = vi.fn(async () => [
      note("r1", "Apuntes recientes"),
      note("x", "Otra nota"),
    ]);
    mount(search, "notes");
    await screen.findByText("Otra nota");
    expect(search).toHaveBeenCalledWith("is:nota");
    expect(screen.getAllByText("Apuntes recientes")).toHaveLength(1);
    expect(screen.queryByText("Nueva tarea")).toBeNull();
  });
});
