// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { FakeEventsApi } from "../../testing";
import EventDialog from "./EventDialog.svelte";

function mount(
  options: { eventId?: string; seed?: (api: FakeEventsApi) => void } = {},
) {
  const api = new FakeEventsApi();
  options.seed?.(api);
  const store = new EventsStore({ api, noticeMs: 60_000 });
  const onClose = vi.fn();
  const onConvert = vi.fn();
  render(EventDialog, {
    store,
    eventId: options.eventId,
    defaultDate: "2026-03-12",
    onClose,
    onConvert,
  });
  return { api, store, onClose, onConvert };
}

const field = (name: RegExp | string) =>
  screen.getByLabelText(name) as HTMLInputElement;

describe("EventDialog (F21-01)", () => {
  it("needs a title before it creates anything", async () => {
    const { api, onClose } = mount();
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    expect(
      await screen.findByText("Escribe un título para el evento."),
    ).toBeTruthy();
    expect(api.events).toHaveLength(0);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("creates a timed event on the day it was asked for", async () => {
    const { api, onClose } = mount();
    await fireEvent.input(field("Título"), { target: { value: "Dentista" } });
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.events[0]).toMatchObject({
      title: "Dentista",
      startDate: "2026-03-12",
      startTime: "09:00",
      endTime: "10:00",
    });
  });

  it("an all-day event carries no times and hides the time fields", async () => {
    const { api, onClose } = mount();
    await fireEvent.input(field("Título"), { target: { value: "Festivo" } });
    await fireEvent.click(screen.getByLabelText("Todo el día"));
    expect(screen.queryByLabelText(/Empieza · Hora/)).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.events[0]).toMatchObject({ startTime: null, allDay: true });
  });

  it("refuses an end before the start", async () => {
    const { api } = mount();
    await fireEvent.input(field("Título"), { target: { value: "Raro" } });
    await fireEvent.input(field(/Termina · Hora/), {
      target: { value: "08:00" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    expect(
      await screen.findByText("El evento no puede terminar antes de empezar."),
    ).toBeTruthy();
    expect(api.events).toHaveLength(0);
  });

  it("moving the start moves the end so the length stays", async () => {
    mount();
    await fireEvent.input(field(/Empieza · Hora/), {
      target: { value: "15:30" },
    });
    expect(field(/Termina · Hora/).value).toBe("16:30");
  });

  it("saves the chosen reminders", async () => {
    const { api, onClose } = mount();
    await fireEvent.input(field("Título"), { target: { value: "Cita" } });
    await fireEvent.click(screen.getByRole("button", { name: "15 min antes" }));
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.events[0].reminderOffsets).toEqual([15]);
  });

  it("edits an existing event as a series and can delete or convert it", async () => {
    const { api, store, onClose, onConvert } = mount({
      eventId: "x",
      seed: (fake) =>
        fake.makeEvent({
          id: "x",
          title: "Yoga",
          rrule: "FREQ=WEEKLY;INTERVAL=1",
        }),
    });
    const title = await waitFor(() => field("Título"));
    expect(title.value).toBe("Yoga");
    expect(
      screen.getByText("Los cambios se aplican a toda la serie."),
    ).toBeTruthy();
    await fireEvent.input(title, { target: { value: "Yoga suave" } });
    await fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(api.events[0].title).toBe("Yoga suave");

    await fireEvent.click(
      screen.getByRole("button", { name: "Convertir en tarea" }),
    );
    expect(onConvert).toHaveBeenCalledWith(
      expect.objectContaining({ id: "x" }),
    );

    await fireEvent.click(screen.getByRole("button", { name: "Eliminar" }));
    await waitFor(() => expect(api.events).toHaveLength(0));
    expect(store.deleted?.id).toBe("x");
  });

  it("closes by itself when the event no longer exists", async () => {
    const { onClose } = mount({ eventId: "gone" });
    await waitFor(() => expect(onClose).toHaveBeenCalled());
  });
});
