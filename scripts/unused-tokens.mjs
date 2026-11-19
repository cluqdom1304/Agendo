// Lists the CSS custom properties that src/styles/tokens.css defines and nothing refers to.
// Informational by default; `--strict` exits with 1 when any is unused.
import { readdirSync, readFileSync } from "node:fs";
import { extname, join, resolve } from "node:path";

const SRC = resolve("src");
const TOKENS = resolve("src/styles/tokens.css");
const EXTENSIONS = new Set([".css", ".svelte", ".ts", ".js", ".html"]);

function collect(dir) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) found.push(...collect(path));
    else if (EXTENSIONS.has(extname(entry.name))) found.push(path);
  }
  return found;
}

const tokenSource = readFileSync(TOKENS, "utf8");
const defined = new Set(
  [...tokenSource.matchAll(/^\s*(--[\w-]+)\s*:/gm)].map((match) => match[1]),
);

// A token counts as used when `var(--x` appears anywhere, or when it is written in a script
// or a style attribute outside its own definition file.
const used = new Set();
for (const file of collect(SRC)) {
  const text = readFileSync(file, "utf8");
  for (const match of text.matchAll(/var\(\s*(--[\w-]+)/g)) used.add(match[1]);
  if (file !== TOKENS) {
    for (const name of defined) if (text.includes(name)) used.add(name);
  }
}

const unused = [...defined].filter((name) => !used.has(name)).sort();
console.log(`Tokens defined: ${defined.size}, unused: ${unused.length}`);
for (const name of unused) console.log(`  ${name}`);
if (unused.length > 0 && process.argv.includes("--strict")) process.exit(1);
