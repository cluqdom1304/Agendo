// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import type { SearchHit } from "../types";
import Spotlight from "./Spotlight.svelte";

const hit = (
  taskId: string,
  title: string,
  extra: Partial<SearchHit> = {},
) => ({
  taskId,
  title,
  snippet: "",
  dueDate: null,
  isCompleted: false,
  ...extra,
});

function mount(search: (query: string) => Promise<SearchHit[]>) {
  const onSelect = vi.fn();
  const onClose = vi.fn();
  render(Spotlight, { search, onSelect, onClose });
  const input = screen.getByRole("combobox");
  return { input, onSelect, onClose };
}

const type = (input: HTMLElement, value: string) =>
  fireEvent.input(input, { target: { value } });

describe("Spotlight", () => {
  it("focuses the field and asks to type before searching", () => {
    const search = vi.fn(async () => []);
    const { input } = mount(search);
    expect(document.activeElement).toBe(input);
    expect(screen.getByRole("status").textContent).toBe("Escribe para buscar");
    expect(search).not.toHaveBeenCalled();
  });

  it("lists results with date and snippet, and reports an empty search", async () => {
    const search = vi.fn(async (query: string) =>
      query === "alg"
        ? [
            hit("a", "Estudiar álgebra", {
              dueDate: "2026-03-12",
              snippet: "tema 3",
            }),
          ]
        : [],
    );
    const { input } = mount(search);
    await type(input, "alg");
    const option = await screen.findByRole("option");
    expect(option.textContent).toContain("Estudiar álgebra");
    expect(option.textContent).toContain("12/03/2026");
    expect(option.textContent).toContain("tema 3");
    expect(screen.getByRole("status").textContent).toBe("1 resultado");

    await type(input, "zzz");
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toBe("Sin resultados"),
    );
  });

  it("navigates with the arrow keys, wrapping, and chooses with Enter", async () => {
    const { input, onSelect } = mount(async () => [
      hit("a", "Uno"),
      hit("b", "Dos"),
      hit("c", "Tres"),
    ]);
    await type(input, "x");
    await screen.findAllByRole("option");
    const selected = () =>
      screen
        .getAllByRole("option")
        .findIndex((o) => o.getAttribute("aria-selected") === "true");
    expect(selected()).toBe(0);

    await fireEvent.keyDown(input, { key: "ArrowDown" });
    await fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(selected()).toBe(2);
    await fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(selected()).toBe(0);
    await fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(selected()).toBe(2);
    expect(input.getAttribute("aria-activedescendant")).toBe("hit-2");

    await fireEvent.keyDown(input, { key: "Enter" });
    expect(onSelect).toHaveBeenCalledWith(
      expect.objectContaining({ taskId: "c" }),
    );
  });

  it("closes with Escape and with Ctrl+K, and keeps keys from the document", async () => {
    const { input, onClose } = mount(async () => []);
    const outer = vi.fn();
    window.addEventListener("keydown", outer);
    await fireEvent.keyDown(input, { key: "Escape" });
    await fireEvent.keyDown(input, { key: "k", ctrlKey: true });
    await fireEvent.keyDown(input, { key: "n", ctrlKey: true });
    window.removeEventListener("keydown", outer);
    expect(onClose).toHaveBeenCalledTimes(2);
    expect(outer).not.toHaveBeenCalled();
  });

  it("ignores a slow answer that arrives after a newer one", async () => {
    let releaseFirst!: (hits: SearchHit[]) => void;
    const search = vi.fn((query: string) =>
      query === "a"
        ? new Promise<SearchHit[]>((resolve) => (releaseFirst = resolve))
        : Promise.resolve([hit("n", "Nuevo")]),
    );
    const { input } = mount(search);
    await type(input, "a");
    await type(input, "ab");
    await screen.findByText("Nuevo");
    releaseFirst([hit("o", "Viejo")]);
    await Promise.resolve();
    expect(screen.queryByText("Viejo")).toBeNull();
    expect(screen.getByText("Nuevo")).toBeTruthy();
  });

  it("shows a message when the search fails", async () => {
    const { input } = mount(async () => {
      throw "DB: fallo";
    });
    await type(input, "x");
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toBe(
        "La búsqueda ha fallado.",
      ),
    );
  });
});
