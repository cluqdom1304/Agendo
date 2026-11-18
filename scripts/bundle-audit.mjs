import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { gzipSync } from "node:zlib";

const CLIENT_DIR = resolve(".svelte-kit/output/client");
const MANIFEST = JSON.parse(
  readFileSync(resolve(CLIENT_DIR, ".vite/manifest.json"), "utf8"),
);

function gzipKb(file) {
  return (
    gzipSync(readFileSync(resolve(CLIENT_DIR, file)), { level: 9 }).length /
    1024
  );
}

// Walks only static imports: what the browser must download before first paint.
function collectStatic(key, seen) {
  if (seen.has(key)) return;
  const entry = MANIFEST[key];
  if (!entry) return;
  seen.add(key);
  for (const dep of entry.imports ?? []) collectStatic(dep, seen);
}

const roots = Object.keys(MANIFEST).filter(
  (key) =>
    key.endsWith("client-optimized/app.js") ||
    key.endsWith("runtime/client/entry.js"),
);
const initial = new Set();
for (const root of roots) collectStatic(root, initial);
// The root layout and the single page are loaded by app.js through dynamic import().
for (const key of Object.keys(MANIFEST)) {
  if (/client-optimized\/nodes\/[012]\.js$/.test(key))
    collectStatic(key, initial);
}

const rows = Object.entries(MANIFEST)
  .filter(([, entry]) => entry.file.endsWith(".js"))
  .map(([key, entry]) => ({
    key,
    kb: gzipKb(entry.file),
    initial: initial.has(key),
  }))
  .sort((a, b) => b.kb - a.kb);

const initialKb = rows
  .filter((r) => r.initial)
  .reduce((sum, r) => sum + r.kb, 0);
const lazyKb = rows.filter((r) => !r.initial).reduce((sum, r) => sum + r.kb, 0);

for (const row of rows) {
  console.log(
    `${row.kb.toFixed(1).padStart(8)} KB  ${row.initial ? "INITIAL" : "lazy   "}  ${row.key}`,
  );
}
console.log(`Initial load: ${initialKb.toFixed(1)} KB gzip`);
console.log(`Lazy chunks:  ${lazyKb.toFixed(1)} KB gzip`);
