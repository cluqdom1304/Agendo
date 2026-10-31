import { invoke } from "@tauri-apps/api/core";

export type EffectMode = "mica" | "acrylic" | "none";

export interface EffectApplied {
  requested: EffectMode;
  applied: EffectMode;
}

export function windowSetEffect(mode: EffectMode): Promise<EffectApplied> {
  return invoke<EffectApplied>("window_set_effect", { mode });
}
