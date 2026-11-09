import { describe, expect, it } from "vitest";

import { makeTask } from "../testing";
import type { Tag, TagUpsertInput, TagWithCount } from "../types";
import { NO_TAG, TagsStore, type TagsApi } from "./tags.svelte";
import { TasksStore } from "./tasks.svelte";
import { FakeApi } from "../testing";

function makeTag(id: string, overrides: Partial<Tag> = {}): TagWithCount {
  return {
    id,
    name: id,
    colorHex: "#4C8DFF",
    parentId: null,
    position: 0,
    pendingCount: 0,
    ...overrides,
  };
}

class FakeTagsApi implements TagsApi {
  tags: TagWithCount[];
  failing = false;
  duplicate = false;

  constructor(tags: TagWithCount[] = []) {
    this.tags = tags;
  }

  async tagList() {
    return this.tags;
  }

  async tagUpsert(input: TagUpsertInput): Promise<Tag> {
    if (this.failing) throw "DB: fallo simulado";
    if (this.duplicate) throw "CONFLICT: ya existe";
    const saved = makeTag(input.id ?? `t${this.tags.length + 1}`, {
      name: input.name,
      colorHex: input.colorHex,
      position: this.tags.length,
    });
    this.tags = [...this.tags.filter((tag) => tag.id !== saved.id), saved];
    return saved;
  }

  async tagDelete(id: string) {
    if (this.failing) throw "DB: fallo simulado";
    this.tags = this.tags.filter((tag) => tag.id !== id);
  }
}

async function setup(tags: TagWithCount[] = [makeTag("mat"), makeTag("fis")]) {
  const api = new FakeTagsApi(tags);
  const store = new TagsStore(api);
  await store.load();
  return { api, store };
}

describe("TagsStore", () => {
  it("loads the tags in the backend's order", async () => {
    const { store } = await setup([makeTag("b"), makeTag("a")]);
    expect(store.tags.map((tag) => tag.id)).toEqual(["b", "a"]);
    expect(store.status).toBe("ready");
  });

  it("shows everything until a group is unchecked", async () => {
    const { store } = await setup();
    const tagged = makeTask({ id: "1", tagIds: ["mat"] });
    const bare = makeTask({ id: "2", tagIds: [] });
    expect(store.filtered).toBe(false);
    expect(store.isVisible(tagged)).toBe(true);
    expect(store.isVisible(bare)).toBe(true);

    store.toggle("mat");
    expect(store.filtered).toBe(true);
    expect(store.isVisible(tagged)).toBe(false);
    expect(store.isVisible(bare)).toBe(true);

    store.toggle(NO_TAG);
    expect(store.isVisible(bare)).toBe(false);

    store.toggle("mat");
    store.toggle(NO_TAG);
    expect(store.filtered).toBe(false);
  });

  it("treats a task whose tag is unknown as untagged", async () => {
    const { store } = await setup();
    const orphan = makeTask({ id: "1", tagIds: ["gone"] });
    expect(store.tagOf(orphan)).toBeUndefined();
    expect(store.isVisible(orphan)).toBe(true);
    store.toggle(NO_TAG);
    expect(store.isVisible(orphan)).toBe(false);
  });

  it("selects all, none, or only one group", async () => {
    const { store } = await setup();
    store.hideAll();
    expect(["mat", "fis", NO_TAG].every((key) => !store.isChecked(key))).toBe(
      true,
    );
    store.only("fis");
    expect(store.isChecked("fis")).toBe(true);
    expect(store.isChecked("mat")).toBe(false);
    expect(store.isChecked(NO_TAG)).toBe(false);
    store.showAll();
    expect(store.filtered).toBe(false);
  });

  it("suggests the first preset colour no tag uses", async () => {
    const { store } = await setup([makeTag("a", { colorHex: "#4c8dff" })]);
    expect(store.nextColor()).not.toBe("#4C8DFF");
  });

  it("creates a tag at the end and edits one in place", async () => {
    const { store } = await setup();
    expect(
      await store.save({ name: "Química", colorHex: "#34B36B" }),
    ).toBeNull();
    expect(store.tags.map((tag) => tag.name)).toEqual([
      "mat",
      "fis",
      "Química",
    ]);
    expect(
      await store.save({ id: "mat", name: "Mates", colorHex: "#E5588A" }),
    ).toBeNull();
    expect(store.tags.map((tag) => tag.name)).toEqual([
      "Mates",
      "fis",
      "Química",
    ]);
  });

  it("explains why a tag was refused", async () => {
    const { api, store } = await setup();
    api.duplicate = true;
    expect(await store.save({ name: "mat", colorHex: "#4C8DFF" })).toBe(
      "duplicate",
    );
    api.duplicate = false;
    api.failing = true;
    expect(await store.save({ name: "x", colorHex: "#4C8DFF" })).toBe("failed");
    expect(store.tags).toHaveLength(2);
  });

  it("forgets a deleted tag and its filter entry", async () => {
    const { store } = await setup();
    store.toggle("mat");
    expect(await store.remove("mat")).toBe(true);
    expect(store.tags.map((tag) => tag.id)).toEqual(["fis"]);
    expect(store.filtered).toBe(false);
  });

  it("keeps the tag when the backend refuses to delete it", async () => {
    const { api, store } = await setup();
    api.failing = true;
    expect(await store.remove("mat")).toBe(false);
    expect(store.tags).toHaveLength(2);
  });
});

