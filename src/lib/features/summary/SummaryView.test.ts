// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { SummaryStore } from "../../stores/summary.svelte";
import { TagsStore } from "../../stores/tags.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeSummaryApi } from "../../testing";
import SummaryView from "./SummaryView.svelte";

const NOW = new Date("2026-03-10T10:00:00");

function mount(
  seed: (api: FakeSummaryApi) => void = () => undefined,
  pickSavePath: (name: string) => Promise<string | null> = async () => null,
) {
  const api = new FakeSummaryApi();
  seed(api);
  const view = new ViewStore({
    api: { settingsGet: async () => null, settingsSet: async () => undefined },
    now: () => NOW,
    firstDay: () => 1,
  });
  const store = new SummaryStore({
    api,
    today: () => view.today(),
    firstDay: () => view.firstDay,
  });
  const tags = new TagsStore({
    tagList: async () => [],
    tagUpsert: async () => {
      throw new Error("unused");
    },
    tagDelete: async () => undefined,
    tagMerge: async () => undefined,
  });
  render(SummaryView, { view, store, tags, pickSavePath });
  return { api, store };
}

describe("SummaryView (F26-02, F26-03)", () => {
  it("shows a calm message when the week is empty", async () => {
    mount();
    expect(await screen.findByText("Una semana tranquila")).toBeTruthy();
  });

  it("shows the figures, the habits and the focus time", async () => {
    mount((api) => {
      api.summary = {
        ...api.summary,
        completed: 5,
        pending: 3,
        overdue: 2,
        byTag: [{ tagId: null, completed: 5, pending: 3, overdue: 2 }],
        events: 2,
        eventMinutes: 90,
        habits: [{ habitId: "h1", title: "Leer", scheduled: 4, done: 3 }],
        focusSeconds: 5100,
        focusSessions: 2,
        focusByDay: [0, 5100, 0, 0, 0, 0, 0],
      };
    });
    await screen.findByText("2 eventos");
    const figure = (name: string) =>
      document.querySelector(`[data-figure="${name}"]`)?.textContent;
    expect([figure("completed"), figure("pending"), figure("overdue")]).toEqual(
      ["5", "3", "2"],
    );
    expect(screen.getByText("Sin etiqueta")).toBeTruthy();
    expect(screen.getByText("1 h 30 min en eventos con hora")).toBeTruthy();
    expect(
      screen.getByRole("meter", { name: "Leer" }).getAttribute("aria-valuenow"),
    ).toBe("75");
    expect(screen.getByText("1 h 25 min en 2 sesiones")).toBeTruthy();
  });

  it("pages between weeks and returns to the current one", async () => {
    const { api } = mount();
    await screen.findByText("Una semana tranquila");
    const thisWeek = screen.getByRole("button", {
      name: "Esta semana",
    }) as HTMLButtonElement;
    expect(thisWeek.disabled).toBe(true);
    await fireEvent.click(
      screen.getByRole("button", { name: "Semana siguiente" }),
    );
    await waitFor(() => expect(api.calls.at(-1)?.date).toBe("2026-03-17"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Semana anterior" }),
    );
    await waitFor(() => expect(api.calls.at(-1)?.date).toBe("2026-03-10"));
  });

  it("exports the week to the chosen file and confirms it", async () => {
    const pick = vi.fn(async () => "C:/Resumen.md");
    const { api } = mount(() => undefined, pick);
    await screen.findByText("Una semana tranquila");
    await fireEvent.click(
      screen.getByRole("button", { name: "Exportar a Markdown…" }),
    );
    await waitFor(() => expect(api.exported).toHaveLength(1));
    expect(pick).toHaveBeenCalledWith("Resumen-2026-03-09.md");
    expect(await screen.findByText("Resumen exportado.")).toBeTruthy();
  });

  it("does nothing when the file dialog is cancelled", async () => {
    const pick = vi.fn(async () => null);
    const { api } = mount(() => undefined, pick);
    await screen.findByText("Una semana tranquila");
    await fireEvent.click(
      screen.getByRole("button", { name: "Exportar a Markdown…" }),
    );
    await waitFor(() => expect(pick).toHaveBeenCalled());
    expect(api.exported).toHaveLength(0);
  });

  it("offers a retry when the summary cannot be prepared", async () => {
    const { api } = mount((fake) => (fake.failWith = "DB: sin acceso"));
    expect(
      await screen.findByText("No se pudo preparar el resumen."),
    ).toBeTruthy();
    api.failWith = null;
    await fireEvent.click(screen.getByRole("button", { name: "Reintentar" }));
    expect(await screen.findByText("Una semana tranquila")).toBeTruthy();
  });
});
