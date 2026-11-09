// @vitest-environment jsdom
import { describe, expect, it } from "vitest";

import { findShortcut, focusContextOf } from "./keymap";

const press = (init: KeyboardEventInit) => new KeyboardEvent("keydown", init);

describe("findShortcut", () => {
  it("maps the shortcuts of SPEC §9 for the list", () => {
    expect(findShortcut(press({ key: " " }), "list")).toBe("toggleComplete");
    expect(findShortcut(press({ key: "Delete" }), "list")).toBe("deleteTask");
    expect(findShortcut(press({ key: "z", ctrlKey: true }), "list")).toBe(
      "undo",
    );
    expect(findShortcut(press({ key: "ArrowDown" }), "list")).toBe(
      "selectNext",
    );
  });

  it("switches views with Ctrl+1…4 from anywhere", () => {
    const ids = ["viewList", "viewDay", "viewWeek", "viewMonth"];
    ["1", "2", "3", "4"].forEach((key, index) => {
      for (const context of ["list", "title", "other"] as const) {
        expect(findShortcut(press({ key, ctrlKey: true }), context)).toBe(
          ids[index],
        );
      }
    });
    expect(findShortcut(press({ key: "1" }), "list")).toBeNull();
  });

  it("tells Ctrl+N from Ctrl+Shift+N, with either letter case", () => {
    expect(findShortcut(press({ key: "n", ctrlKey: true }), "list")).toBe(
      "newTask",
    );
    expect(
      findShortcut(press({ key: "N", ctrlKey: true, shiftKey: true }), "list"),
    ).toBe("newTaskToday");
  });

  it("works while typing for Ctrl+N and Enter/Esc only inside a title", () => {
    expect(findShortcut(press({ key: "n", ctrlKey: true }), "title")).toBe(
      "newTask",
    );
    expect(findShortcut(press({ key: "Enter" }), "title")).toBe("confirmTitle");
    expect(findShortcut(press({ key: "Escape" }), "title")).toBe("cancelTitle");
    expect(findShortcut(press({ key: "Enter" }), "list")).toBeNull();
  });

  it("never steals typing keys from inputs or other controls", () => {
    for (const key of [" ", "Delete", "ArrowUp"]) {
      expect(findShortcut(press({ key }), "title")).toBeNull();
      expect(findShortcut(press({ key }), "other")).toBeNull();
    }
    expect(
      findShortcut(press({ key: "z", ctrlKey: true }), "title"),
    ).toBeNull();
  });

  it("ignores combinations with Alt", () => {
    expect(
      findShortcut(press({ key: "n", ctrlKey: true, altKey: true }), "list"),
    ).toBeNull();
  });
});

describe("focusContextOf", () => {
  it("recognises title inputs, rows, the page and other controls", () => {
    const title = document.createElement("input");
    title.setAttribute("data-title-input", "");
    const row = document.createElement("div");
    row.setAttribute("data-task-row", "");
    const button = document.createElement("button");
    expect(focusContextOf(title)).toBe("title");
    expect(focusContextOf(row)).toBe("list");
    expect(focusContextOf(document.body)).toBe("list");
    expect(focusContextOf(button)).toBe("other");
  });
});
