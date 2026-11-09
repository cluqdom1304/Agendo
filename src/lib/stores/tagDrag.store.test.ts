// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";

import { TagDrag } from "./tagDrag.svelte";

const press = (x = 0, y = 0) =>
  ({ button: 0, clientX: x, clientY: y }) as PointerEvent;

function move(x: number, y: number) {
  window.dispatchEvent(
    new MouseEvent("pointermove", { clientX: x, clientY: y }),
  );
}

function release() {
  window.dispatchEvent(new MouseEvent("pointerup"));
}

/** Makes `elementFromPoint` answer with a drop target for the given key (jsdom has none). */
function dropZoneAt(key: string | null) {
  const zone = document.createElement("div");
  if (key !== null) zone.setAttribute("data-tag-drop", key);
  document.body.append(zone);
  document.elementFromPoint = () => zone;
  return zone;
}

afterEach(async () => {
  // Lets a drag's "swallow the next click" listener expire, as it does in the app.
  await new Promise((resolve) => setTimeout(resolve, 0));
  document.body.innerHTML = "";
  document.body.className = "";
  // @ts-expect-error jsdom does not define it; remove the stub.
  delete document.elementFromPoint;
});

describe("TagDrag", () => {
  const subject = { id: "t1", title: "Estudiar" };

  it("does not start until the pointer moves a few pixels", () => {
    const drag = new TagDrag();
    drag.begin(press(), subject, vi.fn());
    move(2, 2);
    expect(drag.active).toBeNull();
    move(20, 0);
    expect(drag.active?.title).toBe("Estudiar");
    release();
  });

  it("drops on the tag under the pointer", () => {
    const drag = new TagDrag();
    const onDrop = vi.fn();
    dropZoneAt("mat");
    drag.begin(press(), subject, onDrop);
    move(30, 30);
    expect(drag.active?.overKey).toBe("mat");
    expect(document.body.classList.contains("tag-dragging")).toBe(true);
    release();
    expect(onDrop).toHaveBeenCalledWith("mat");
    expect(drag.active).toBeNull();
    expect(document.body.classList.contains("tag-dragging")).toBe(false);
  });

  it("drops nothing when released outside a tag", () => {
    const drag = new TagDrag();
    const onDrop = vi.fn();
    dropZoneAt(null);
    drag.begin(press(), subject, onDrop);
    move(30, 30);
    expect(drag.active?.overKey).toBeNull();
    release();
    expect(onDrop).not.toHaveBeenCalled();
  });

  it("is cancelled with Escape", () => {
    const drag = new TagDrag();
    const onDrop = vi.fn();
    dropZoneAt("mat");
    drag.begin(press(), subject, onDrop);
    move(30, 30);
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(drag.active).toBeNull();
    release();
    expect(onDrop).not.toHaveBeenCalled();
  });

  it("leaves a plain click alone but swallows the click that ends a drag", () => {
    const drag = new TagDrag();
    const clicks = vi.fn();
    document.body.addEventListener("click", clicks);

    drag.begin(press(), subject, vi.fn());
    release();
    document.body.click();
    expect(clicks).toHaveBeenCalledTimes(1);

    dropZoneAt("mat");
    drag.begin(press(), subject, vi.fn());
    move(40, 40);
    release();
    document.body.click();
    expect(clicks).toHaveBeenCalledTimes(1);
  });

  it("ignores other mouse buttons", () => {
    const drag = new TagDrag();
    drag.begin({ ...press(), button: 2 } as PointerEvent, subject, vi.fn());
    move(40, 40);
    expect(drag.active).toBeNull();
  });
});
