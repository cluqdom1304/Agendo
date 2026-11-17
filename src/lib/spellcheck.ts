// WebView2 keeps the red squiggles of the spell checker after a field loses focus. Turning
// `spellcheck` off on blur (and back on at focus) clears them without losing the checking while
// typing. Fields that opted out on their own (`spellcheck="false"`) are left alone.

const EDITABLE = "textarea, [contenteditable='true'], [contenteditable='']";
const MARK = "spellMuted";

function editableOf(target: EventTarget | null): HTMLElement | null {
  return target instanceof HTMLElement ? target.closest(EDITABLE) : null;
}

function onFocusOut(event: FocusEvent) {
  const field = editableOf(event.target);
  if (!field || field.getAttribute("spellcheck") === "false") return;
  field.setAttribute("spellcheck", "false");
  field.dataset[MARK] = "1";
}

function onFocusIn(event: FocusEvent) {
  const field = editableOf(event.target);
  if (!field || field.dataset[MARK] !== "1") return;
  field.setAttribute("spellcheck", "true");
  delete field.dataset[MARK];
}

/** Installs the listeners on `root`; returns the function that removes them. */
export function muteSpellcheckOnBlur(root: Document | HTMLElement): () => void {
  root.addEventListener("focusout", onFocusOut as EventListener);
  root.addEventListener("focusin", onFocusIn as EventListener);
  return () => {
    root.removeEventListener("focusout", onFocusOut as EventListener);
    root.removeEventListener("focusin", onFocusIn as EventListener);
  };
}
