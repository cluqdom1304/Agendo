import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { gzipSync } from "node:zlib";

// D-98: the whole bundle and what the browser must load before the first paint.
const DEFAULT_BUDGET_KB = 340;
const DEFAULT_INITIAL_BUDGET_KB = 120;
const BUILD_DIR = resolve("build");
const MANIFEST = resolve(".svelte-kit/output/client/.vite/manifest.json");

function parseKb(args, flag, fallback) {
  const index = args.indexOf(flag);
  if (index === -1) return fallback;
  const value = Number(args[index + 1]);
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error(`${flag} requires a positive number`);
  }
  return value;
}

/** Files reached through static imports from the entry, the root layout and the page. */
function initialFiles() {
  const manifest = JSON.parse(readFileSync(MANIFEST, "utf8"));
  const seen = new Set();
  const visit = (key) => {
    const entry = manifest[key];
    if (!entry || seen.has(key)) return;
    seen.add(key);
    for (const dep of entry.imports ?? []) visit(dep);
  };
  for (const key of Object.keys(manifest)) {
    if (
      key.endsWith("client-optimized/app.js") ||
      key.endsWith("runtime/client/entry.js") ||
      /client-optimized\/nodes\/[012]\.js$/.test(key)
    ) {
      visit(key);
    }
  }
  return new Set([...seen].map((key) => manifest[key].file));
}

function collectFiles(dir, extension) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      found.push(...collectFiles(path, extension));
    } else if (entry.name.endsWith(extension)) {
      found.push(path);
    }
  }
  return found;
}

function measure(files) {
  return files
    .map((path) => ({
      path: relative(BUILD_DIR, path).replaceAll("\\", "/"),
      gzipBytes: gzipSync(readFileSync(path), { level: 9 }).length,
    }))
    .sort((a, b) => b.gzipBytes - a.gzipBytes);
}

function toKb(bytes) {
  return (bytes / 1024).toFixed(1);
}

function report(title, entries) {
  console.log(title);
  for (const { path, gzipBytes } of entries) {
    console.log(`  ${toKb(gzipBytes).padStart(8)} KB  ${path}`);
  }
}

function main() {
  const args = process.argv.slice(2);
  const budgetKb = parseKb(args, "--budget-kb", DEFAULT_BUDGET_KB);
  const initialBudgetKb = parseKb(
    args,
    "--initial-budget-kb",
    DEFAULT_INITIAL_BUDGET_KB,
  );

  if (!existsSync(BUILD_DIR) || !existsSync(MANIFEST)) {
    throw new Error(`"${BUILD_DIR}" does not exist; run "npm run build" first`);
  }

  const js = measure(collectFiles(BUILD_DIR, ".js"));
  const css = measure(collectFiles(BUILD_DIR, ".css"));
  const jsTotal = js.reduce((sum, entry) => sum + entry.gzipBytes, 0);
  const initial = initialFiles();
  const jsInitial = js
    .filter((entry) => initial.has(entry.path))
    .reduce((sum, entry) => sum + entry.gzipBytes, 0);
  const cssTotal = css.reduce((sum, entry) => sum + entry.gzipBytes, 0);

  report("JS (gzip):", js);
  report("CSS (gzip):", css);
  console.log(
    `JS initial: ${toKb(jsInitial)} KB gzip (budget ${initialBudgetKb} KB)`,
  );
  console.log(`JS total:   ${toKb(jsTotal)} KB gzip (budget ${budgetKb} KB)`);
  console.log(`CSS total:  ${toKb(cssTotal)} KB gzip (informational)`);

  if (initial.size === 0 || jsInitial === 0) {
    console.error("FAIL: the initial load could not be found in the manifest");
    process.exitCode = 1;
    return;
  }
  if (jsInitial > initialBudgetKb * 1024) {
    console.error(
      `FAIL: the initial load exceeds the budget of ${initialBudgetKb} KB gzip`,
    );
    process.exitCode = 1;
    return;
  }
  if (jsTotal > budgetKb * 1024) {
    console.error(`FAIL: JS bundle exceeds the budget of ${budgetKb} KB gzip`);
    process.exitCode = 1;
    return;
  }
  console.log("OK: JS bundle is within budget");
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}
