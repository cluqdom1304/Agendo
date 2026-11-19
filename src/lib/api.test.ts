import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { noteTitles, tasksListAll } from "./api";

beforeEach(() => invoke.mockReset());

describe("batched reads", () => {
  it("asks for every task in one call, optionally for one tag", async () => {
    invoke.mockResolvedValue({ todo: [], agenda: [] });
    await tasksListAll("mat");
    expect(invoke).toHaveBeenCalledWith("tasks_list_all", { tagId: "mat" });
  });

  it("splits more than 1000 note ids into calls of at most 1000", async () => {
    const sizes: number[] = [];
    invoke.mockImplementation(
      async (_command: string, args: { ids: string[] }) => {
        if (!args) return [];
        sizes.push(args.ids.length);
        return args.ids.map((id) => ({ id, title: id }));
      },
    );
    const ids = Array.from({ length: 2500 }, (_, at) => `n${at}`);
    const found = await noteTitles(ids);
    expect(sizes).toEqual([1000, 1000, 500]);
    expect(found).toHaveLength(2500);
  });

  it("does not call the backend without ids", async () => {
    expect(await noteTitles([])).toEqual([]);
    expect(invoke).not.toHaveBeenCalled();
  });
});
