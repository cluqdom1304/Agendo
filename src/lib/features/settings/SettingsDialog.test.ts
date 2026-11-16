// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import {
  SettingsStore,
  type EffectRequester,
  type SettingsApi,
} from "../../stores/settings.svelte";
import SettingsDialog from "./SettingsDialog.svelte";

function mount(
  options: {
    open?: string | null;
    save?: string | null;
    folder?: string | null;
    enex?: string[];
  } = {},
) {
  const api = {
    settingsGet: vi.fn(async () => null),
    settingsSet: vi.fn(async () => undefined),
    autostartGet: vi.fn(async () => false),
    autostartSet: vi.fn(async () => undefined),
    backupCreate: vi.fn(async () => undefined),
    backupRestore: vi.fn(async () => undefined),
    exportNotes: vi.fn(async () => ({ notes: 0, files: 0 })),
    exportJson: vi.fn(async () => ({ notes: 0, files: 0 })),
    importMarkdown: vi.fn(async () => ({
      notes: 0,
      folders: 0,
      tags: 0,
      skipped: 0,
    })),
    importEnex: vi.fn(async () => ({
      notes: 0,
      folders: 0,
      tags: 0,
      skipped: 0,
    })),
  } satisfies SettingsApi;
  const effects: EffectRequester = {
    applied: "mica",
    request: vi.fn(async () => undefined),
  };
  const store = new SettingsStore({ api, effects, apply: () => undefined });
  const onClose = vi.fn();
  const onRestored = vi.fn();
  const pickSavePath = vi.fn(async () => options.save ?? null);
  const pickOpenPath = vi.fn(async () => options.open ?? null);
  const pickFolder = vi.fn(async () => options.folder ?? null);
  const pickJsonPath = vi.fn(async () => options.save ?? null);
  const pickEnexPaths = vi.fn(async () => options.enex ?? []);
  render(SettingsDialog, {
    store,
    onClose,
    onRestored,
    pickSavePath,
    pickOpenPath,
    pickFolder,
    pickJsonPath,
    pickEnexPaths,
    now: () => new Date(2026, 2, 9, 12, 0),
  });
  return {
    api,
    store,
    onClose,
    onRestored,
    pickSavePath,
    pickOpenPath,
    pickFolder,
    pickJsonPath,
    pickEnexPaths,
  };
}

