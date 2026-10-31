import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { gzipSync } from "node:zlib";

const DEFAULT_BUDGET_KB = 250;
const BUILD_DIR = resolve("build");

function parseBudgetKb(args) {
  const index = args.indexOf("--budget-kb");
  if (index === -1) return DEFAULT_BUDGET_KB;
  const value = Number(args[index + 1]);
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error("--budget-kb requires a positive number");
  }
  return value;
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
  const budgetKb = parseBudgetKb(process.argv.slice(2));

  if (!existsSync(BUILD_DIR)) {
    throw new Error(`"${BUILD_DIR}" does not exist; run "npm run build" first`);
  }

  const js = measure(collectFiles(BUILD_DIR, ".js"));
  const css = measure(collectFiles(BUILD_DIR, ".css"));
  const jsTotal = js.reduce((sum, entry) => sum + entry.gzipBytes, 0);
  const cssTotal = css.reduce((sum, entry) => sum + entry.gzipBytes, 0);

  report("JS (gzip):", js);
  report("CSS (gzip):", css);
  console.log(`JS total:  ${toKb(jsTotal)} KB gzip (budget ${budgetKb} KB)`);
  console.log(`CSS total: ${toKb(cssTotal)} KB gzip (informational)`);

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
