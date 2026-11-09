// @vitest-environment jsdom
import { describe, expect, it } from "vitest";

import { applyTheme, parseTheme } from "./theme";

describe("theme", () => {
  it("falls back to the system theme for unknown values", () => {
    expect(parseTheme(null)).toBe("system");
    expect(parseTheme("sepia")).toBe("system");
    expect(parseTheme("dark")).toBe("dark");
    expect(parseTheme("light")).toBe("light");
  });

  it("forces light or dark through data-theme and lets the system decide otherwise", () => {
    const root = document.createElement("html");
    applyTheme("dark", root);
    expect(root.getAttribute("data-theme")).toBe("dark");
    applyTheme("light", root);
    expect(root.getAttribute("data-theme")).toBe("light");
    applyTheme("system", root);
    expect(root.hasAttribute("data-theme")).toBe(false);
  });
});
