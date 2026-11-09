<script lang="ts">
  import { t } from "../../i18n";
  import { normalizeHex, TAG_PRESETS } from "../../tagColor";

  interface Props {
    /** Current values when editing; the suggested colour when creating. */
    name?: string;
    color: string;
    /** Returns why the tag was refused, or `null` once it is saved. */
    onSave: (
      name: string,
      color: string,
    ) => Promise<"duplicate" | "failed" | null>;
    onDelete?: () => Promise<boolean>;
    onCancel: () => void;
  }

  let { name = "", color, onSave, onDelete, onCancel }: Props = $props();

  // The form starts from the props and then owns its state.
  // svelte-ignore state_referenced_locally
  let draftName = $state(name);
  // svelte-ignore state_referenced_locally
  let hexText = $state(color.toUpperCase());
  let error = $state("");
  let saving = $state(false);
  let confirming = $state(false);

  const hex = $derived(normalizeHex(hexText));
  const hexInvalid = $derived(hexText.trim() !== "" && hex === null);
  const canSave = $derived(draftName.trim() !== "" && hex !== null && !saving);

  async function submit() {
    if (!canSave || hex === null) return;
    saving = true;
    error = "";
    const failure = await onSave(draftName.trim(), hex);
    saving = false;
    if (failure) {
      error = failure === "duplicate" ? t.tags.duplicate : t.tags.saveFailed;
    }
  }

  async function remove() {
    if (!onDelete) return;
    saving = true;
    const removed = await onDelete();
    saving = false;
    if (!removed) error = t.tags.deleteFailed;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    onCancel();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<form
  class="editor"
  onsubmit={(event) => {
    event.preventDefault();
    void submit();
  }}
  onkeydown={onKeydown}
>
  <label class="field">
    <span class="label">{t.tags.nameLabel}</span>
    <input
      type="text"
      class="text"
      maxlength="40"
      autocomplete="off"
      placeholder={t.tags.namePlaceholder}
      data-autofocus
      bind:value={draftName}
    />
  </label>

  <div class="field">
    <span class="label" id="tag-color-label">{t.tags.colorLabel}</span>
    <div class="swatches" role="radiogroup" aria-labelledby="tag-color-label">
      {#each TAG_PRESETS as preset (preset)}
        <button
          type="button"
          role="radio"
          class="swatch"
          class:selected={hex === preset}
          aria-checked={hex === preset}
          aria-label={t.tags.presetLabel(preset)}
          style:--tag={preset}
          onclick={() => (hexText = preset)}
        ></button>
      {/each}
    </div>
    <div class="custom">
      <input
        type="color"
        class="picker"
        aria-label={t.tags.pickColor}
        title={t.tags.pickColor}
        value={(hex ?? "#4C8DFF").toLowerCase()}
        oninput={(event) => (hexText = event.currentTarget.value.toUpperCase())}
      />
      <input
        type="text"
        class="text hex"
        maxlength="7"
        spellcheck="false"
        autocomplete="off"
        aria-label={t.tags.hexLabel}
        aria-invalid={hexInvalid}
        bind:value={hexText}
      />
    </div>
    {#if hexInvalid}
      <p class="error" role="alert">{t.tags.hexInvalid}</p>
    {/if}
  </div>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  {#if confirming}
    <p class="confirm">{t.tags.confirmDelete}</p>
    <div class="actions">
      <button type="button" onclick={() => (confirming = false)}>
        {t.tags.cancel}
      </button>
      <button type="button" class="danger" disabled={saving} onclick={remove}>
        {t.tags.confirmYes}
      </button>
    </div>
  {:else}
    <div class="actions">
      {#if onDelete}
        <button
          type="button"
          class="danger-text"
          onclick={() => (confirming = true)}
        >
          {t.tags.delete}
        </button>
      {/if}
      <span class="spacer"></span>
      <button type="button" onclick={onCancel}>{t.tags.cancel}</button>
      <button type="submit" class="primary" disabled={!canSave}>
        {t.tags.save}
      </button>
    </div>
  {/if}
</form>

<style>
  .editor {
    box-sizing: border-box;
    max-width: 100%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: var(--bg-elevated);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .label {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .text {
    min-width: 0;
    padding: 4px 8px;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
  }

  .text:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 0;
  }

  .text[aria-invalid="true"] {
    outline: 2px solid var(--danger);
    outline-offset: 0;
  }

  .hex {
    flex: 1;
    font-family: var(--font-mono);
    text-transform: uppercase;
  }

  .swatches {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 4px;
  }

  .swatch {
    aspect-ratio: 1;
    min-width: 0;
    padding: 0;
    border: 2px solid transparent;
    border-radius: var(--radius-chip);
    background: var(--tag);
    cursor: pointer;
  }

  .swatch.selected {
    border-color: var(--text-primary);
    box-shadow: inset 0 0 0 2px var(--bg-window-solid);
  }

  .swatch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .custom {
    display: flex;
    gap: 6px;
  }

  .picker {
    flex: none;
    width: 32px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: transparent;
    cursor: pointer;
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 11px;
  }

  .confirm {
    margin: 0;
    color: var(--text-primary);
    font-size: 12px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }

  .spacer {
    flex: 1;
  }

  .actions button {
    flex: none;
    padding: 4px 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .actions button:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .actions button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .actions .primary {
    border-color: transparent;
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 600;
  }

  .actions .danger {
    border-color: transparent;
    background: var(--danger);
    color: var(--on-danger);
  }

  .actions .danger-text {
    border-color: transparent;
    color: var(--danger);
  }

  .actions button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
