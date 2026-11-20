// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import { TagsStore } from "../../stores/tags.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeApi, makeTask } from "../../testing";
import type { TagWithCount } from "../../types";
import { FakeNotesApi, folder, note } from "../notes/testing";
import ContextMenu from "./ContextMenu.svelte";
import { contextMenu } from "./context.svelte";
import { nodeRemovers } from "./nodeRemovers";

const tag = (id: string): TagWithCount => ({
  id,
  name: `Etiqueta ${id}`,
  colorHex: "#4C8DFF",
  parentId: null,
  position: 0,
  pendingCount: 0,
  noteCount: 0,
});

async function setup(html: string) {
  const settings = new Map<string, string>();
  const view = new ViewStore({
    api: {
      settingsGet: async (key) => settings.get(key) ?? null,
      settingsSet: async (key, value) => void settings.set(key, value),
    },
    now: () => new Date("2026-03-10T10:00:00"),
    firstDay: () => 1,
  });
  const api = new FakeApi([
    makeTask({ id: "t1", title: "Estudiar", dueDate: "2026-03-12" }),
    makeTask({ id: "t2", title: "Hecha", isCompleted: true }),
  ]);
  const tasks = new TasksStore({ api, settleMs: () => 0 });
  await tasks.load();
  const tags = new TagsStore({
    tagList: async () => [tag("a"), tag("b")],
    tagUpsert: async () => {
      throw new Error("unused");
    },
    tagDelete: async () => undefined,
    tagMerge: async () => undefined,
  });
  await tags.load();
  const notesApi = new FakeNotesApi();
  notesApi.folders = [folder("f1")];
  notesApi.notes = [note("n1"), note("n2", { isPinned: true })];
  const notes = new NotesStore(notesApi);
  await notes.load();
  const deps = {
    view,
    tasks,
    tags,
    notes,
    newTask: vi.fn(),
    editTask: vi.fn(),
    openTask: vi.fn(),
    showNote: vi.fn(async () => undefined),
    noteForTask: vi.fn(async () => null),
    taskToNote: vi.fn(async () => null),
    newEvent: vi.fn(),
    openEvent: vi.fn(),
    taskToEvent: vi.fn(async () => null),
    eventToTask: vi.fn(async () => null),
    deleteEvent: vi.fn(async () => null),
  };
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  render(ContextMenu, { deps });
  return { ...deps, api, notesApi, host };
}

const rightClick = (selector: string) => {
  const el = document.querySelector(selector);
  if (!el) throw new Error(`no ${selector}`);
  return fireEvent.contextMenu(el, { clientX: 40, clientY: 50 });
};

const items = () =>
  Array.from(
    document.querySelectorAll<HTMLElement>(
      "[data-context-menu] [role^='menuitem']",
    ),
  );

const names = () => items().map((item) => item.textContent?.trim());

const press = (key: string, init: KeyboardEventInit = {}) =>
  fireEvent.keyDown(document.activeElement as Element, { key, ...init });

beforeEach(() => {
  document.body.replaceChildren();
});
afterEach(() => {
  contextMenu.close();
  contextMenu.intent = null;
});

