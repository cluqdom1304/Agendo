import { describe, expect, it } from "vitest";

const sources = import.meta.glob("../../**/*.svelte", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

// app.css loads Tailwind, which turns any of these words used as a class into a utility in
// a layer the component's own rule does not always override (a `.fixed` row became
// `position: fixed` in «Planificar mi día»). Component classes must not take these names.
const RESERVED = [
  "absolute",
  "fixed",
  "relative",
  "static",
  "sticky",
  "hidden",
  "invisible",
  "visible",
  "collapse",
  "contents",
  "container",
  "table",
  "ring",
  "shadow",
  "border",
  "outline",
  "rounded",
  "transform",
  "filter",
  "blur",
  "truncate",
];

describe("component class names", () => {
  it("never reuse a Tailwind utility that changes layout or drawing", () => {
    const clashes: string[] = [];
    for (const [file, text] of Object.entries(sources)) {
      for (const match of text.matchAll(/\bclass="([^"{]*)"/g)) {
        for (const name of match[1].split(/\s+/)) {
          if (RESERVED.includes(name)) clashes.push(`${file}: ${name}`);
        }
      }
      for (const match of text.matchAll(/\bclass:([a-zA-Z-]+)/g)) {
        if (RESERVED.includes(match[1])) clashes.push(`${file}: ${match[1]}`);
      }
    }
    expect(clashes).toEqual([]);
  });
});
