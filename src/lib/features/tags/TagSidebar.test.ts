// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { TagsStore, type TagsApi } from "../../stores/tags.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, makeTask } from "../../testing";
import type { Tag, TagUpsertInput } from "../../types";
import TagPicker from "./TagPicker.svelte";
import TagSidebar from "./TagSidebar.svelte";

function tag(id: string, name: string, position: number): Tag {
  return { id, name, colorHex: "#4C8DFF", parentId: null, position };
}

class FakeTagsApi implements TagsApi {
  tags = [tag("mat", "Mates", 0), tag("fis", "Física", 1)];
  async tagList() {
    return this.tags.map((item) => ({ ...item, pendingCount: 0 }));
  }
  async tagUpsert(input: TagUpsertInput) {
    const saved = tag(input.id ?? "new", input.name, this.tags.length);
    this.tags = [...this.tags.filter((item) => item.id !== saved.id), saved];
    return { ...saved, colorHex: input.colorHex };
  }
  async tagDelete(id: string) {
    this.tags = this.tags.filter((item) => item.id !== id);
  }
}

async function mount(open = true) {
  const tags = new TagsStore(new FakeTagsApi());
  await tags.load();
  const tasks = new TasksStore({
    api: new FakeApi([
      makeTask({ id: "1", tagIds: ["mat"] }),
      makeTask({ id: "2", tagIds: ["mat"] }),
      makeTask({ id: "3", tagIds: ["mat"], isCompleted: true }),
      makeTask({ id: "4", tagIds: [] }),
    ]),
    settleMs: () => 0,
  });
  tasks.filter = (task) => tags.isVisible(task);
  await tasks.load();
  render(TagSidebar, { tags, tasks, open });
  return { tags, tasks };
}

const box = (name: string) =>
  screen.getByRole("checkbox", { name: `Mostrar tareas de ${name}` });

describe("TagSidebar", () => {
  it("lists every tag in order, then the untagged group, with pending counts", async () => {
    await mount();
    const names = screen
      .getAllByRole("checkbox")
      .map((input) => input.getAttribute("aria-label"));
    expect(names).toEqual([
      "Mostrar tareas de Mates",
      "Mostrar tareas de Física",
      "Mostrar tareas de Sin etiqueta",
    ]);
    const counts = Array.from(document.querySelectorAll(".count")).map(
      (node) => node.textContent,
    );
    // The completed task does not count; a tag without tasks shows zero.
    expect(counts).toEqual(["2", "0", "1"]);
  });

  it("starts with everything checked and filters as boxes are unchecked", async () => {
    const { tags, tasks } = await mount();
    expect((box("Mates") as HTMLInputElement).checked).toBe(true);
    await fireEvent.click(box("Mates"));
    expect((box("Mates") as HTMLInputElement).checked).toBe(false);
    expect(tags.isChecked("mat")).toBe(false);
    expect(tasks.visibleTasks.map((task) => task.id)).toEqual(["4"]);
  });

  it("selects all, none or a single group", async () => {
    const { tags } = await mount();
    await fireEvent.click(screen.getByRole("button", { name: "Ninguna" }));
    expect(
      screen
        .getAllByRole("checkbox")
        .every((input) => !(input as HTMLInputElement).checked),
    ).toBe(true);
    await fireEvent.click(
      screen.getByRole("button", { name: "Mostrar solo Física" }),
    );
    expect(tags.isChecked("fis")).toBe(true);
    expect(tags.isChecked("mat")).toBe(false);
    await fireEvent.click(screen.getByRole("button", { name: "Todas" }));
    expect(tags.filtered).toBe(false);
  });

  it("creates a tag from a name and a typed colour", async () => {
    const { tags } = await mount();
    await fireEvent.click(
      screen.getByRole("button", { name: "Nueva etiqueta" }),
    );
    await fireEvent.input(screen.getByLabelText("Nombre"), {
      target: { value: "Química" },
    });
    await fireEvent.input(screen.getByLabelText("Código de color"), {
      target: { value: "#00ff80" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() =>
      expect(tags.tags.map((item) => item.name)).toContain("Química"),
    );
    expect(tags.tags.find((item) => item.name === "Química")?.colorHex).toBe(
      "#00FF80",
    );
  });

  it("will not save an invalid colour", async () => {
    await mount();
    await fireEvent.click(
      screen.getByRole("button", { name: "Nueva etiqueta" }),
    );
    await fireEvent.input(screen.getByLabelText("Nombre"), {
      target: { value: "X" },
    });
    await fireEvent.input(screen.getByLabelText("Código de color"), {
      target: { value: "#12" },
    });
    expect(screen.getByRole("alert").textContent).toContain("#RRGGBB");
    expect(
      (screen.getByRole("button", { name: "Guardar" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });

  it("deletes a tag after confirming and reloads the tasks", async () => {
    const { tags } = await mount();
    await fireEvent.click(screen.getByRole("button", { name: "Editar Mates" }));
    await fireEvent.click(screen.getByRole("button", { name: "Eliminar" }));
    expect(screen.getByText(/Las tareas se quedan sin etiqueta/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Sí, eliminar" }));
    await waitFor(() =>
      expect(tags.tags.map((item) => item.id)).toEqual(["fis"]),
    );
  });

  it("is hidden from assistive technology while folded", async () => {
    await mount(false);
    expect(document.querySelector<HTMLElement>("aside")?.inert).toBe(true);
  });
});

describe("TagPicker", () => {
  it("selects one tag at a time, or none", async () => {
    const changes: (string | null)[] = [];
    render(TagPicker, {
      tags: [tag("mat", "Mates", 0), tag("fis", "Física", 1)],
      value: "mat",
      onChange: (id: string | null) => changes.push(id),
    });
    const radios = screen.getAllByRole("radio");
    expect(radios.map((radio) => radio.getAttribute("aria-checked"))).toEqual([
      "false",
      "true",
      "false",
    ]);
    await fireEvent.click(screen.getByRole("radio", { name: "Física" }));
    await fireEvent.click(screen.getByRole("radio", { name: "Sin etiqueta" }));
    expect(changes).toEqual(["fis", null]);
  });

  it("points to the sidebar when there are no tags", () => {
    render(TagPicker, { tags: [], value: null, onChange: () => {} });
    expect(screen.getByText(/barra lateral/)).toBeTruthy();
  });
});

describe("TagMenu", () => {
  it("offers every tag and none, marks the current one and reports the choice", async () => {
    const { tagsStore } = await import("../../stores/tags.svelte");
    const TagMenu = (await import("./TagMenu.svelte")).default;
    const store = tagsStore;
    store.tags = [tag("mat", "Mates", 0), tag("fis", "Física", 1)];
    const chosen: (string | null)[] = [];
    let closed = 0;
    render(TagMenu, {
      value: "fis",
      onSelect: (id: string | null) => chosen.push(id),
      onClose: () => closed++,
    });
    const items = screen.getAllByRole("menuitemradio");
    expect(items.map((item) => item.getAttribute("aria-checked"))).toEqual([
      "false",
      "false",
      "true",
    ]);
    await fireEvent.click(screen.getByRole("menuitemradio", { name: /Mates/ }));
    expect(chosen).toEqual(["mat"]);
    expect(closed).toBe(1);
    await fireEvent.click(
      screen.getByRole("menuitemradio", { name: /Sin etiqueta/ }),
    );
    expect(chosen).toEqual(["mat", null]);
    store.tags = [];
  });
});
