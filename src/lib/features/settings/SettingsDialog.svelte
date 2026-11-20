<script lang="ts">
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { appInfo } from "../../api";

  import { toLocalIsoDate } from "../../dates";
  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import SegmentedControl from "../../shell/SegmentedControl.svelte";
  import {
    AUTO_BACKUP_KEEP_CHOICES,
    AUTO_BACKUP_MODES,
    type AutoBackupMode,
    type SettingsStore,
  } from "../../stores/settings.svelte";
  import SettingsAbout from "./SettingsAbout.svelte";
  import type {
    AppInfo,
    EffectMode,
    ExportFormat,
    ThemeMode,
  } from "../../types";

  interface Props {
    store: SettingsStore;
    onClose: () => void;
    /** The data was replaced: views must load it again. */
    onRestored: () => void;
    pickSavePath?: (defaultName: string) => Promise<string | null>;
    pickOpenPath?: () => Promise<string | null>;
    /** Folder to write exported notes into, or to read Markdown files from. */
    pickFolder?: () => Promise<string | null>;
    pickJsonPath?: (defaultName: string) => Promise<string | null>;
    pickEnexPaths?: () => Promise<string[]>;
    now?: () => Date;
    loadInfo?: () => Promise<AppInfo>;
  }

  const filters = [
    { name: t.settings.backupFilter, extensions: ["agendo"] },
    { name: t.settings.backupFilterLegacy, extensions: ["sqlite"] },
  ];

  let {
    store,
    onClose,
    onRestored,
    pickSavePath = (defaultName) => save({ defaultPath: defaultName, filters }),
    pickOpenPath = async () => {
      const chosen = await open({ multiple: false, directory: false, filters });
      return typeof chosen === "string" ? chosen : null;
    },
    pickFolder = async () => {
      const chosen = await open({ directory: true, multiple: false });
      return typeof chosen === "string" ? chosen : null;
    },
    pickJsonPath = (defaultName) =>
      save({
        defaultPath: defaultName,
        filters: [{ name: t.settings.jsonFilter, extensions: ["json"] }],
      }),
    pickEnexPaths = async () => {
      const chosen = await open({
        multiple: true,
        directory: false,
        filters: [{ name: t.settings.enexFilter, extensions: ["enex"] }],
      });
      return Array.isArray(chosen) ? chosen : [];
    },
    now = () => new Date(),
    loadInfo = appInfo,
  }: Props = $props();

  const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
    { value: "system", label: t.settings.themeSystem },
    { value: "light", label: t.settings.themeLight },
    { value: "dark", label: t.settings.themeDark },
  ];

  const EFFECT_OPTIONS: { value: EffectMode; label: string }[] = [
    { value: "mica", label: t.settings.effectMica },
    { value: "acrylic", label: t.settings.effectAcrylic },
    { value: "none", label: t.settings.effectNone },
  ];

  const EFFECT_NAMES: Record<EffectMode, string> = {
    mica: t.settings.effectMica,
    acrylic: t.settings.effectAcrylic,
    none: t.settings.effectNone,
  };

  let confirmingRestore = $state(false);

  const fallbackNote = $derived(
    store.opaque
      ? t.settings.effectOpaqueInLight
      : store.appliedEffect === store.effect
        ? null
        : t.settings.effectFallback(EFFECT_NAMES[store.appliedEffect]),
  );

  onMount(() => {
    void store.loadAutostart();
    void store.refreshAutoBackupLast();
    store.notice = null;
  });

  const autoBackupText = $derived(
    store.autoBackupLast
      ? t.settings.autoBackupLast(
          new Date(store.autoBackupLast).toLocaleString("es-ES", {
            dateStyle: "short",
            timeStyle: "short",
          }),
        )
      : t.settings.autoBackupNever,
  );

  async function createBackup() {
    const path = await pickSavePath(
      t.settings.backupFileName(toLocalIsoDate(now())),
    );
    if (path) await store.createBackup(path);
  }

  async function exportNotes(format: ExportFormat) {
    const path = await pickFolder();
    if (path) await store.exportNotes(format, path);
  }

  async function exportJson() {
    const path = await pickJsonPath(
      t.settings.jsonFileName(toLocalIsoDate(now())),
    );
    if (path) await store.exportJson(path);
  }

  async function importMarkdown() {
    const path = await pickFolder();
    if (path && (await store.importMarkdown(path))) onRestored();
  }

  async function importEnex() {
    const paths = await pickEnexPaths();
    if (paths.length > 0 && (await store.importEnex(paths))) onRestored();
  }

  async function restoreBackup() {
    const path = await pickOpenPath();
    if (!path) return;
    confirmingRestore = false;
    if (await store.restoreBackup(path)) onRestored();
  }
