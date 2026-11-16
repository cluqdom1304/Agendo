// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import NoteReminderDialog from "./NoteReminderDialog.svelte";
import { FakeNotesApi } from "./testing";

async function mount() {
  const api = new FakeNotesApi();
  const notes = new NotesStore(api);
  await notes.load();
  let closed = false;
  render(NoteReminderDialog, {
    notes,
    noteId: "n1",
    onClose: () => (closed = true),
  });
  return { api, closed: () => closed };
}

describe("NoteReminderDialog", () => {
  it("starts tomorrow at 09:00 and saves a future reminder as a UTC instant", async () => {
    const { api, closed } = await mount();
    await waitFor(() =>
      expect(screen.getByText("Esta nota no tiene recordatorio.")).toBeTruthy(),
    );
    expect((screen.getByLabelText("Hora") as HTMLInputElement).value).toBe(
      "09:00",
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Guardar recordatorio" }),
    );
    await waitFor(() => expect(closed()).toBe(true));
    expect(api.reminders.get("n1")).toMatch(/^\d{4}-\d\d-\d\dT\d\d:\d\d:00Z$/);
  });

  it("refuses a time in the past without calling the backend", async () => {
    const { api } = await mount();
    await fireEvent.input(screen.getByLabelText("Fecha"), {
      target: { value: "2020-01-01" },
    });
    expect(screen.getByRole("alert").textContent).toContain("futuras");
    expect(
      (
        screen.getByRole("button", {
          name: "Guardar recordatorio",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(api.reminders.size).toBe(0);
  });

  it("shows the current reminder and removes it", async () => {
    const api = new FakeNotesApi();
    api.reminders.set("n1", "2099-01-01T08:00:00Z");
    const notes = new NotesStore(api);
    await notes.load();
    let closed = false;
    render(NoteReminderDialog, {
      notes,
      noteId: "n1",
      onClose: () => (closed = true),
    });
    await waitFor(() =>
      expect(screen.getByText(/Te avisaremos el/)).toBeTruthy(),
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Quitar recordatorio" }),
    );
    await waitFor(() => expect(closed).toBe(true));
    expect(api.reminders.has("n1")).toBe(false);
  });
});
