// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import TopBar from "./TopBar.svelte";

function mount(overrides: Partial<Record<string, unknown>> = {}) {
  const props = {
    sidebarOpen: true,
    filtered: false,
    canGoBack: false,
    canGoForward: true,
    newLabel: "Nueva tarea",
    onToggleSidebar: vi.fn(),
    onNew: vi.fn(),
    onBack: vi.fn(),
    onForward: vi.fn(),
    onSearch: vi.fn(),
    onSettings: vi.fn(),
    ...overrides,
  };
  render(TopBar, props);
  return props;
}

describe("TopBar (F19-03)", () => {
  it("folds the sidebar, creates and opens the palette from the search box", async () => {
    const props = mount();
    const fold = screen.getByRole("button", {
      name: "Mostrar u ocultar la barra lateral",
    });
    expect(fold.getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(fold);
    expect(props.onToggleSidebar).toHaveBeenCalledOnce();

    await fireEvent.click(screen.getByRole("button", { name: "Nueva tarea" }));
    expect(props.onNew).toHaveBeenCalledOnce();

    await fireEvent.click(
      screen.getByRole("button", { name: /Buscar o crear/ }),
    );
    expect(props.onSearch).toHaveBeenCalledOnce();
  });

  it("enables back and forward only when there is somewhere to go", async () => {
    const props = mount();
    const back = screen.getByRole("button", { name: "Atrás" });
    const forward = screen.getByRole("button", { name: "Adelante" });
    expect((back as HTMLButtonElement).disabled).toBe(true);
    expect((forward as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(forward);
    expect(props.onForward).toHaveBeenCalledOnce();
  });

  it("names the active tag filter on the fold button", () => {
    mount({ filtered: true });
    expect(
      screen.getByRole("button", {
        name: /Mostrar u ocultar la barra lateral\./,
      }),
    ).toBeTruthy();
  });

  it("offers the inbox toggle only where the panel can open (F20-04)", async () => {
    const onToggleInbox = vi.fn();
    mount({ inboxAvailable: true, inboxOpen: true, onToggleInbox });
    const toggle = screen.getByRole("button", {
      name: "Mostrar u ocultar la bandeja de entrada",
    });
    expect(toggle.getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(toggle);
    expect(onToggleInbox).toHaveBeenCalledOnce();
  });

  it("hides the inbox toggle elsewhere", () => {
    mount();
    expect(
      screen.queryByRole("button", {
        name: "Mostrar u ocultar la bandeja de entrada",
      }),
    ).toBeNull();
  });

  it("shows the focus countdown and stops the session from it (F25-01)", async () => {
    const onStopFocus = vi.fn();
    mount({ focusLabel: "12:34", focusTitle: "Informe", onStopFocus });
    expect(screen.getByText("12:34")).toBeTruthy();
    await fireEvent.click(
      screen.getByRole("button", {
        name: "Terminar la sesión de enfoque: Informe",
      }),
    );
    expect(onStopFocus).toHaveBeenCalledOnce();
  });
});
