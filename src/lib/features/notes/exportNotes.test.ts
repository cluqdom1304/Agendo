import { describe, expect, it, vi } from "vitest";

import { exportSelection, type ExportDeps } from "./exportNotes";

function deps(path: string | null) {
  return {
    pickFolder: vi.fn(async () => path),
    exportNotes: vi.fn(async () => ({ notes: 2, files: 0 })),
  } satisfies ExportDeps;
}

describe("exportSelection", () => {
  it("writes the selection into the chosen folder and says how many notes", async () => {
    const d = deps("C:/salida");
    const notice = await exportSelection("html", "folder", "f1", d);
    expect(d.exportNotes).toHaveBeenCalledWith(
      "html",
      "folder",
      "f1",
      "C:/salida",
    );
    expect(notice).toEqual({ kind: "info", message: "2 notas exportadas." });
  });

  it("does nothing when the folder dialog is cancelled", async () => {
    const d = deps(null);
    expect(await exportSelection("markdown", "note", "n1", d)).toBeNull();
    expect(d.exportNotes).not.toHaveBeenCalled();
  });

  it("explains a rejection with the backend's message", async () => {
    const d = deps("C:/salida");
    d.exportNotes.mockRejectedValue("IO: disco lleno");
    const notice = await exportSelection("markdown", "note", "n1", d);
    expect(notice?.kind).toBe("error");
    expect(notice?.message).toContain("disco lleno");
  });
});