describe("TasksStore with a tag filter", () => {
  async function tasksSetup() {
    const api = new FakeApi([
      makeTask({ id: "a", title: "A", tagIds: ["mat"] }),
      makeTask({ id: "b", title: "B", tagIds: [] }),
      makeTask({ id: "c", title: "C", dueDate: "2026-03-10", tagIds: ["fis"] }),
    ]);
    const store = new TasksStore({ api, settleMs: () => 0 });
    const tags = new TagsStore(
      new FakeTagsApi([makeTag("mat"), makeTag("fis")]),
    );
    await tags.load();
    store.filter = (task) => tags.isVisible(task);
    await store.load();
    return { api, store, tags };
  }

  it("filters both the list and the agenda", async () => {
    const { store, tags } = await tasksSetup();
    expect(store.visibleTasks).toHaveLength(3);

    tags.toggle("mat");
    expect(store.todoBlock.open.map((task) => task.id)).toEqual(["b"]);
    expect(store.agendaBlocks.map((block) => block.date)).toEqual([
      "2026-03-10",
    ]);

    tags.toggle("fis");
    tags.toggle(NO_TAG);
    expect(store.visibleTasks).toHaveLength(0);
    expect(store.allFiltered).toBe(true);
    expect(store.isEmpty).toBe(false);
  });

  it("gives a task its tag, or takes it away", async () => {
    const { api, store } = await tasksSetup();
    await store.setTag("b", "mat");
    expect(store.find("b")?.tagIds).toEqual(["mat"]);
    expect(api.calls).toContain("taskTagsSet:b=mat");
    await store.setTag("b", null);
    expect(store.find("b")?.tagIds).toEqual([]);
  });

  it("puts the tag back when the backend refuses it", async () => {
    const { api, store } = await tasksSetup();
    api.failing.add("taskTagsSet");
    await store.setTag("b", "mat");
    expect(store.find("b")?.tagIds).toEqual([]);
    expect(store.notice?.kind).toBe("error");
  });

  it("creates a tagged task and warns when the filter hides it", async () => {
    const { store, tags } = await tasksSetup();
    tags.toggle("fis");
    expect(await store.createDetailed({ title: "Nueva", tagId: "fis" })).toBe(
      true,
    );
    const created = store.tasks.find((task) => task.title === "Nueva");
    expect(created?.tagIds).toEqual(["fis"]);
    expect(store.notice?.message).toContain("filtro");
  });

  it("saves a changed tag with the other details", async () => {
    const { api, store } = await tasksSetup();
    const ok = await store.saveDetails("b", {
      title: "B",
      dueDate: null,
      dueTime: null,
      durationMinutes: null,
      tagId: "fis",
    });
    expect(ok).toBe(true);
    expect(store.find("b")?.tagIds).toEqual(["fis"]);
    expect(api.calls.filter((call) => call.startsWith("taskTagsSet"))).toEqual([
      "taskTagsSet:b=fis",
    ]);
  });
});
