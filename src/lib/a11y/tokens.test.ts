import { describe, expect, it } from "vitest";

import css from "../../styles/tokens.css?raw";
import { composite, contrastRatio, parseColor, type Rgba } from "./contrast";

type Tokens = Record<string, string>;

/** Declarations of the block that starts with `selector`. */
function block(selector: string): Tokens {
  const start = css.indexOf(selector);
  if (start < 0) throw new Error(`Missing block ${selector}`);
  const open = css.indexOf("{", start);
  const close = css.indexOf("}", open);
  const tokens: Tokens = {};
  for (const match of css
    .slice(open + 1, close)
    .matchAll(/([\w-]+):\s*([^;]+);/g)) {
    tokens[match[1]] = match[2].trim();
  }
  return tokens;
}

const THEMES: Record<string, Tokens> = {
  "dark (forced)": block(':root[data-theme="dark"]'),
  "light (forced)": block(':root[data-theme="light"]'),
  "light (system)": block(':root:not([data-theme="dark"])'),
};

const WHITE: Rgba = { r: 255, g: 255, b: 255, a: 1 };
const BLACK: Rgba = { r: 0, g: 0, b: 0, a: 1 };

const TEXT_TOKENS = [
  "--text-primary",
  "--text-secondary",
  "--accent",
  "--danger",
  "--success",
  ...Array.from({ length: 8 }, (_, i) => `--tag-${i + 1}`),
];

/** AA for normal text. */
const AA = 4.5;

describe("design tokens", () => {
  it("define the same set of names in every theme", () => {
    const reference = Object.keys(THEMES["dark (forced)"]).sort();
    for (const [name, tokens] of Object.entries(THEMES)) {
      expect(Object.keys(tokens).sort(), name).toEqual(reference);
    }
  });

  it("forced light matches the light block that follows the system", () => {
    expect(THEMES["light (forced)"]).toEqual(THEMES["light (system)"]);
  });

  describe.each(Object.entries(THEMES))("%s", (_name, tokens) => {
    const dark = tokens["color-scheme"] === "dark";
    // With a native effect the backdrop is Windows' own material, which Windows keeps close to
    // #202020 in the dark theme and #f3f3f3 in the light one. Real measurement is [H].
    const material: Rgba = dark
      ? { r: 32, g: 32, b: 32, a: 1 }
      : { r: 243, g: 243, b: 243, a: 1 };

    /** Every surface text can sit on, over the backdrops it can meet. */
    const surfaces: [string, Rgba][] = [
      [
        "solid window over black",
        composite(parseColor(tokens["--bg-window-solid"]), BLACK),
      ],
      [
        "solid window over white",
        composite(parseColor(tokens["--bg-window-solid"]), WHITE),
      ],
      [
        "translucent window over the material",
        composite(parseColor(tokens["--bg-window"]), material),
      ],
      [
        "native window over the material",
        composite(parseColor(tokens["--bg-window-native"]), material),
      ],
    ];

    it.each(TEXT_TOKENS)(
      "%s reaches WCAG AA on the window surfaces",
      (token) => {
        for (const [label, background] of surfaces) {
          expect(
            contrastRatio(parseColor(tokens[token]), background),
            `${token} on ${label}`,
          ).toBeGreaterThanOrEqual(AA);
        }
      },
    );

    it.each([
      ["--on-accent", "--accent"],
      ["--on-danger", "--danger"],
    ])("%s reaches AA on %s", (foreground, background) => {
      expect(
        contrastRatio(
          parseColor(tokens[foreground]),
          parseColor(tokens[background]),
        ),
      ).toBeGreaterThanOrEqual(AA);
    });

    it("primary and secondary text reach AA on elevated surfaces", () => {
      for (const [label, window] of surfaces) {
        const elevated = composite(parseColor(tokens["--bg-elevated"]), window);
        for (const token of ["--text-primary", "--text-secondary"]) {
          expect(
            contrastRatio(parseColor(tokens[token]), elevated),
            `${token} on elevated, ${label}`,
          ).toBeGreaterThanOrEqual(AA);
        }
      }
    });
  });
});
