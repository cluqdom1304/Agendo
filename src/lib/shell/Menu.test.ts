// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import Menu from "./Menu.svelte";

const items = [
  { id: "updated", label: "Modificación", checked: true },
  { id: "created", label: "Creación", checked: false },
  { id: "title", label: "Título", checked: false },
];

function mount(overrides: Partial<Parameters<typeof render>[1]> = {}) {
  const onSelect = vi.fn<(id: string) => void>();
  const onClose = vi.fn<() => void>();
  render(Menu, {
    label: "Ordenar por",
    items,
    onSelect,
    onClose,
    ...overrides,
  });
  return { onSelect, onClose };
}

describe("Menu", () => {
  it("marks the current choice and starts the keyboard on it", () => {
    mount();
    const current = screen.getByRole("menuitemradio", { name: "Modificación" });
    expect(current.getAttribute("aria-checked")).toBe("true");
    expect(document.activeElement).toBe(current);
  });

  it("chooses an item and closes", async () => {
    const { onSelect, onClose } = mount();
    await fireEvent.click(
      screen.getByRole("menuitemradio", { name: "Título" }),
    );
    expect(onSelect).toHaveBeenCalledWith("title");
    expect(onClose).toHaveBeenCalled();
  });

  it("moves with the arrows, wrapping at the ends", async () => {
    mount();
    await fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(document.activeElement?.textContent?.trim()).toBe("Creación");
    await fireEvent.keyDown(window, { key: "ArrowUp" });
    await fireEvent.keyDown(window, { key: "ArrowUp" });
    expect(document.activeElement?.textContent?.trim()).toBe("Título");
    await fireEvent.keyDown(window, { key: "Home" });
    expect(document.activeElement?.textContent?.trim()).toBe("Modificación");
  });

  it("closes with Escape and with a press outside, but not on its opener", async () => {
    const { onClose, onSelect } = mount();
    const opener = document.createElement("button");
    opener.setAttribute("data-menu-toggle", "");
    document.body.append(opener);

    await fireEvent.pointerDown(opener);
    expect(onClose).not.toHaveBeenCalled();
    await fireEvent.pointerDown(screen.getByRole("menu"));
    expect(onClose).not.toHaveBeenCalled();

    await fireEvent.pointerDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(1);
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(2);
    expect(onSelect).not.toHaveBeenCalled();
    opener.remove();
  });

  it("draws plain actions as menu items", () => {
    mount({
      items: [
        { id: "duplicate", label: "Duplicar" },
        { id: "delete", label: "Eliminar", danger: true, separated: true },
      ],
    });
    expect(screen.getAllByRole("menuitem")).toHaveLength(2);
    expect(screen.queryByRole("menuitemradio")).toBeNull();
  });
});
