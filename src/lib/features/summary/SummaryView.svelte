<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  import { addDays } from "../calendar/grid";
  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import { SummaryStore } from "../../stores/summary.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import {
    focusLabel,
    habitRows,
    isQuiet,
    minutesLabel,
    tagRows,
    weekLabel,
  } from "./summary";

  // «Resumen» (F26-02): what the week held, with paging between weeks and an export to Markdown.
  interface Props {
    view: ViewStore;
    /** The tests pass their own store; the app lets the view make it. */
    store?: SummaryStore;
    tags: TagsStore;
    pickSavePath?: (defaultName: string) => Promise<string | null>;
  }

  let {
    view,
    store = new SummaryStore({
      today: () => view.today(),
      firstDay: () => view.firstDay,
    }),
    tags,
    pickSavePath = (defaultName) =>
      save({
        defaultPath: defaultName,
        filters: [{ name: t.summary.markdownFilter, extensions: ["md"] }],
      }),
  }: Props = $props();
  const l = t.summary;

  // Once: an effect would read the week shown and bring it back to today on every page turn.
  onMount(() => {
    void store.show();
  });

  const summary = $derived(store.summary);
  const label = $derived(summary ? weekLabel(summary.start, summary.end) : "");
  const rows = $derived(
    summary
      ? tagRows(
          summary.byTag,
          (id) => {
            const tag = tags.byId.get(id);
            return tag ? { name: tag.name, color: tag.colorHex } : undefined;
          },
          { untagged: l.untagged, gone: l.tagGone },
        )
      : [],
  );
  const habits = $derived(summary ? habitRows(summary.habits) : []);
  const weekdayShort = new Intl.DateTimeFormat("es-ES", { weekday: "short" });
  const bars = $derived.by(() => {
    if (!summary) return [];
    const peak = Math.max(1, ...summary.focusByDay);
    return summary.focusByDay.map((seconds, index) => {
      const date = addDays(summary.start, index);
      return {
        date,
        seconds,
        name: weekdayShort.format(new Date(`${date}T12:00:00`)),
        height: Math.round((seconds / peak) * 100),
      };
    });
  });
  const exportMessage = $derived(
    store.exportState === "done"
      ? l.exported
      : store.exportState === "error"
        ? l.exportFailed
        : "",
  );

  async function exportMarkdown() {
    const path = await pickSavePath(l.exportFileName(summary?.start ?? ""));
    if (path) await store.exportMarkdown(path);
  }
</script>

