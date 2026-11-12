// The `#tag` shortcut of the note editor: typing `#` and a few letters offers the tags that
// match. Pure functions, apart from the editor, so they are easy to test.

import type { Tag } from "../../types";

/** A `#` at the start of a line or after a space, followed by the letters typed so far. */
const QUERY = /(?:^|\s)#([\p{L}\p{N}_-]{0,40})$/u;

/** The text typed after the `#` that ends `textBefore` (the text before the cursor), if any. */
export function findTagQuery(textBefore: string): string | null {
  const found = QUERY.exec(textBefore);
  return found ? (found[1] ?? "") : null;
}

function fold(text: string): string {
  return text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();
}

/** Tags whose name starts with the query first, then the ones that contain it. */
export function matchTags(
  tags: readonly Tag[],
  query: string,
  limit = 6,
): Tag[] {
  const wanted = fold(query);
  if (wanted === "") return tags.slice(0, limit);
  const starts: Tag[] = [];
  const inside: Tag[] = [];
  for (const tag of tags) {
    const name = fold(tag.name);
    if (name.startsWith(wanted)) starts.push(tag);
    else if (name.includes(wanted)) inside.push(tag);
  }
  return [...starts, ...inside].slice(0, limit);
}