</script>

<Modal label={t.settings.title} closeLabel={t.settings.close} {onClose}>
  <h2>{t.settings.title}</h2>

  <section aria-labelledby="settings-appearance">
    <h3 id="settings-appearance">{t.settings.appearance}</h3>

    <div class="row">
      <span>{t.settings.theme}</span>
      <SegmentedControl
        label={t.settings.theme}
        options={THEME_OPTIONS}
        value={store.theme}
        onChange={(theme) => void store.setTheme(theme)}
      />
    </div>

    <div class="row">
      <span>{t.settings.windowEffect}</span>
      <SegmentedControl
        label={t.settings.windowEffect}
        options={EFFECT_OPTIONS}
        value={store.effect}
        onChange={(effect) => void store.setEffect(effect)}
      />
    </div>
    {#if fallbackNote}
      <p class="hint" role="status">{fallbackNote}</p>
    {/if}
  </section>

  <section aria-labelledby="settings-reminders">
    <h3 id="settings-reminders">{t.settings.reminders}</h3>
    <label class="row">
      <span>
        {t.settings.notifyTime}
        <small class="hint">{t.settings.notifyTimeHint}</small>
      </span>
      <input
        type="time"
        required
        value={store.notifyTime}
        onchange={(event) => {
          if (event.currentTarget.value)
            void store.setNotifyTime(event.currentTarget.value);
        }}
      />
    </label>
  </section>

  <section aria-labelledby="settings-system">
    <h3 id="settings-system">{t.settings.system}</h3>
    <label class="row">
      <span>
        {t.settings.autostart}
        <small class="hint">{t.settings.autostartHint}</small>
      </span>
      <input
        type="checkbox"
        role="switch"
        checked={store.autostart}
        onchange={(event) =>
          void store.setAutostart(event.currentTarget.checked)}
      />
    </label>
  </section>

  <section aria-labelledby="settings-backups">
    <h3 id="settings-backups">{t.settings.backups}</h3>
    <p class="hint">{t.settings.backupHint}</p>

    <label class="row">
      <span>
        {t.settings.autoBackup}
        <small class="hint">{t.settings.autoBackupHint}</small>
      </span>
      <select
        value={store.autoBackup}
        onchange={(event) =>
          void store.setAutoBackup(event.currentTarget.value as AutoBackupMode)}
      >
        {#each AUTO_BACKUP_MODES as mode (mode)}
          <option value={mode}>{t.settings.autoBackupModes[mode]}</option>
        {/each}
      </select>
    </label>
    {#if store.autoBackup !== "off"}
      <label class="row">
        <span>{t.settings.autoBackupKeep}</span>
        <select
          value={String(store.autoBackupKeep)}
          onchange={(event) =>
            void store.setAutoBackupKeep(Number(event.currentTarget.value))}
        >
          {#each AUTO_BACKUP_KEEP_CHOICES as count (count)}
            <option value={String(count)}>{count}</option>
          {/each}
        </select>
      </label>
      <p class="hint" role="status">
        {autoBackupText}
      </p>
    {/if}

    <div class="buttons">
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        onclick={createBackup}
      >
        {t.settings.backupCreate}
      </button>
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        aria-expanded={confirmingRestore}
        onclick={() => (confirmingRestore = !confirmingRestore)}
      >
        {t.settings.backupRestore}
      </button>
    </div>

    {#if confirmingRestore}
      <div class="confirm" role="group" aria-label={t.settings.backupRestore}>
        <p>{t.settings.restoreWarning}</p>
        <div class="buttons">
          <button
            type="button"
            class="btn danger"
            disabled={store.busy}
            onclick={restoreBackup}
          >
            {t.settings.restoreChoose}
          </button>
          <button
            type="button"
            class="btn"
            onclick={() => (confirmingRestore = false)}
          >
            {t.settings.restoreCancel}
          </button>
        </div>
      </div>
    {/if}
  </section>

  <section aria-labelledby="settings-exchange">
    <h3 id="settings-exchange">{t.settings.exchange}</h3>
    <p class="hint">{t.settings.exchangeHint}</p>

    <div class="buttons">
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        onclick={() => exportNotes("markdown")}
      >
        {t.settings.exportMarkdown}
      </button>
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        onclick={() => exportNotes("html")}
      >
        {t.settings.exportHtml}
      </button>
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        onclick={exportJson}
      >
        {t.settings.exportJson}
      </button>
    </div>
    <div class="buttons">
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        onclick={importMarkdown}
      >
        {t.settings.importMarkdown}
      </button>
      <button
        type="button"
        class="btn"
        disabled={store.busy}
        onclick={importEnex}
      >
        {t.settings.importEnex}
      </button>
    </div>

    {#if store.busy}
      <p class="hint" role="status">{t.settings.working}</p>
    {:else if store.notice}
      <p
        class="notice"
        class:error={store.notice.kind === "error"}
        role={store.notice.kind === "error" ? "alert" : "status"}
      >
        {store.notice.text}
      </p>
    {/if}
  </section>

  <SettingsAbout {loadInfo} />
</Modal>

<style>
  h2 {
    margin: 0 0 var(--space-2);
    font-family: var(--font-display);
    font-size: var(--text-title);
    font-weight: 700;
    letter-spacing: -0.015em;
  }

  h3 {
    margin: 22px 0 6px;
    padding-left: 14px;
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 700;
    letter-spacing: 0.01em;
  }

  /* Settings read as an inset grouped list (F26.1): rows that follow each other share one rounded
     card, parted by a hairline that starts at the text, as in the system settings. */
  .row {
    position: relative;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-4);
    min-height: 44px;
    padding: 8px 14px;
    background: var(--bg-card);
    font-size: 14px;
    font-weight: 500;
  }

  .row:first-child,
  :not(.row) + .row {
    border-top-left-radius: var(--radius-card);
    border-top-right-radius: var(--radius-card);
  }

  .row:last-child,
  .row:has(+ :not(.row)) {
    margin-bottom: 8px;
    border-bottom-left-radius: var(--radius-card);
    border-bottom-right-radius: var(--radius-card);
  }

  .row + .row::before {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    left: 14px;
    height: 1px;
    background: var(--separator);
  }

  .hint {
    display: block;
    margin: 2px 0 0;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 400;
    line-height: 1.4;
  }

  /* A note under a group is indented with the rows, as a footer of the list. */
  p.hint {
    margin: 0 0 8px;
    padding: 0 14px;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  input[role="switch"] {
    position: relative;
    flex: none;
    appearance: none;
    width: 36px;
    height: 22px;
    margin: 0;
    border-radius: var(--radius-chip);
    background: var(--bg-control-hover);
    box-shadow: inset 0 0 0 1px var(--border-subtle);
    cursor: pointer;
    transition: background-color var(--motion-base) var(--ease-out);
  }

  input[role="switch"]::before {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: var(--radius-chip);
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform var(--motion-spring) var(--ease-spring);
  }

  input[role="switch"]:checked {
    background: var(--success);
    box-shadow: none;
  }

  input[role="switch"]:checked::before {
    transform: translateX(14px);
  }

  .confirm {
    margin-top: var(--space-3);
    padding: var(--space-3) 14px;
    border-radius: var(--radius-card);
    background: var(--bg-card);
    font-size: 13px;
    animation: rise-in var(--motion-base) var(--ease-out);
  }

  .confirm p {
    margin: 0;
  }

  .notice {
    margin: var(--space-3) 0 0;
    color: var(--success);
    font-size: 13px;
  }

  .notice.error {
    color: var(--danger);
  }
</style>
