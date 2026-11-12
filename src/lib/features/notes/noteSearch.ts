// Find inside the open note (F12-04, Ctrl+F): a ProseMirror plugin that marks every match and
// the current one. The text is compared ignoring case and accents, as the global search does.
import { Extension } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";
import { type EditorState, Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

export interface Match {
  from: number;
  to: number;
}

interface SearchState {
  query: string;
  /** Index of the current match, or -1. */
  current: number;
  matches: Match[];
  decorations: DecorationSet;
}

// Same length before and after folding, so offsets of the folded text are offsets of the
// original: each character is folded on its own.
function fold(text: string): string {
  let folded = "";
  for (const char of text) {
    const plain = char.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
    folded += plain.length === char.length ? plain : char.toLowerCase();
  }
  return folded;
}

/** Every place `query` appears in the text of `doc`, in order. Matches never span blocks. */
export function findMatches(doc: PmNode, query: string): Match[] {
  const needle = fold(query);
  if (needle === "") return [];
  const found: Match[] = [];
  doc.descendants((node, pos) => {
    if (!node.isTextblock) return true;
    const text = node.textBetween(0, node.content.size, undefined, "￼");
    const haystack = fold(text);
    let at = haystack.indexOf(needle);
    while (at !== -1) {
      // Inline atoms count as one position, as in the document.
      found.push({ from: pos + 1 + at, to: pos + 1 + at + needle.length });
      at = haystack.indexOf(needle, at + needle.length);
    }
    return false;
  });
  return found;
}

const key = new PluginKey<SearchState>("noteSearch");

const EMPTY: SearchState = {
  query: "",
  current: -1,
  matches: [],
  decorations: DecorationSet.empty,
};

function build(doc: PmNode, query: string, wanted: number): SearchState {
  const matches = findMatches(doc, query);
  const current =
    matches.length === 0
      ? -1
      : Math.min(Math.max(wanted, 0), matches.length - 1);
  return {
    query,
    current,
    matches,
    decorations: DecorationSet.create(
      doc,
      matches.map((match, index) =>
        Decoration.inline(match.from, match.to, {
          class: index === current ? "find-match find-current" : "find-match",
        }),
      ),
    ),
  };
}

interface SetMeta {
  query: string;
  current: number;
}

export const NoteSearch = Extension.create({
  name: "noteSearch",
  addProseMirrorPlugins() {
    return [
      new Plugin<SearchState>({
        key,
        state: {
          init: () => EMPTY,
          apply(tr, old) {
            const meta = tr.getMeta(key) as SetMeta | undefined;
            if (meta) return build(tr.doc, meta.query, meta.current);
            // The document changed under an open search: keep looking for the same text.
            if (tr.docChanged && old.query !== "") {
              return build(tr.doc, old.query, old.current);
            }
            return old;
          },
        },
        props: { decorations: (state) => key.getState(state)?.decorations },
      }),
    ];
  },
});

/** The state of the search plugin in an editor state, for the editor handle. */
export function searchStateOf(state: EditorState): {
  count: number;
  current: number;
  match: Match | null;
} {
  const found = key.getState(state) ?? EMPTY;
  return {
    count: found.matches.length,
    current: found.current,
    match: found.matches[found.current] ?? null,
  };
}

export function searchTransaction(
  state: EditorState,
  query: string,
  current: number,
) {
  return state.tr.setMeta(key, { query, current } satisfies SetMeta);
}
