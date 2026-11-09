// Accessible-name audit used by component tests (N-08). Not imported by the app.

const NAMED_ROLES = [
  "button",
  "checkbox",
  "radio",
  "switch",
  "combobox",
  "textbox",
  "dialog",
  "grid",
];

const CONTROLS = [
  "button",
  "input:not([type=hidden])",
  "select",
  "textarea",
  "a[href]",
  ...NAMED_ROLES.map((role) => `[role=${role}]`),
].join(",");

function textOf(element: Element, root: Document): string {
  const labelledby = element.getAttribute("aria-labelledby");
  if (labelledby) {
    return labelledby
      .split(/\s+/)
      .map((id) => root.getElementById(id)?.textContent ?? "")
      .join(" ")
      .trim();
  }
  const label = element.getAttribute("aria-label")?.trim();
  if (label) return label;
  const id = element.getAttribute("id");
  const wrapping = element.closest("label")?.textContent?.trim();
  const explicit = id
    ? root.querySelector(`label[for="${id}"]`)?.textContent?.trim()
    : "";
  return (
    wrapping ||
    explicit ||
    element.getAttribute("title")?.trim() ||
    (element instanceof HTMLInputElement && element.type === "text"
      ? element.getAttribute("placeholder")?.trim()
      : "") ||
    element.textContent?.trim() ||
    ""
  );
}

/** Descriptions of the interactive elements under `container` that have no accessible name. */
export function unnamedControls(container: ParentNode = document): string[] {
  const root = container instanceof Document ? container : document;
  return [...container.querySelectorAll(CONTROLS)]
    .filter((element) => !element.closest("[aria-hidden=true]"))
    .filter((element) => !textOf(element, root))
    .map((element) => element.outerHTML.slice(0, 120));
}
