import * as backend from "../api";
import { t } from "../i18n";
import { applyTheme, parseTheme, resolveTheme } from "../theme";
import type {
  EffectMode,
  ExportFormat,
  ExportKind,
  ExportReport,
  ImportReport,
  SettingKey,
  ThemeMode,
} from "../types";
import { windowEffect } from "./windowEffect.svelte";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface SettingsApi {
  settingsGet(key: SettingKey): Promise<string | null>;
  settingsSet(key: SettingKey, value: string): Promise<void>;
  autostartGet(): Promise<boolean>;
  autostartSet(enabled: boolean): Promise<void>;
  backupCreate(path: string): Promise<void>;
  backupRestore(path: string): Promise<void>;
  exportNotes(
    format: ExportFormat,
    kind: ExportKind,
    id: string | null,
    path: string,
  ): Promise<ExportReport>;
  exportJson(path: string): Promise<ExportReport>;
  importMarkdown(path: string): Promise<ImportReport>;
  importEnex(paths: string[]): Promise<ImportReport>;
}

export interface EffectRequester {
  readonly applied: EffectMode;
  request(mode: EffectMode, theme?: ThemeMode): Promise<void>;
}

export const DEFAULT_NOTIFY_TIME = "09:00";

/** How often the app makes a copy by itself (F17-09). */
export type AutoBackupMode = "off" | "daily" | "weekly";
export const AUTO_BACKUP_MODES: readonly AutoBackupMode[] = [
  "off",
  "daily",
  "weekly",
];
export const AUTO_BACKUP_KEEP_CHOICES = [3, 5, 7, 10, 14, 30] as const;
export const DEFAULT_AUTO_BACKUP_KEEP = 7;

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

function importedText(report: ImportReport): string {
  return t.settings.imported(
    report.notes,
    report.folders,
    report.tags,
    report.skipped,
    report.duplicates,
  );
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
  /** `HH:MM` at which reminders of tasks without a time sound (D-35). */
  notifyTime = $state(DEFAULT_NOTIFY_TIME);
  autoBackup = $state<AutoBackupMode>("off");
  /** How many automatic copies are kept. */
  autoBackupKeep = $state(DEFAULT_AUTO_BACKUP_KEEP);
  /** UTC instant of the last automatic copy, or `null` if none was made yet. */
  autoBackupLast = $state<string | null>(null);
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

  /**
   * In the light theme the window is opaque: a translucent material shows the desktop through
   * and washes the light palette out (D-36).
   */
  get opaque(): boolean {
    return resolveTheme(this.theme) === "light";
  }

  /** The effect the OS really applied, which can differ from the one chosen. */
  get appliedEffect(): EffectMode {
    return this.opaque ? this.effect : this.#effects.applied;
  }

  #requestEffect(effect: EffectMode): Promise<void> {
    return this.#effects.request(this.opaque ? "none" : effect, this.theme);
  }

  /** `system` can flip while the app runs; the material must follow. */
  watchSystemTheme(): () => void {
    if (typeof window === "undefined" || !window.matchMedia) return () => {};
    const query = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = () => {
      if (this.theme === "system") void this.#requestEffect(this.effect);
    };
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }

  /** Reads the saved theme and window effect and applies both. Runs at startup. */
  async load(): Promise<void> {
    const [theme, effect, notifyTime, mode, keep, last] = await Promise.all([
      this.#api.settingsGet("theme").catch(() => null),
      this.#api.settingsGet("window_effect").catch(() => null),
      this.#api.settingsGet("default_notify_time").catch(() => null),
      this.#api.settingsGet("auto_backup").catch(() => null),
      this.#api.settingsGet("auto_backup_keep").catch(() => null),
      this.#api.settingsGet("auto_backup_last").catch(() => null),
    ]);
    this.autoBackup =
      AUTO_BACKUP_MODES.find((known) => known === mode) ?? "off";
    const kept = Number(keep);
    this.autoBackupKeep = AUTO_BACKUP_KEEP_CHOICES.some((c) => c === kept)
      ? kept
      : DEFAULT_AUTO_BACKUP_KEEP;
    this.autoBackupLast = last || null;
    this.theme = parseTheme(theme);
    this.notifyTime =
      notifyTime && /^\d{2}:\d{2}$/.test(notifyTime)
        ? notifyTime
        : DEFAULT_NOTIFY_TIME;
    this.effect = parseEffect(effect);
    this.#apply(this.theme);
    await this.#requestEffect(this.effect);
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
    await this.#requestEffect(this.effect);
    await this.#save("theme", theme);
  }

  async setEffect(effect: EffectMode): Promise<void> {
    this.effect = effect;
    await this.#requestEffect(effect);
    await this.#save("window_effect", effect);
  }

  /** The reminders already saved move too, so tasks must be read again by the caller. */
  async setNotifyTime(time: string): Promise<boolean> {
    if (!/^\d{2}:\d{2}$/.test(time)) return false;
    const previous = this.notifyTime;
    this.notifyTime = time;
    try {
      await this.#api.settingsSet("default_notify_time", time);
      this.notice = null;
      return true;
    } catch (error) {
      this.notifyTime = previous;
      this.#fail(error);
      return false;
    }
  }

  async setAutoBackup(mode: AutoBackupMode): Promise<boolean> {
    const previous = this.autoBackup;
    this.autoBackup = mode;
    try {
      await this.#api.settingsSet("auto_backup", mode);
      this.notice = null;
      return true;
    } catch (error) {
      this.autoBackup = previous;
      this.#fail(error);
      return false;
    }
  }

  async setAutoBackupKeep(count: number): Promise<boolean> {
    const previous = this.autoBackupKeep;
    this.autoBackupKeep = count;
    try {
      await this.#api.settingsSet("auto_backup_keep", String(count));
      this.notice = null;
      return true;
    } catch (error) {
      this.autoBackupKeep = previous;
      this.#fail(error);
      return false;
    }
  }

  /** The schedule runs in the backend: this reads when it last made a copy. */
  async refreshAutoBackupLast(): Promise<void> {
    try {
      this.autoBackupLast =
        (await this.#api.settingsGet("auto_backup_last")) || null;
    } catch {
      this.autoBackupLast = null;
    }
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

  /** Writes every note into the folder `path` (F14-01). */
  async exportNotes(format: ExportFormat, path: string): Promise<boolean> {
    return this.#run(
      () => this.#api.exportNotes(format, "all", null, path),
      (report) => t.settings.exportedNotes(report.notes, report.files),
    );
  }

  /** Writes all the data as one JSON file (F14-02). */
  async exportJson(path: string): Promise<boolean> {
    return this.#run(
      () => this.#api.exportJson(path),
      (report) => t.settings.exportedJson(report.notes),
    );
  }

  /** Imports a folder of Markdown and text files (F14-03). */
  async importMarkdown(path: string): Promise<boolean> {
    return this.#run(() => this.#api.importMarkdown(path), importedText);
  }

  /** Imports Evernote exports, one folder per file (F14-04). */
  async importEnex(paths: string[]): Promise<boolean> {
    return this.#run(() => this.#api.importEnex(paths), importedText);
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

  async #run<T>(
    action: () => Promise<T>,
    success: string | ((result: T) => string),
  ): Promise<boolean> {
    this.busy = true;
    this.notice = null;
    try {
      const result = await action();
      this.notice = {
        kind: "info",
        text: typeof success === "string" ? success : success(result),
      };
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
