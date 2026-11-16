// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import NoteHistory from "./NoteHistory.svelte";
import { FakeNotesApi, note } from "./testing";

const doc = (...texts: string[]) =>
  JSON.stringify({
    type: "doc",
    content: texts.map((text) => ({
      type: "paragraph",
      content: [{ type: "text", text }],
    })),
  });

async function mount(versions: number) {
  const api = new FakeNotesApi();
  const current = note("n1", { title: "Apuntes", contentJson: doc("a", "c") });
  api.notes = [current];
  for (let index = 0; index < versions; index += 1) {
    api.versions.push({
      id: `v${index}`,
      noteId: "n1",
      title: "Apuntes",
      contentJson: doc("a", "b"),
      createdAt: "2026-03-10T10:00:00Z",
    });
  }
  const notes = new NotesStore(api);
  await notes.load();
  let closed = false;
  render(NoteHistory, {
    notes,
    note: current,
    onClose: () => (closed = true),
  });
  return { api, notes, closed: () => closed };
}

describe("NoteHistory", () => {
  it("says so when there are no versions", async () => {
    await mount(0);
    await waitFor(() =>
      expect(
        screen.getByText(/aún no tiene versiones anteriores/),
      ).toBeTruthy(),
    );
  });

  it("compares a version with the note and restores it", async () => {
    const { notes, closed } = await mount(1);
    const version = await screen.findByRole("button", { name: /Apuntes/ });
    await fireEvent.click(version);
    await waitFor(() =>
      expect(
        screen.getByText("2 diferencias respecto a la nota actual."),
      ).toBeTruthy(),
    );
    expect(screen.getByText("b")).toBeTruthy();
    const before = notes.revision;
    await fireEvent.click(
      screen.getByRole("button", { name: "Restaurar esta versión" }),
    );
    await waitFor(() => expect(closed()).toBe(true));
    expect(notes.revision).toBe(before + 1);
    expect(notes.open).toBeNull();
    expect(notes.notice?.kind).toBe("info");
  });
});
