<script lang="ts">
  import { findShortcut } from "../../keymap";

  interface Props {
    value: string;
    label: string;
    placeholder?: string;
    completed?: boolean;
    /** Focus the input as soon as it is created (new-task drafts). */
    focusOnMount?: boolean;
    /** Enter or focus loss with something to save. `enter` also asks for the next task. */
    onCommit: (text: string, trigger: "enter" | "blur") => void;
    /** Esc, or focus loss on an empty draft. */
    onCancel?: () => void;
  }

  let {
    value,
    label,
    placeholder = "",
    completed = false,
    focusOnMount = false,
    onCommit,
    onCancel,
  }: Props = $props();

  let input = $state<HTMLInputElement>();
  // `null` while the input is not being edited, so it always shows the stored title.
  let typed = $state<string | null>(null);

  $effect(() => {
    if (focusOnMount) input?.focus();
  });

  const currentText = () => (typed ?? value).trim();

  function onkeydown(event: KeyboardEvent) {
    if (event.isComposing) return;
    const shortcut = findShortcut(event, "title");
    if (shortcut === "confirmTitle") {
      event.preventDefault();
      const text = currentText();
      if (onCancel && !text) onCancel();
      else onCommit(text, "enter");
      // A draft keeps its text until the backend accepts it.
      if (!onCancel) typed = null;
    } else if (shortcut === "cancelTitle") {
      event.preventDefault();
      typed = null;
      onCancel?.();
      input?.blur();
    }
  }

  function onblur() {
    const text = currentText();
    if (onCancel) {
      if (text) onCommit(text, "blur");
      else onCancel();
      return;
    }
    typed = null;
    if (text && text !== value) onCommit(text, "blur");
  }
</script>

<input
  bind:this={input}
  data-title-input
  type="text"
  class="title"
  class:completed
  value={typed ?? value}
  {placeholder}
  aria-label={label}
  maxlength={500}
  spellcheck="false"
  autocomplete="off"
  onfocus={() => (typed = value)}
  oninput={(event) => (typed = event.currentTarget.value)}
  {onkeydown}
  {onblur}
/>

<style>
  .title {
    flex: 1;
    min-width: 0;
    padding: 3px 6px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    outline: none;
    text-overflow: ellipsis;
    transition: background-color var(--motion-instant) linear;
  }

  .title::placeholder {
    color: var(--text-secondary);
  }

  .title:focus {
    background: var(--bg-control);
  }

  .title.completed {
    color: var(--text-secondary);
    text-decoration: line-through;
  }
</style>
