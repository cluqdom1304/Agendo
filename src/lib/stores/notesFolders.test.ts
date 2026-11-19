import { describe, expect, it } from "vitest";

import type { FolderWithCount } from "../types";
import { walkFolders } from "./notesFolders";

function folder(id: string, parentId: string | null): FolderWithCount {
  return { id, name: id, parentId, noteCount: 0 } as FolderWithCount;
}

const folders = [folder("a", null), folder("a1", "a"), folder("b", null)];

describe("walkFolders", () => {
  it("hides the children of a collapsed folder", () => {
    const rows = walkFolders(folders, () => false, false);
    expect(rows.map((row) => row.folder.id)).toEqual(["a", "b"]);
    expect(rows[0]).toMatchObject({ hasChildren: true, expanded: false });
  });

  it("shows the children of an expanded folder, indented", () => {
    const rows = walkFolders(folders, (id) => id === "a", false);
    expect(rows.map((row) => [row.folder.id, row.depth])).toEqual([
      ["a", 0],
      ["a1", 1],
      ["b", 0],
    ]);
  });

  it("lists everything when asked for the whole tree", () => {
    const rows = walkFolders(folders, () => false, true);
    expect(rows.map((row) => row.folder.id)).toEqual(["a", "a1", "b"]);
  });
});
