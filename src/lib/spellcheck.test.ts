// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";

import { muteSpellcheckOnBlur } from "./spellcheck";

let stop: (() => void) | null = null;
afterEach(() => {
  stop?.();
  stop = null;
  document.body.innerHTML = "";
});

function setup(html: string) {
  document.body.innerHTML = html;
  stop = muteSpellcheckOnBlur(document);
}

describe("muteSpellcheckOnBlur", () => {
  it("turns the checker off when a textarea loses focus and on again at focus", () => {
    setup('<textarea id="a"></textarea><button id="b">x</button>');
    const area = document.getElementById("a") as HTMLTextAreaElement;
    area.focus();
    document.getElementById("b")?.focus();
    expect(area.getAttribute("spellcheck")).toBe("false");
    area.focus();
    expect(area.getAttribute("spellcheck")).toBe("true");
  });

  it("does the same for editable regions (the notes editor)", () => {
    setup(
      '<div id="e" contenteditable="true" tabindex="0"></div><button id="b">x</button>',
    );
    const editor = document.getElementById("e") as HTMLElement;
    editor.focus();
    document.getElementById("b")?.focus();
    expect(editor.getAttribute("spellcheck")).toBe("false");
  });

  it("leaves alone a field that already had the checker off", () => {
    setup(
      '<textarea id="a" spellcheck="false"></textarea><button id="b">x</button>',
    );
    const area = document.getElementById("a") as HTMLTextAreaElement;
    area.focus();
    document.getElementById("b")?.focus();
    area.focus();
    expect(area.getAttribute("spellcheck")).toBe("false");
    expect(area.dataset.spellMuted).toBeUndefined();
  });
});
