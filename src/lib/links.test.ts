import { describe, expect, it } from "vitest";

import { sanitizeLink } from "./links";

describe("sanitizeLink", () => {
  it.each([
    ["https://example.com", "https://example.com/"],
    ["http://example.com/a?b=1#c", "http://example.com/a?b=1#c"],
    ["  HTTPS://Example.com/x  ", "https://example.com/x"],
    ["mailto:ana@example.com", "mailto:ana@example.com"],
    ["ana@example.com", "mailto:ana@example.com"],
    ["example.com/ruta", "https://example.com/ruta"],
  ])("accepts %j", (input, expected) => {
    expect(sanitizeLink(input)).toBe(expected);
  });

  it.each([
    "javascript:alert(1)",
    "JavaScript:alert(1)",
    " javascript:alert(1)",
    "java\tscript:alert(1)",
    "java\nscript:alert(1)",
    "file:///C:/Windows/System32",
    "data:text/html,<script>alert(1)</script>",
    "vbscript:msgbox(1)",
    "ftp://example.com",
    "tauri://localhost",
    "ipc:foo",
    "https://",
    "http:///",
    "mailto:",
    "",
    "   ",
    "dos palabras",
  ])("rejects %j", (input) => {
    expect(sanitizeLink(input)).toBeNull();
  });
});
