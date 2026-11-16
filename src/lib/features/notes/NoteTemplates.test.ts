// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { NotesStore } from "../../stores/notes.svelte";
import NoteTemplates from "./NoteTemplates.svelte";
import { FakeNotesApi } from "./testing";

async function mount() {
  const api = new FakeNotesApi();
  const notes = new NotesStore(api);
  await notes.load();
  let closed = false;
  render(NoteTemplates, { notes, onClose: () => (closed = true) });
  return { api, notes, closed: () => closed };
}

describe("NoteTemplates", () => {
  it("explains that a note must be open before it can be saved", async () => {
    await mount();
    expect(
      screen.getByText("Abre una nota para poder guardarla como plantilla."),
    ).toBeTruthy();
    expect(
      (
        screen.getByRole("button", {
          name: "Guardar como plantilla",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });

  it("saves the open note under a name and then lists it", async () => {
    const { api, notes } = await mount();
    await notes.createNoteWith({ title: "Acta" });
    await fireEvent.input(screen.getByLabelText("Nombre de la plantilla"), {
      target: { value: "Reunión" },
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Guardar como plantilla" }),
    );
    await waitFor(() => expect(screen.getByText("Reunión")).toBeTruthy());
    expect(api.templates.map((item) => item.name)).toEqual(["Reunión"]);
  });

  it("deletes a template", async () => {
    const { api, notes } = await mount();
    await api.templateSave({ name: "Diario", contentJson: "{}" });
    await notes.loadTemplates();
    await waitFor(() => expect(screen.getByText("Diario")).toBeTruthy());
    await fireEvent.click(screen.getByRole("button", { name: "Eliminar" }));
    await waitFor(() => expect(screen.queryByText("Diario")).toBeNull());
  });
});
