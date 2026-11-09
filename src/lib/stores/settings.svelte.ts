import * as backend from "../api";
import { t } from "../i18n";
import { applyTheme, parseTheme } from "../theme";
import type { EffectMode, SettingKey, ThemeMode } from "../types";
import { windowEffect } from "./windowEffect.svelte";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface SettingsApi {
  settingsGet(key: SettingKey): Promise<string | null>;
  settingsSet(key: SettingKey, value: string): Promise<void>;
  autostartGet(): Promise<boolean>;
  autostartSet(enabled: boolean): Promise<void>;
  backupCreate(path: string): Promise<void>;
  backupRestore(path: string): Promise<void>;
}

export interface EffectRequester {
  readonly applied: EffectMode;
  request(mode: EffectMode): Promise<void>;
}

const EFFECTS: readonly EffectMode[] = ["mica", "acrylic", "none"];

function parseEffect(value: string | null): EffectMode {
  return EFFECTS.find((effect) => effect === value) ?? "mica";
}

/** The backend's own message for rejected input; a generic one for everything else. */
export function describeError(raw: unknown): string {
  const { code, message } = backend.parseError(raw);
  return code === "VALIDATION" || code === "NOT_FOUND" || code === "IO"
    ? message
    : t.errors[code];
}

export interface SettingsNotice {
  kind: "info" | "error";
  text: string;
}

export interface SettingsStoreOptions {
  api?: SettingsApi;
  effects?: EffectRequester;
  apply?: (theme: ThemeMode) => void;
}

/** User preferences (F6-05) and the backup actions (F-16). */
export class SettingsStore {
  theme = $state<ThemeMode>("system");
  effect = $state<EffectMode>("mica");
  autostart = $state(false);
  busy = $state(false);
  notice = $state<SettingsNotice | null>(null);

  readonly #api: SettingsApi;
  readonly #effects: EffectRequester;
  readonly #apply: (theme: ThemeMode) => void;

  constructor(options: SettingsStoreOptions = {}) {
    this.#api = options.api ?? backend;
    this.#effects = options.effects ?? windowEffect;
    this.#apply = options.apply ?? ((theme) => applyTheme(theme));
  }

  /** The effect the OS really applied, which can differ from the one chosen. */
  get appliedEffect(): EffectMode {
    return this.#effects.applied;
  }

  /** Reads the saved theme and window effect and applies both. Runs at startup. */
  async load(): Promise<void> {
    const [theme, effect] = await Promise.all([
      this.#api.settingsGet("theme").catch(() => null),
      this.#api.settingsGet("window_effect").catch(() => null),
    ]);
    this.theme = parseTheme(theme);
    this.effect = parseEffect(effect);
    this.#apply(this.theme);
    await this.#effects.request(this.effect);
  }

  async loadAutostart(): Promise<void> {
    try {
      this.autostart = await this.#api.autostartGet();
    } catch {
      this.autostart = false;
    }
  }

  async setTheme(theme: ThemeMode): Promise<void> {
    this.theme = theme;
    this.#apply(theme);
    await this.#save("theme", theme);
  }

  async setEffect(effect: EffectMode): Promise<void> {
    this.effect = effect;
    await this.#effects.request(effect);
    await this.#save("window_effect", effect);
  }

  async setAutostart(enabled: boolean): Promise<void> {
    try {
      await this.#api.autostartSet(enabled);
      this.autostart = enabled;
      this.notice = null;
    } catch (error) {
      this.#fail(error);
    }
  }

  async createBackup(path: string): Promise<boolean> {
    return this.#run(
      () => this.#api.backupCreate(path),
      t.settings.backupCreated,
    );
  }

  /** On success the preferences are read again: they came back with the data. */
  async restoreBackup(path: string): Promise<boolean> {
    const done = await this.#run(
      () => this.#api.backupRestore(path),
      t.settings.restored,
    );
    if (done) await this.load();
    return done;
  }

  async #save(key: SettingKey, value: string): Promise<void> {
    try {
      await this.#api.settingsSet(key, value);
    } catch (error) {
      this.#fail(error);
    }
  }

  #fail(error: unknown): void {
    this.notice = {
      kind: "error",
      text: t.settings.failed(describeError(error)),
    };
  }

  async #run(action: () => Promise<void>, success: string): Promise<boolean> {
    this.busy = true;
    this.notice = null;
    try {
      await action();
      this.notice = { kind: "info", text: success };
      return true;
    } catch (error) {
      this.#fail(error);
      return false;
    } finally {
      this.busy = false;
    }
  }
}

export const settingsStore = new SettingsStore();
