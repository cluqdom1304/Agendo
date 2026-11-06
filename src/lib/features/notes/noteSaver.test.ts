import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { NoteSaver } from "./noteSaver";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe("NoteSaver", () => {
  it("saves once, 400 ms after the last change, with the last text", async () => {
    const save = vi.fn(async () => {});
    const saver = new NoteSaver(save);
    saver.schedule("a");
    await vi.advanceTimersByTimeAsync(300);
    saver.schedule("ab");
    await vi.advanceTimersByTimeAsync(399);
    expect(save).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith("ab");
    expect(saver.dirty).toBe(false);
  });

  it("flush saves immediately and cancels the timer", async () => {
    const save = vi.fn(async () => {});
    const saver = new NoteSaver(save);
    saver.schedule("x");
    await saver.flush();
    expect(save).toHaveBeenCalledWith("x");
    await vi.advanceTimersByTimeAsync(1000);
    expect(save).toHaveBeenCalledTimes(1);
  });

  it("flush with nothing pending does nothing", async () => {
    const save = vi.fn(async () => {});
    await new NoteSaver(save).flush();
    expect(save).not.toHaveBeenCalled();
  });

  it("never overlaps saves and keeps their order", async () => {
    const order: string[] = [];
    let release!: () => void;
    const first = new Promise<void>((resolve) => (release = resolve));
    const save = vi.fn(async (text: string) => {
      order.push(`start ${text}`);
      if (text === "1") await first;
      order.push(`end ${text}`);
    });
    const saver = new NoteSaver(save);
    saver.schedule("1");
    await vi.advanceTimersByTimeAsync(400);
    saver.schedule("2");
    await vi.advanceTimersByTimeAsync(400);
    expect(order).toEqual(["start 1"]);
    release();
    await saver.flush();
    expect(order).toEqual(["start 1", "end 1", "start 2", "end 2"]);
  });

  it("keeps the text after a failure and retries it on the next flush", async () => {
    const save = vi
      .fn<(text: string) => Promise<void>>()
      .mockRejectedValueOnce("DB: fallo")
      .mockResolvedValue();
    const saver = new NoteSaver(save);
    saver.schedule("texto");
    await saver.flush();
    expect(saver.dirty).toBe(true);
    await saver.flush();
    expect(save).toHaveBeenCalledTimes(2);
    expect(saver.dirty).toBe(false);
  });
});
