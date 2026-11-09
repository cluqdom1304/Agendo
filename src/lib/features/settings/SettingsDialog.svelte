<script lang="ts">
  import { onMount } from "svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";

  import { toLocalIsoDate } from "../../dates";
  import { t } from "../../i18n";
  import Modal from "../../shell/Modal.svelte";
  import SegmentedControl from "../../shell/SegmentedControl.svelte";
  import type { SettingsStore } from "../../stores/settings.svelte";
  import type { EffectMode, ThemeMode } from "../../types";

  interface Props {
    store: SettingsStore;
    onClose: () => void;
    /** The data was replaced: views must load it again. */
    onRestored: () => void;
    pickSavePath?: (defaultName: string) => Promise<string | null>;
    pickOpenPath?: () => Promise<string | null>;
    now?: () => Date;
  }

  const filters = [{ name: t.settings.backupFilter, extensions: ["sqlite"] }];

  let {
    store,
    onClose,
    onRestored,
    pickSavePath = (defaultName) => save({ defaultPath: defaultName, filters }),
    pickOpenPath = async () => {
      const chosen = await open({ multiple: false, directory: false, filters });
      return typeof chosen === "string" ? chosen : null;
    },
    now = () => new Date(),
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
    store.notice = null;
  });

  async function createBackup() {
    const path = await pickSavePath(
      t.settings.backupFileName(toLocalIsoDate(now())),
    );
    if (path) await store.createBackup(path);
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

    <div class="buttons">
      <button type="button" disabled={store.busy} onclick={createBackup}>
        {t.settings.backupCreate}
      </button>
      <button
        type="button"
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
            class="danger"
            disabled={store.busy}
            onclick={restoreBackup}
          >
            {t.settings.restoreChoose}
          </button>
          <button type="button" onclick={() => (confirmingRestore = false)}>
            {t.settings.restoreCancel}
          </button>
        </div>
      </div>
    {/if}

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
</Modal>

<style>
  h2 {
    margin: 0 0 var(--space-3);
    font-size: 16px;
  }

  h3 {
    margin: var(--space-4) 0 var(--space-2);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-2) 0;
  }

  .hint {
    display: block;
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .buttons {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  button {
    padding: 6px 14px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: var(--bg-elevated);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  button.danger {
    border-color: var(--danger);
    color: var(--danger);
  }

  .confirm {
    margin-top: var(--space-3);
    padding: var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    font-size: 13px;
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