describe("ContextMenu", () => {
  it("opens the task actions on a right click and runs one", async () => {
    const { api } = await setup(
      '<div data-menu-area="agenda"><div data-task-id="t1" tabindex="0" id="row">Estudiar</div></div>',
    );
    await rightClick("#row");
    const menu = await screen.findByRole("menu");
    expect(menu.getAttribute("aria-label")).toBe("Menú contextual");
    expect(names()).toContain("Marcar como hecha");
    expect(names()).toContain("Duplicar");
    expect(names().at(-1)).toBe("Borrar");
    await fireEvent.click(screen.getByText("Marcar como hecha"));
    expect(screen.queryByRole("menu")).toBeNull();
    await waitFor(() => expect(api.tasks[0]?.isCompleted).toBe(true));
  });

  it("offers to reopen a done task, and prevents the browser menu", async () => {
    await setup('<div data-task-id="t2" id="row">Hecha</div>');
    const event = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
    });
    document.querySelector("#row")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    await screen.findByRole("menu");
    expect(names()).toContain("Reabrir");
    expect(names()).not.toContain("Añadir aviso…");
  });

  it("keeps the browser menu in text fields", async () => {
    await setup('<input id="field" type="text">');
    const event = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
    });
    document.querySelector("#field")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("opens with the Menu key and Shift+F10, moves with the arrows and closes with Escape", async () => {
    await setup('<div data-task-id="t1" tabindex="0" id="row">Estudiar</div>');
    const row = document.querySelector<HTMLElement>("#row");
    row?.focus();
    await fireEvent.keyDown(row as HTMLElement, { key: "ContextMenu" });
    await screen.findByRole("menu");
    const all = items();
    expect(document.activeElement).toBe(all[0]);
    await press("ArrowDown");
    expect(document.activeElement).toBe(all[1]);
    await press("ArrowUp");
    await press("ArrowUp");
    expect(document.activeElement).toBe(all.at(-1));
    await press("Escape");
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(row);

    await fireEvent.keyDown(row as HTMLElement, { key: "F10", shiftKey: true });
    await screen.findByRole("menu");
  });

  it("opens a submenu with the right arrow and ticks tags without closing", async () => {
    const { api } = await setup(
      '<div data-task-id="t1" tabindex="0" id="row">Estudiar</div>',
    );
    await rightClick("#row");
    await screen.findByRole("menu");
    const tags = screen.getByRole("menuitem", { name: /Etiquetas/ });
    tags.focus();
    expect(tags.getAttribute("aria-expanded")).toBe("false");
    await fireEvent.keyDown(tags, { key: "ArrowRight" });
    const first = await screen.findByRole("menuitemcheckbox", {
      name: "Etiqueta a",
    });
    await waitFor(() => expect(document.activeElement).toBe(first));
    await fireEvent.click(first);
    await waitFor(() => expect(api.tasks[0]?.tagIds).toEqual(["a"]));
    // Still open, and the tick follows the task.
    await waitFor(() =>
      expect(
        screen
          .getByRole("menuitemcheckbox", { name: "Etiqueta a" })
          .getAttribute("aria-checked"),
      ).toBe("true"),
    );
    await press("ArrowLeft");
    expect(screen.queryAllByRole("menuitemcheckbox")).toHaveLength(0);
    expect(document.activeElement).toBe(
      screen.getByRole("menuitem", { name: /Etiquetas/ }),
    );
  });

  it("sets the date from the submenu and asks the page for the dialogs", async () => {
    const { api, editTask, tasks } = await setup(
      '<div data-task-id="t1" tabindex="0" id="row">Estudiar</div>',
    );
    await rightClick("#row");
    await fireEvent.click(
      await screen.findByRole("menuitem", { name: /Fecha/ }),
    );
    await fireEvent.click(screen.getByText("Mañana"));
    await waitFor(() => expect(api.tasks[0]?.dueDate).toBe("2026-03-11"));

    await rightClick("#row");
    await fireEvent.click(await screen.findByText("Editar…"));
    expect(editTask).toHaveBeenCalledWith(tasks.find("t1"));
  });

  it("converts a task into an event from its menu (F21-04)", async () => {
    const { taskToEvent, tasks } = await setup(
      '<div data-task-id="t1" tabindex="0" id="row">Estudiar</div>',
    );
    await rightClick("#row");
    await fireEvent.click(await screen.findByText("Convertir en evento"));
    expect(taskToEvent).toHaveBeenCalledWith(tasks.find("t1"));
  });

  it("offers an event its own actions (F21-04)", async () => {
    const { openEvent, eventToTask, deleteEvent } = await setup(
      '<div data-menu-area="events"><div data-event-id="e1" tabindex="0" id="row">Cita</div></div>',
    );
    await rightClick("#row");
    expect(await screen.findByRole("menu")).toBeTruthy();
    expect(names()).toEqual(["Editar evento…", "Convertir en tarea", "Borrar"]);
    await fireEvent.click(screen.getByText("Editar evento…"));
    expect(openEvent).toHaveBeenCalledWith("e1");

    await rightClick("#row");
    await fireEvent.click(await screen.findByText("Convertir en tarea"));
    expect(eventToTask).toHaveBeenCalledWith("e1");

    await rightClick("#row");
    await fireEvent.click(await screen.findByText("Borrar"));
    expect(deleteEvent).toHaveBeenCalledWith("e1");
  });

  it("offers a new event on the background of the calendar and of the events", async () => {
    const { newEvent } = await setup(
      '<div data-menu-area="agenda"><div data-date="2026-03-12" id="day">12</div></div><div data-menu-area="events" id="list">Eventos</div>',
    );
    await rightClick("#day");
    await fireEvent.click(
      await screen.findByText("Nuevo evento el 12/03/2026"),
    );
    expect(newEvent).toHaveBeenCalledWith("2026-03-12");

    await rightClick("#list");
    await fireEvent.click(await screen.findByText("Nuevo evento"));
    expect(newEvent).toHaveBeenLastCalledWith(null);
  });

  it("sets the priority from its submenu (F20-02)", async () => {
    const { api } = await setup(
      '<div data-task-id="t1" tabindex="0" id="row">Estudiar</div>',
    );
    await rightClick("#row");
    await fireEvent.click(
      await screen.findByRole("menuitem", { name: /Prioridad/ }),
    );
    expect(
      screen
        .getByRole("menuitemradio", { name: "Sin prioridad" })
        .getAttribute("aria-checked"),
    ).toBe("true");
    await fireEvent.click(
      screen.getByRole("menuitemradio", { name: "Urgente" }),
    );
    await waitFor(() => expect(api.tasks[0]?.priority).toBe(3));
  });

  it("duplicates and deletes a task", async () => {
    const { api, tasks } = await setup(
      '<div data-task-id="t1" tabindex="0" id="row">Estudiar</div>',
    );
    await rightClick("#row");
    await fireEvent.click(await screen.findByText("Duplicar"));
    await waitFor(() => expect(api.tasks).toHaveLength(3));
    expect(api.tasks.at(-1)).toMatchObject({
      title: "Estudiar",
      dueDate: "2026-03-12",
    });
    await rightClick("#row");
    await fireEvent.click(await screen.findByText("Borrar"));
    // The row leaves the list at once; the backend deletes when the undo window ends.
    await waitFor(() => expect(tasks.find("t1")).toBeUndefined());
  });

  it("closes when it is clicked outside", async () => {
    await setup('<div data-task-id="t1" id="row">x</div><p id="out">y</p>');
    await rightClick("#row");
    await screen.findByRole("menu");
    await fireEvent.pointerDown(document.querySelector("#out") as Element);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("note actions: pin, unpin label and move to a folder", async () => {
    const { notesApi } = await setup(
      '<div data-note-id="n1" id="card">n1</div><div data-note-id="n2" id="pinned">n2</div>',
    );
    await rightClick("#card");
    await screen.findByRole("menu");
    expect(names()).toContain("Fijar");
    await fireEvent.click(screen.getByText("Fijar"));
    await waitFor(() =>
      expect(notesApi.notes.find((n) => n.id === "n1")?.isPinned).toBe(true),
    );

    await rightClick("#pinned");
    await screen.findByRole("menu");
    expect(names()).toContain("Quitar de fijadas");

    await rightClick("#card");
    await fireEvent.click(
      await screen.findByRole("menuitem", { name: /Mover a/ }),
    );
    await fireEvent.click(screen.getByRole("menuitemradio", { name: "f1" }));
    await waitFor(() =>
      expect(notesApi.notes.find((n) => n.id === "n1")?.folderId).toBe("f1"),
    );
  });

  it("folder actions ask the sidebar for its fields", async () => {
    await setup('<div data-folder-id="f1" id="folder">f1</div>');
    await rightClick("#folder");
    await fireEvent.click(await screen.findByText("Renombrar…"));
    expect(contextMenu.intent).toEqual({
      kind: "folder",
      mode: "rename",
      id: "f1",
    });
    await rightClick("#folder");
    await fireEvent.click(await screen.findByText("Eliminar carpeta…"));
    expect(contextMenu.intent).toEqual({
      kind: "folder",
      mode: "delete",
      id: "f1",
    });
  });

  it("tag actions: show only this one, edit, and the notes version", async () => {
    const { tags } = await setup(
      '<div data-tag-id="a" id="tag">a</div><div data-note-tag-id="b" id="ntag">b</div>',
    );
    await rightClick("#tag");
    await fireEvent.click(await screen.findByText("Mostrar solo esta"));
    expect(tags.isChecked("a")).toBe(true);
    expect(tags.isChecked("b")).toBe(false);
    await rightClick("#tag");
    await fireEvent.click(await screen.findByText("Editar etiqueta…"));
    expect(contextMenu.intent).toEqual({ kind: "tag-edit", id: "a" });

    await rightClick("#ntag");
    await screen.findByRole("menu");
    expect(names()).toEqual([
      "Ver sus notas",
      "Nueva nota con esta etiqueta",
      "Fusionar con",
    ]);
  });

  it("the background of the agenda creates a task on the day under the pointer", async () => {
    const { newTask, view } = await setup(
      '<div data-menu-area="agenda"><div data-date="2026-03-12" id="day">12</div><p id="blank">.</p></div>',
    );
    await rightClick("#day");
    await fireEvent.click(await screen.findByText("Nueva tarea el 12/03/2026"));
    expect(newTask).toHaveBeenCalledWith("2026-03-12");

    await rightClick("#blank");
    await fireEvent.click(await screen.findByText("Nueva tarea"));
    expect(newTask).toHaveBeenLastCalledWith(null);

    await rightClick("#blank");
    await fireEvent.click(
      await screen.findByRole("menuitem", { name: /Vista/ }),
    );
    await fireEvent.click(
      screen.getByRole("menuitemradio", { name: "Semana" }),
    );
    expect(view.mode).toBe("week");
  });

  it("the background of the notes creates a note", async () => {
    const { notesApi } = await setup(
      '<div data-menu-area="notes"><p id="blank">.</p></div>',
    );
    await rightClick("#blank");
    await fireEvent.click(await screen.findByText("Nueva nota"));
    await waitFor(() => expect(notesApi.notes).toHaveLength(3));
  });

  it("links and references in the editor open the target and remove the node", async () => {
    const { showNote, openTask } = await setup(
      `<div class="tiptap">
        <span data-note-link data-note-id="n1" id="link">n1</span>
        <div data-task-ref data-task-id="t1" id="ref">t1</div>
      </div>`,
    );
    const removed = vi.fn();
    nodeRemovers.set(document.querySelector("#link") as Element, removed);
    await rightClick("#link");
    await screen.findByRole("menu");
    expect(names()).toEqual(["Abrir la nota enlazada", "Quitar el enlace"]);
    await fireEvent.click(screen.getByText("Quitar el enlace"));
    expect(removed).toHaveBeenCalledTimes(1);

    await rightClick("#link");
    await fireEvent.click(await screen.findByText("Abrir la nota enlazada"));
    expect(showNote).toHaveBeenCalledWith("n1");

    await rightClick("#ref");
    await fireEvent.click(await screen.findByText("Abrir la tarea"));
    expect(openTask).toHaveBeenCalledWith("t1");
  });

  it("the editor offers the clipboard", async () => {
    await setup('<div class="tiptap" id="ed">texto</div>');
    await rightClick("#ed");
    await screen.findByRole("menu");
    expect(names()).toEqual(["Cortar", "Copiar", "Pegar", "Seleccionar todo"]);
  });

  it("opens nothing where there is nothing to do, and still hides the browser menu", async () => {
    await setup('<div role="dialog"><p id="x">x</p></div>');
    const event = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
    });
    document.querySelector("#x")?.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    expect(screen.queryByRole("menu")).toBeNull();
  });
});
