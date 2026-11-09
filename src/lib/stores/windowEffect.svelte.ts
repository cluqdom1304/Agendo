import { windowSetEffect, type EffectMode } from "../api";
import type { ThemeMode } from "../types";

class WindowEffectStore {
  applied = $state<EffectMode>("none");
  isNative = $derived(this.applied !== "none");

  async request(mode: EffectMode, theme?: ThemeMode): Promise<void> {
    try {
      const result = await windowSetEffect(mode, theme);
      this.applied = result.applied;
    } catch (error) {
      console.error("window_set_effect failed:", error);
      this.applied = "none";
    }
  }
}

export const windowEffect = new WindowEffectStore();
