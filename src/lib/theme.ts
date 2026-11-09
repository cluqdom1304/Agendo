import type { ThemeMode } from "./types";

export const THEMES: readonly ThemeMode[] = ["system", "light", "dark"];

export function parseTheme(value: string | null): ThemeMode {
  return THEMES.find((theme) => theme === value) ?? "system";
}

/** `system` removes the attribute so `prefers-color-scheme` decides (tokens.css). */
export function applyTheme(
  theme: ThemeMode,
  root: HTMLElement = document.documentElement,
): void {
  if (theme === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", theme);
}

/** The theme the window really shows: `system` follows the OS. */
export function resolveTheme(theme: ThemeMode): "light" | "dark" {
  if (theme !== "system") return theme;
  const light =
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: light)").matches;
  return light ? "light" : "dark";
}
