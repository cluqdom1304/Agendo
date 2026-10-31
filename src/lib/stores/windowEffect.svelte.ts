import { windowSetEffect, type EffectMode } from "$lib/api";

class WindowEffectStore {
  applied = $state<EffectMode>("none");
  isNative = $derived(this.applied !== "none");

  async request(mode: EffectMode): Promise<void> {
    try {
      const result = await windowSetEffect(mode);
      this.applied = result.applied;
    } catch (error) {
      console.error("window_set_effect failed:", error);
      this.applied = "none";
    }
  }
}

export const windowEffect = new WindowEffectStore();
