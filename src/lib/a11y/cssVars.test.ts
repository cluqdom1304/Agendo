import { describe, expect, it } from "vitest";

const sources = import.meta.glob(["../../**/*.svelte", "../../**/*.css"], {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

describe("css custom properties", () => {
  const defined = new Set<string>();
  for (const text of Object.values(sources)) {
    for (const match of text.matchAll(/(--[a-zA-Z0-9-]+)\s*:/g))
      defined.add(match[1]);
    for (const match of text.matchAll(/style:(--[a-zA-Z0-9-]+)/g))
      defined.add(match[1]);
  }

  it("every var() without a fallback names a property that exists", () => {
    const missing: string[] = [];
    for (const [file, text] of Object.entries(sources)) {
      for (const match of text.matchAll(/var\((--[a-zA-Z0-9-]+)\)/g)) {
        if (!defined.has(match[1])) missing.push(`${file}: ${match[1]}`);
      }
    }
    expect(missing).toEqual([]);
  });
});
