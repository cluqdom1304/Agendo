import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import { defineConfig } from "vitest/config";

// Not the SvelteKit config: tests only need the Svelte compiler, not Tailwind or the router.
// Component tests opt into jsdom with `// @vitest-environment jsdom`.
export default defineConfig({
  plugins: [svelte(), svelteTesting()],
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
    // The token review test reads tokens.css as text (`?raw`); Vitest empties CSS by default.
    css: { include: [/tokens.css/] },
  },
});
