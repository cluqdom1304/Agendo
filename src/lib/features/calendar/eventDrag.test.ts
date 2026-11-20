import { describe, expect, it, vi } from "vitest";

import type { EventOccurrence } from "../../types";
import { EventDrag } from "./eventDrag.svelte";

const occurrence: EventOccurrence = {
  eventId: "e1",
  title: "Reunión",
  startDate: "2026-03-10",
  startTime: "10:00",
  endDate: "2026-03-10",
  endTime: "11:00",
  allDay: false,
  recurring: false,
  tagIds: [],
};

const HOUR_PX = 48;
const pointer = (clientX: number, clientY: number, button = 0) =>
  ({ clientX, clientY, button }) as PointerEvent;

function setup(dateAt: (x: number, y: number) => string | null = () => null) {
  const onMove = vi.fn();
  const onResize = vi.fn();
  const drag = new EventDrag<EventOccurrence>({
    hourPx: HOUR_PX,
    dateAt,
    onMove,
    onResize,
  });
  return { drag, onMove, onResize };
}

describe("EventDrag", () => {
  it("a press that barely moves is a click, not a drag", () => {
    const { drag, onMove } = setup();
    drag.begin(pointer(100, 100), occurrence, "move", "2026-03-10", 600, 60);
    drag.pointerMove(pointer(101, 101));
    expect(drag.preview).toBeNull();
    drag.pointerUp(pointer(101, 101));
    expect(onMove).not.toHaveBeenCalled();
    expect(drag.consumeClick()).toBe(false);
  });

  it("moves to the quarter hour under the pointer and to the column it is over", () => {
    const { drag, onMove } = setup(() => "2026-03-12");
    drag.begin(pointer(100, 100), occurrence, "move", "2026-03-10", 600, 60);
    // One hour down.
    drag.pointerMove(pointer(300, 100 + HOUR_PX));
    expect(drag.preview).toMatchObject({
      date: "2026-03-12",
      start: 660,
      end: 720,
    });
    drag.pointerUp(pointer(300, 100 + HOUR_PX));
    expect(onMove).toHaveBeenCalledWith(occurrence, "2026-03-12", 660);
    expect(drag.preview).toBeNull();
    expect(drag.consumeClick()).toBe(true);
    expect(drag.consumeClick()).toBe(false);
  });

  it("does not run past the end of the day", () => {
    const { drag, onMove } = setup();
    drag.begin(pointer(0, 0), occurrence, "move", "2026-03-10", 600, 60);
    drag.pointerUp(pointer(0, 0));
    drag.begin(pointer(0, 0), occurrence, "move", "2026-03-10", 600, 60);
    drag.pointerMove(pointer(0, 20 * HOUR_PX));
    drag.pointerUp(pointer(0, 20 * HOUR_PX));
    expect(onMove).toHaveBeenCalledWith(occurrence, "2026-03-10", 23 * 60);
  });

  it("resizing moves the end and keeps the start", () => {
    const { drag, onResize } = setup();
    drag.begin(pointer(0, 0), occurrence, "resize", "2026-03-10", 600, 60);
    drag.pointerMove(pointer(0, HOUR_PX / 2));
    expect(drag.preview).toMatchObject({ start: 600, end: 690 });
    drag.pointerUp(pointer(0, HOUR_PX / 2));
    expect(onResize).toHaveBeenCalledWith(occurrence, 690);
  });

  it("the end never goes before the start", () => {
    const { drag, onResize } = setup();
    drag.begin(pointer(0, 0), occurrence, "resize", "2026-03-10", 600, 60);
    drag.pointerMove(pointer(0, -5 * HOUR_PX));
    drag.pointerUp(pointer(0, -5 * HOUR_PX));
    expect(onResize).toHaveBeenCalledWith(occurrence, 615);
  });

  it("cancel drops the drag without saving", () => {
    const { drag, onMove } = setup();
    drag.begin(pointer(0, 0), occurrence, "move", "2026-03-10", 600, 60);
    drag.pointerMove(pointer(0, 100));
    drag.cancel();
    drag.pointerUp(pointer(0, 100));
    expect(onMove).not.toHaveBeenCalled();
    expect(drag.preview).toBeNull();
  });

  it("ignores other buttons", () => {
    const { drag, onMove } = setup();
    drag.begin(pointer(0, 0, 2), occurrence, "move", "2026-03-10", 600, 60);
    drag.pointerMove(pointer(0, 100));
    drag.pointerUp(pointer(0, 100));
    expect(onMove).not.toHaveBeenCalled();
  });
});