describe("SettingsDialog", () => {
  it("is a labelled modal dialog with focus inside", () => {
    mount();
    const dialog = screen.getByRole("dialog", { name: "Ajustes" });
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(dialog.contains(document.activeElement)).toBe(true);
  });

  it("closes with Escape and with the close button", async () => {
    const { onClose } = mount();
    await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    await fireEvent.click(
      screen.getByRole("button", { name: "Cerrar ajustes" }),
    );
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it("keeps Tab inside the dialog", async () => {
    mount();
    const dialog = screen.getByRole("dialog");
    const items = dialog.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled])",
    );
    const last = items[items.length - 1];
    last.focus();
    await fireEvent.keyDown(dialog, { key: "Tab" });
    expect(document.activeElement).toBe(items[0]);
    await fireEvent.keyDown(dialog, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it("changes the theme and the window effect", async () => {
    const { api } = mount();
    await fireEvent.click(screen.getByRole("radio", { name: "Oscuro" }));
    await fireEvent.click(screen.getByRole("radio", { name: "Acrílico" }));
    await waitFor(() => {
      expect(api.settingsSet).toHaveBeenCalledWith("theme", "dark");
      expect(api.settingsSet).toHaveBeenCalledWith("window_effect", "acrylic");
    });
  });

  it("shows start with Windows as off by default and turns it on", async () => {
    const { api } = mount();
    const toggle = screen.getByRole("switch", { name: /Iniciar con Windows/ });
    expect((toggle as HTMLInputElement).checked).toBe(false);
    await fireEvent.click(toggle);
    await waitFor(() => expect(api.autostartSet).toHaveBeenCalledWith(true));
  });

  it("does nothing when the save dialog is cancelled", async () => {
    const { api, pickSavePath } = mount({ save: null });
    await fireEvent.click(screen.getByRole("button", { name: "Crear copia…" }));
    await waitFor(() => expect(pickSavePath).toHaveBeenCalled());
    expect(api.backupCreate).not.toHaveBeenCalled();
  });

  it("passes the chosen path to backup_create, offering a dated name", async () => {
    const { api, pickSavePath } = mount({ save: "C:/copias/a.sqlite" });
    await fireEvent.click(screen.getByRole("button", { name: "Crear copia…" }));
    await waitFor(() =>
      expect(api.backupCreate).toHaveBeenCalledWith("C:/copias/a.sqlite"),
    );
    expect(pickSavePath).toHaveBeenCalledWith("Agendo-2026-03-09.agendo");
    expect((await screen.findByRole("status")).textContent).toContain(
      "Copia de seguridad guardada",
    );
  });

  it("exports all the notes to the chosen folder in either format", async () => {
    const { api } = mount({ folder: "C:/salida" });
    await fireEvent.click(
      screen.getByRole("button", { name: "Exportar notas a Markdown…" }),
    );
    await waitFor(() =>
      expect(api.exportNotes).toHaveBeenCalledWith(
        "markdown",
        "all",
        null,
        "C:/salida",
      ),
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Exportar notas a HTML…" }),
    );
    await waitFor(() =>
      expect(api.exportNotes).toHaveBeenLastCalledWith(
        "html",
        "all",
        null,
        "C:/salida",
      ),
    );
  });

  it("does nothing for the JSON export when the dialog is cancelled", async () => {
    const cancelled = mount({ save: null });
    await fireEvent.click(
      screen.getByRole("button", { name: "Exportar todo a JSON…" }),
    );
    await waitFor(() =>
      expect(cancelled.pickJsonPath).toHaveBeenCalledWith(
        "Agendo-2026-03-09.json",
      ),
    );
    expect(cancelled.api.exportJson).not.toHaveBeenCalled();
  });

  it("imports a folder and Evernote files and reloads the views", async () => {
    const { api, onRestored, pickFolder, pickEnexPaths } = mount({
      folder: "C:/notas",
      enex: ["C:/a.enex", "C:/b.enex"],
    });
    await fireEvent.click(
      screen.getByRole("button", {
        name: "Importar carpeta de Markdown o texto…",
      }),
    );
    await waitFor(() =>
      expect(api.importMarkdown).toHaveBeenCalledWith("C:/notas"),
    );
    await waitFor(() => expect(onRestored).toHaveBeenCalledTimes(1));
    expect(pickFolder).toHaveBeenCalled();

    await fireEvent.click(
      screen.getByRole("button", { name: "Importar desde Evernote…" }),
    );
    await waitFor(() =>
      expect(api.importEnex).toHaveBeenCalledWith(["C:/a.enex", "C:/b.enex"]),
    );
    await waitFor(() => expect(onRestored).toHaveBeenCalledTimes(2));
    expect(pickEnexPaths).toHaveBeenCalled();
  });

  it("does not reload the views when an import fails", async () => {
    const { api, onRestored } = mount({ enex: ["C:/mal.enex"] });
    api.importEnex.mockRejectedValue("VALIDATION: no es un .enex");
    await fireEvent.click(
      screen.getByRole("button", { name: "Importar desde Evernote…" }),
    );
    expect((await screen.findByRole("alert")).textContent).toContain(
      "no es un .enex",
    );
    expect(onRestored).not.toHaveBeenCalled();
  });

  it("asks for confirmation before choosing the file to restore", async () => {
    const { api, pickOpenPath, onRestored } = mount({
      open: "C:/copias/a.sqlite",
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Restaurar copia…" }),
    );
    expect(pickOpenPath).not.toHaveBeenCalled();
    expect(screen.getByRole("group").textContent).toContain(
      "sustituye todos los datos actuales",
    );

    await fireEvent.click(
      screen.getByRole("button", { name: "Elegir archivo…" }),
    );
    await waitFor(() =>
      expect(api.backupRestore).toHaveBeenCalledWith("C:/copias/a.sqlite"),
    );
    await waitFor(() => expect(onRestored).toHaveBeenCalled());
  });

  it("shows why a restore was rejected and does not reload the views", async () => {
    const { api, onRestored } = mount({ open: "C:/mala.sqlite" });
    api.backupRestore.mockRejectedValue(
      "VALIDATION: el archivo no es una copia válida: está dañado",
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Restaurar copia…" }),
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Elegir archivo…" }),
    );
    expect((await screen.findByRole("alert")).textContent).toContain(
      "está dañado",
    );
    expect(onRestored).not.toHaveBeenCalled();
  });
});
