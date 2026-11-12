// @vitest-environment jsdom
import { describe, expect, it } from "vitest";

import { resolveTarget } from "./target";

function dom(html: string): Element {
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.replaceChildren(host);
  return host;
}

const at = (host: Element, selector: string) => {
  const el = host.querySelector(selector);
  if (!el) throw new Error(`no ${selector}`);
  return resolveTarget(el);
};

describe("resolveTarget", () => {
  it("leaves text fields to the browser, but not checkboxes", () => {
    const host = dom(
      '<input id="a" type="text"><textarea id="b"></textarea><input id="c" type="checkbox" data-task-id="t1">',
    );
    expect(at(host, "#a")).toEqual({ kind: "field" });
    expect(at(host, "#b")).toEqual({ kind: "field" });
    expect(at(host, "#c")).toEqual({ kind: "task", taskId: "t1" });
  });

  it("recognises tasks, notes, folders and tags from the nearest ancestor", () => {
    const host = dom(
      `<div data-task-id="t1"><span id="a">x</span></div>
       <div data-note-id="n1"><span id="b">x</span></div>
       <div data-folder-id="f1"><span id="c">x</span></div>
       <div data-tag-id="g1"><span id="d">x</span></div>
       <div data-note-tag-id="g2"><span id="e">x</span></div>`,
    );
    expect(at(host, "#a")).toEqual({ kind: "task", taskId: "t1" });
    expect(at(host, "#b")).toEqual({ kind: "note", noteId: "n1" });
    expect(at(host, "#c")).toEqual({ kind: "folder", folderId: "f1" });
    expect(at(host, "#d")).toEqual({ kind: "tag", tagId: "g1" });
    expect(at(host, "#e")).toEqual({ kind: "note-tag", tagId: "g2" });
  });

  it("tells links and task references of the editor from a task row", () => {
    const host = dom(
      `<div class="tiptap">
         <span data-note-link data-note-id="n9"><i id="a"></i></span>
         <div data-task-ref data-task-id="t9"><i id="b"></i></div>
         <p id="c">text</p>
       </div>`,
    );
    expect(at(host, "#a")).toMatchObject({ kind: "note-link", noteId: "n9" });
    expect(at(host, "#b")).toMatchObject({ kind: "task-ref", taskId: "t9" });
    expect(at(host, "#c")).toEqual({ kind: "editor" });
  });

  it("falls back to the section background, with the day under the pointer", () => {
    const host = dom(
      `<div data-menu-area="agenda"><div data-date="2026-03-12"><b id="a"></b></div><i id="b"></i></div>
       <div data-menu-area="notes"><i id="c"></i></div>
       <div id="d"></div>`,
    );
    expect(at(host, "#a")).toEqual({ kind: "agenda", date: "2026-03-12" });
    expect(at(host, "#b")).toEqual({ kind: "agenda", date: null });
    expect(at(host, "#c")).toEqual({ kind: "notes" });
    expect(at(host, "#d")).toEqual({ kind: "none" });
  });

  it("offers nothing on the background of a dialog or on the menu itself", () => {
    const host = dom(
      `<div data-menu-area="agenda"><div role="dialog"><i id="a"></i></div></div>
       <div data-context-menu data-task-id="t1"><i id="b"></i></div>`,
    );
    expect(at(host, "#a")).toEqual({ kind: "none" });
    expect(at(host, "#b")).toEqual({ kind: "none" });
  });
});
