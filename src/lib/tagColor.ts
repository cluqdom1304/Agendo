/** Preset colours offered when creating a tag; mid tones that read on both themes. */
export const TAG_PRESETS: readonly string[] = [
  "#4C8DFF",
  "#34B36B",
  "#F08A3C",
  "#E5588A",
  "#8E6CE0",
  "#1FA8A0",
  "#D9A920",
  "#A5805F",
];

const HEX = /^#[0-9a-fA-F]{6}$/;

export function isHexColor(value: string): boolean {
  return HEX.test(value);
}

/**
 * Turns what the user typed into `#RRGGBB` (the only form the backend accepts), or `null`.
 * Accepts a missing `#` and the 3-digit shorthand.
 */
export function normalizeHex(raw: string): string | null {
  const text = raw.trim().replace(/^#/, "");
  const full =
    text.length === 3 ? [...text].map((digit) => digit + digit).join("") : text;
  const hex = `#${full}`;
  return isHexColor(hex) ? hex.toUpperCase() : null;
}