<div class="summary">
  <header class="head">
    <div>
      <h2>{l.title}</h2>
      <p class="sub">{summary ? l.weekOf(label) : l.subtitle}</p>
    </div>
    <div class="actions">
      <button
        type="button"
        class="btn"
        aria-label={l.previous}
        title={l.previous}
        onclick={() => void store.go(-1)}>‹</button
      >
      <button
        type="button"
        class="btn"
        disabled={store.current}
        onclick={() => void store.show()}>{l.thisWeek}</button
      >
      <button
        type="button"
        class="btn"
        aria-label={l.next}
        title={l.next}
        onclick={() => void store.go(1)}>›</button
      >
      <button
        type="button"
        class="btn primary"
        disabled={!summary}
        onclick={() => void exportMarkdown()}>{l.exportMarkdown}</button
      >
    </div>
  </header>

  <p class="status" role="status" aria-live="polite">{exportMessage}</p>

  {#if store.status === "error" && !summary}
    <EmptyState title={l.error} description={store.error ?? ""} />
    <div class="retry">
      <button type="button" class="btn" onclick={() => void store.refresh()}>
        {l.retry}
      </button>
    </div>
  {:else if !summary}
    <EmptyState variant="loading" title={l.loading} />
  {:else if isQuiet(summary)}
    <EmptyState title={l.quietTitle} description={l.quietDescription} />
  {:else}
    <section class="card" aria-labelledby="sum-tasks">
      <h3 id="sum-tasks">{l.tasks}</h3>
      <dl class="figures">
        <div>
          <dd data-figure="completed">{summary.completed}</dd>
          <dt>{l.completed}</dt>
        </div>
        <div>
          <dd data-figure="pending">{summary.pending}</dd>
          <dt>{l.pending}</dt>
        </div>
        <div class:late={summary.overdue > 0}>
          <dd data-figure="overdue">{summary.overdue}</dd>
          <dt>{l.overdue}</dt>
        </div>
      </dl>
      {#if rows.length > 0}
        <table class="tags">
          <caption>{l.byTag}</caption>
          <thead>
            <tr>
              <th scope="col">{l.byTag}</th>
              <th scope="col">{l.completed}</th>
              <th scope="col">{l.pending}</th>
              <th scope="col">{l.overdue}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as row (row.tagId ?? "none")}
              <tr>
                <th scope="row">
                  {#if row.color}
                    <span class="swatch" style:background={row.color}></span>
                  {/if}
                  {row.name}
                </th>
                <td>{row.completed}</td>
                <td>{row.pending}</td>
                <td>{row.overdue}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="card" aria-labelledby="sum-events">
      <h3 id="sum-events">{l.events}</h3>
      <p class="line">{l.eventCount(summary.events)}</p>
      {#if summary.eventMinutes > 0}
        <p class="muted">{l.eventTime(minutesLabel(summary.eventMinutes))}</p>
      {/if}
    </section>

    <section class="card" aria-labelledby="sum-habits">
      <h3 id="sum-habits">{l.habits}</h3>
      {#if habits.length === 0}
        <p class="muted">{l.noHabits}</p>
      {:else}
        <ul class="habits">
          {#each habits as habit (habit.habitId)}
            <li>
              <span class="name">{habit.title}</span>
              <span class="muted">
                {l.habitProgress(habit.done, habit.scheduled)} · {habit.percent} %
              </span>
              <span
                class="meter"
                role="meter"
                aria-label={habit.title}
                aria-valuemin="0"
                aria-valuemax="100"
                aria-valuenow={habit.percent}
              >
                <span class="fill" style:width="{habit.percent}%"></span>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="card" aria-labelledby="sum-focus">
      <h3 id="sum-focus">{l.focus}</h3>
      {#if summary.focusSeconds === 0}
        <p class="muted">{l.noFocus}</p>
      {:else}
        <p class="line">
          {l.focusTotal(
            focusLabel(summary.focusSeconds),
            summary.focusSessions,
          )}
        </p>
        <ul class="bars">
          {#each bars as bar (bar.date)}
            <li title={bar.seconds > 0 ? focusLabel(bar.seconds) : undefined}>
              <span class="column">
                <span class="bar" style:height="{bar.height}%"></span>
              </span>
              <span class="day">{bar.name}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>

<style>
  .summary {
    display: grid;
    align-content: start;
    gap: var(--space-4);
    box-sizing: border-box;
    width: 100%;
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) var(--space-8);
  }

  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 34px;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.025em;
  }

  h3 {
    margin: 0 0 var(--space-3);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .sub,
  .muted {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .status {
    min-height: 1em;
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .retry {
    display: flex;
    justify-content: center;
  }

  .card {
    padding: var(--space-4);
    border-radius: var(--radius-card);
    background: var(--bg-control);
  }

  .line {
    margin: 0 0 2px;
    font-size: 15px;
    font-weight: 500;
  }

  .figures {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
    margin: 0;
  }

  .figures div {
    display: grid;
    gap: 2px;
  }

  .figures dd {
    margin: 0;
    font-family: var(--font-display);
    font-size: 30px;
    font-weight: 700;
    line-height: 1.1;
  }

  .figures dt {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .figures .late dd {
    color: var(--danger, var(--accent));
  }

  .tags {
    width: 100%;
    margin-top: var(--space-4);
    border-collapse: collapse;
    font-size: 13px;
  }

  .tags caption {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }

  .tags th,
  .tags td {
    padding: 6px 0;
    border-top: 1px solid var(--separator);
    text-align: right;
    font-weight: 400;
  }

  .tags thead th {
    border-top: 0;
    color: var(--text-secondary);
    font-size: 11px;
  }

  .tags th:first-child {
    text-align: left;
  }

  .swatch {
    display: inline-block;
    width: 8px;
    height: 8px;
    margin-right: 6px;
    border-radius: 50%;
  }

  .habits {
    display: grid;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .habits li {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 4px var(--space-3);
    font-size: 13px;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meter {
    grid-column: 1 / -1;
    height: 6px;
    overflow: hidden;
    border-radius: 3px;
    background: var(--separator);
  }

  .fill {
    display: block;
    height: 100%;
    background: var(--accent);
  }

  .bars {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: var(--space-2);
    margin: var(--space-3) 0 0;
    padding: 0;
    list-style: none;
  }

  .bars li {
    display: grid;
    gap: 4px;
    justify-items: center;
  }

  .column {
    display: flex;
    align-items: flex-end;
    width: 100%;
    height: 64px;
  }

  .bar {
    display: block;
    width: 100%;
    min-height: 2px;
    border-radius: 3px 3px 0 0;
    background: var(--accent);
  }

  .day {
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
  }
</style>
