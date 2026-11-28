<script lang="ts">
  import { t } from "../../i18n";
  import type { AppInfo } from "../../types";

  interface Props {
    loadInfo: () => Promise<AppInfo>;
  }

  let { loadInfo }: Props = $props();

  let infoOpen = $state(false);
  let info = $state.raw<AppInfo | null>(null);
  let infoFailed = $state(false);

  async function toggleInfo() {
    infoOpen = !infoOpen;
    if (!infoOpen || info) return;
    infoFailed = false;
    try {
      info = await loadInfo();
    } catch {
      infoFailed = true;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    const units = ["KB", "MB", "GB"];
    let value = bytes / 1024;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toLocaleString("es-ES", { maximumFractionDigits: 1 })} ${units[unit]}`;
  }

  const infoRows = $derived.by(() => {
    if (!info) return [];
    const l = t.settings.aboutRows;
    return [
      [l.version, info.version],
      [l.build, t.settings.aboutBuild[info.build]],
      [l.os, `${info.os} (${info.arch})`],
      [l.webview, info.webviewVersion ?? t.settings.aboutUnknown],
      [l.tauri, info.tauriVersion],
      [l.sqlite, info.sqliteVersion],
      [l.schema, String(info.schemaVersion)],
      [l.dataDir, info.dataDir],
      [l.size, formatBytes(info.databaseBytes)],
      [l.tasks, String(info.tasks)],
      [l.notes, String(info.notes)],
      [l.tags, String(info.tags)],
      [l.attachments, String(info.attachments)],
    ] as const;
  });
</script>

<section aria-labelledby="settings-about">
  <h3 id="settings-about">{t.settings.about}</h3>
  <div class="buttons">
    <button
      type="button"
      class="btn"
      aria-expanded={infoOpen}
      onclick={toggleInfo}
    >
      {t.settings.aboutButton}
    </button>
  </div>
  {#if infoOpen}
    {#if infoFailed}
      <p class="notice error" role="alert">{t.settings.aboutFailed}</p>
    {:else if !info}
      <p class="hint" role="status">{t.settings.aboutLoading}</p>
    {:else}
      <dl class="info">
        {#each infoRows as [label, value] (label)}
          <dt>{label}</dt>
          <dd>{value}</dd>
        {/each}
      </dl>
      <p class="hint">{t.settings.aboutPrivacy}</p>
    {/if}
  {/if}
</section>

<style>
  h3 {
    margin: 22px 0 6px;
    padding-left: 14px;
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 700;
    letter-spacing: 0.01em;
  }

  .hint {
    display: block;
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-caption);
  }

  .info {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 4px var(--space-4);
    margin: 8px 0;
    padding: 12px 14px;
    border-radius: var(--radius-card);
    background: var(--bg-card);
    font-size: 13px;
  }

  .info dt {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .info dd {
    margin: 0;
    font-weight: 500;
    overflow-wrap: anywhere;
    user-select: text;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-2);
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
