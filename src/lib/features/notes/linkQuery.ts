// The `[[note]]` shortcut of the note editor: typing `[[` and a few letters offers the notes
// that match. Pure, so it is easy to test.

/** `[[` followed by the letters typed so far, with no bracket in between. */
const QUERY = /\[\[([^[\]]{0,80})$/u;

/** The text typed after the `[[` that ends `textBefore` (the text before the cursor), if any. */
export function findLinkQuery(textBefore: string): string | null {
  const found = QUERY.exec(textBefore);
  return found ? (found[1] ?? "") : null;
}

/**
 * The search box understands `tag:`, `in:` and `is:`: what is typed between the brackets is
 * plain text, so those characters are dropped before it is searched.
 */
export function plainQuery(text: string): string {
  return text.replace(/[:"]/g, " ").replace(/\s+/g, " ").trim();
}
