import { describe, expect, it, vi } from "vitest";

import type { EffectMode, SettingKey, ThemeMode } from "../types";
import {
  describeError,
  SettingsStore,
  type SettingsApi,
} from "./settings.svelte";

function setup(saved: Partial<Record<SettingKey, string>> = {}) {
  const stored = new Map<string, string>(Object.entries(saved));
  const api = {
    settingsGet: vi.fn(async (key: SettingKey) => stored.get(key) ?? null),
    settingsSet: vi.fn(async (key: SettingKey, value: string) => {
      stored.set(key, value);
    }),
    autostartGet: vi.fn(async () => false),
    autostartSet: vi.fn(async () => undefined),
    backupCreate: vi.fn(async () => undefined),
    backupRestore: vi.fn(async () => undefined),
  } satisfies SettingsApi;
  const applied: ThemeMode[] = [];
  const effects = {
    applied: "mica" as EffectMode,
    request: vi.fn(async (mode: EffectMode) => {
      effects.applied = mode;
    }),
  };
  const store = new SettingsStore({
    api,
    effects,
    apply: (theme) => applied.push(theme),
  });
  return { store, api, effects, applied, stored };
}

describe("SettingsStore", () => {
  it("starts on the system theme and Mica when nothing is saved", async () => {
    const { store, applied, effects } = setup();
    await store.load();
    expect(store.theme).toBe("system");
    expect(store.effect).toBe("mica");
    expect(applied).toEqual(["system"]);
    expect(effects.request).toHaveBeenCalledWith("mica", "system");
  });

  it("applies the saved theme and effect at startup, ignoring invalid values", async () => {
    const saved = setup({ theme: "dark", window_effect: "acrylic" });
    await saved.store.load();
    expect(saved.applied).toEqual(["dark"]);
    expect(saved.effects.request).toHaveBeenCalledWith("acrylic", "dark");

    const invalid = setup({ theme: "neon", window_effect: "glass" });
    await invalid.store.load();
    expect(invalid.store.theme).toBe("system");
    expect(invalid.store.effect).toBe("mica");
  });

  it("keeps the window opaque in the light theme and restores the material in dark", async () => {
    const { store, effects } = setup({ theme: "light" });
    await store.load();
    expect(effects.request).toHaveBeenLastCalledWith("none", "light");
    expect(store.opaque).toBe(true);
    expect(store.appliedEffect).toBe("mica");

    await store.setEffect("acrylic");
    expect(effects.request).toHaveBeenLastCalledWith("none", "light");

    await store.setTheme("dark");
    expect(effects.request).toHaveBeenLastCalledWith("acrylic", "dark");
    expect(store.opaque).toBe(false);
  });

  it("still starts when the saved settings cannot be read", async () => {
    const { store, api, applied } = setup();
    api.settingsGet.mockRejectedValue("DB: fallo");
    await store.load();
    expect(applied).toEqual(["system"]);
  });

  it("reads and saves the default notification time, 09:00 when none is saved", async () => {
    const { store, api, stored } = setup();
    await store.load();
    expect(store.notifyTime).toBe("09:00");

    expect(await store.setNotifyTime("07:45")).toBe(true);
    expect(store.notifyTime).toBe("07:45");
    expect(stored.get("default_notify_time")).toBe("07:45");

    expect(await store.setNotifyTime("7:45")).toBe(false);
    expect(api.settingsSet).toHaveBeenCalledTimes(1);

    const reloaded = setup({ default_notify_time: "18:30" });
    await reloaded.store.load();
    expect(reloaded.store.notifyTime).toBe("18:30");
  });

  it("applies a theme change at once and saves it", async () => {
    const { store, applied, stored } = setup();
    await store.setTheme("light");
    expect(applied).toEqual(["light"]);
    expect(stored.get("theme")).toBe("light");
  });

  it("changes the effect and saves it; the applied one may differ", async () => {
    const { store, effects, stored } = setup();
    effects.request.mockImplementation(async () => {
      effects.applied = "none";
    });
    await store.setEffect("acrylic");
    expect(stored.get("window_effect")).toBe("acrylic");
    expect(store.effect).toBe("acrylic");
    expect(store.appliedEffect).toBe("none");
  });

  it("keeps the change but reports it when saving fails", async () => {
    const { store, api } = setup();
    api.settingsSet.mockRejectedValue("DB: fallo");
    await store.setTheme("dark");
    expect(store.theme).toBe("dark");
    expect(store.notice?.kind).toBe("error");
  });

  it("toggles start with Windows only when the OS accepts it", async () => {
    const { store, api } = setup();
    await store.setAutostart(true);
    expect(store.autostart).toBe(true);
    api.autostartSet.mockRejectedValue("IO: sin permiso");
    await store.setAutostart(false);
    expect(store.autostart).toBe(true);
    expect(store.notice?.text).toContain("sin permiso");
  });

  it("reports the autostart state as off when it cannot be read", async () => {
    const { store, api } = setup();
    api.autostartGet.mockRejectedValue("IO: fallo");
    await store.loadAutostart();
    expect(store.autostart).toBe(false);
  });

  it("creates a backup and reports success or the backend's reason", async () => {
    const { store, api } = setup();
    expect(await store.createBackup("C:/copia.sqlite")).toBe(true);
    expect(api.backupCreate).toHaveBeenCalledWith("C:/copia.sqlite");
    expect(store.notice).toEqual({
      kind: "info",
      text: "Copia de seguridad guardada.",
    });

    api.backupCreate.mockRejectedValue("IO: disco lleno");
    expect(await store.createBackup("C:/copia.sqlite")).toBe(false);
    expect(store.notice?.kind).toBe("error");
    expect(store.notice?.text).toContain("disco lleno");
    expect(store.busy).toBe(false);
  });

  it("reloads the preferences after a restore and keeps them on a rejected one", async () => {
    const { store, api, stored } = setup({ theme: "dark" });
    expect(await store.restoreBackup("C:/copia.sqlite")).toBe(true);
    expect(store.theme).toBe("dark");

    stored.set("theme", "light");
    api.backupRestore.mockRejectedValue(
      "VALIDATION: el archivo no es una copia válida: está dañado",
    );
    expect(await store.restoreBackup("C:/mala.sqlite")).toBe(false);
    expect(store.theme).toBe("dark");
    expect(store.notice?.text).toContain("está dañado");
  });

  it("shows technical failures as a generic message", () => {
    expect(describeError("DB: no such table: tasks")).not.toContain("tasks");
    expect(describeError("VALIDATION: valor no válido")).toBe(
      "valor no válido",
    );
  });
});
