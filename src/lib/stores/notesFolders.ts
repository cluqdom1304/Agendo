import type { FolderWithCount } from "../types";

export interface FolderRow {
  folder: FolderWithCount;
  depth: number;
  hasChildren: boolean;
  expanded: boolean;
}

/** Folders in tree order; with `all` false, children show only under an expanded folder. */
export function walkFolders(
  folders: readonly FolderWithCount[],
  isExpanded: (id: string) => boolean,
  all: boolean,
): FolderRow[] {
  const children = new Map<string | null, FolderWithCount[]>();
  for (const folder of folders) {
    const list = children.get(folder.parentId) ?? [];
    list.push(folder);
    children.set(folder.parentId, list);
  }
  const result: FolderRow[] = [];
  const walk = (parent: string | null, depth: number) => {
    for (const folder of children.get(parent) ?? []) {
      const hasChildren = children.has(folder.id);
      const expanded = hasChildren && isExpanded(folder.id);
      result.push({ folder, depth, hasChildren, expanded });
      if (all || expanded) walk(folder.id, depth + 1);
    }
  };
  walk(null, 0);
  return result;
}
