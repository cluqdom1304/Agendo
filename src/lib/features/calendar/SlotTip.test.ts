// @vitest-environment jsdom
import { render, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";

import { TagDrag } from "../../stores/tagDrag.svelte";
import SlotTip from "./SlotTip.svelte";

afterEach(() => {
  delete (document as { elementFromPoint?: unknown }).elementFromPoint;
  document.body.innerHTML = "";
});

describe("SlotTip (F24)", () => {
  it("shows the quarter hour under the pointer while a task is dragged over a column", async () => {
    const column = document.createElement("div");
    column.dataset.slotDrop = "2026-03-11";
    column.getBoundingClientRect = () => ({ top: 0 }) as DOMRect;
    document.body.append(column);
    document.elementFromPoint = () => column;
    const drag = new TagDrag("data-slot-drop");
    const { container } = render(SlotTip, { drag });
    expect(container.querySelector("[data-slot-tip]")).toBeNull();
    drag.begin(
      new MouseEvent("pointerdown", {
        button: 0,
        clientX: 0,
        clientY: 0,
      }) as PointerEvent,
      { id: "a", title: "x" },
      () => {},
    );
    // 17:20 at 48 px/hour: the 17:15 quarter.
    window.dispatchEvent(
      new MouseEvent("pointermove", {
        clientX: 10,
        clientY: (17 + 20 / 60) * 48,
      }),
    );
    await waitFor(() =>
      expect(
        container.querySelector("[data-slot-tip]")?.textContent?.trim(),
      ).toBe("17:15"),
    );
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
  });
});
