// Syntax colouring of code blocks (F12-03), without a highlighting library (D-51): one small
// tokenizer that knows comments, strings, numbers and the keywords of a few languages. It only
// draws; the document keeps the plain text.
import { Extension } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

export type TokenKind = "comment" | "string" | "number" | "keyword" | "type";

export interface Token {
  from: number;
  to: number;
  kind: TokenKind;
}

const words = (text: string) => new Set(text.split(" "));

const KEYWORDS: Record<string, Set<string>> = {
  js: words(
    "const let var function return if else for while do switch case break continue new class extends import export from default async await try catch finally throw typeof instanceof in of this super null undefined true false interface type enum implements public private protected readonly static yield void",
  ),
  python: words(
    "def class return if elif else for while in is not and or import from as with try except finally raise pass break continue lambda yield None True False global nonlocal async await self",
  ),
  rust: words(
    "fn let mut const static struct enum impl trait pub use mod crate self Self super match if else for while loop break continue return as in where move ref async await dyn unsafe type true false",
  ),
  sql: words(
    "select from where and or not insert into values update set delete create table drop alter join left right inner outer on group by order having limit offset as distinct null is in like between union case when then else end primary key foreign references index unique default",
  ),
  css: words("important inherit initial unset none auto"),
  bash: words(
    "if then else elif fi for while do done case esac function in return exit export local echo cd ls grep cat",
  ),
  json: words("true false null"),
  html: words(""),
};

const ALIASES: Record<string, string> = {
  javascript: "js",
  jsx: "js",
  ts: "js",
  typescript: "js",
  tsx: "js",
  py: "python",
  rs: "rust",
  sh: "bash",
  shell: "bash",
  zsh: "bash",
  xml: "html",
  svelte: "html",
  scss: "css",
};

/** The language a fence names (```ts), normalised; unknown or absent ones are `generic`. */
export function languageOf(name: string | null | undefined): string {
  const key = (name ?? "").trim().toLowerCase();
  const known = ALIASES[key] ?? key;
  return known in KEYWORDS ? known : "generic";
}

const GENERIC = new Set(
  [...Object.values(KEYWORDS)].flatMap((set) => [...set]).filter(Boolean),
);

// Comments, strings, numbers and words, in this order of precedence at each position.
const SCAN =
  /(\/\*[\s\S]*?(?:\*\/|$))|(\/\/[^\n]*|#[^\n]*|--[^\n]*|<!--[\s\S]*?(?:-->|$))|("(?:\\.|[^"\\\n])*"?|'(?:\\.|[^'\\\n])*'?|`(?:\\.|[^`\\])*`?)|(\b0x[\da-f]+\b|\b\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?\b)|([A-Za-z_$][\w$]*)/gi;

/** Which comment styles a language has; `#` is not a comment in C-like code. */
function commentAllowed(language: string, marker: string): boolean {
  if (marker.startsWith("//") || marker.startsWith("/*")) {
    return !["python", "bash", "sql"].includes(language);
  }
  if (marker.startsWith("#")) {
    return ["python", "bash", "generic"].includes(language);
  }
  if (marker.startsWith("--")) return ["sql", "generic"].includes(language);
  if (marker.startsWith("<!--")) return ["html", "generic"].includes(language);
  return true;
}

/** Colours `source`; positions are offsets into it. */
export function tokenize(source: string, language: string): Token[] {
  const keywords = KEYWORDS[language] ?? GENERIC;
  const caseless = language === "sql";
  const tokens: Token[] = [];
  SCAN.lastIndex = 0;
  for (let match = SCAN.exec(source); match; match = SCAN.exec(source)) {
    const from = match.index;
    const to = from + match[0].length;
    if (match[1] !== undefined) {
      if (commentAllowed(language, "/*")) {
        tokens.push({ from, to, kind: "comment" });
      } else {
        SCAN.lastIndex = from + 1;
      }
    } else if (match[2] !== undefined) {
      if (commentAllowed(language, match[2])) {
        tokens.push({ from, to, kind: "comment" });
      } else {
        // Not a comment here: rescan after its first character.
        SCAN.lastIndex = from + 1;
      }
    } else if (match[3] !== undefined) {
      tokens.push({ from, to, kind: "string" });
    } else if (match[4] !== undefined) {
      tokens.push({ from, to, kind: "number" });
    } else if (match[5] !== undefined) {
      const word = match[5];
      if (keywords.has(caseless ? word.toLowerCase() : word)) {
        tokens.push({ from, to, kind: "keyword" });
      } else if (/^[A-Z][a-z]/.test(word) && language !== "sql") {
        tokens.push({ from, to, kind: "type" });
      }
    }
  }
  return tokens;
}

const key = new PluginKey<DecorationSet>("codeHighlight");

function decorate(doc: PmNode): DecorationSet {
  const decorations: Decoration[] = [];
  doc.descendants((node, pos) => {
    if (node.type.name !== "codeBlock") return true;
    const language = languageOf(node.attrs.language as string | null);
    const start = pos + 1;
    for (const token of tokenize(node.textContent, language)) {
      decorations.push(
        Decoration.inline(start + token.from, start + token.to, {
          class: `tok-${token.kind}`,
        }),
      );
    }
    return false;
  });
  return DecorationSet.create(doc, decorations);
}

export const CodeHighlight = Extension.create({
  name: "codeHighlight",
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key,
        state: {
          init: (_config, state) => decorate(state.doc),
          apply: (tr, old) => (tr.docChanged ? decorate(tr.doc) : old),
        },
        props: { decorations: (state) => key.getState(state) },
      }),
    ];
  },
});
