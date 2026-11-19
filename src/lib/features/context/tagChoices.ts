// The tags of a task or a note as ticks in a menu.

import { t } from "../../i18n";
import type { ContextEntry } from "./context.svelte";
import type { ContextDeps } from "./deps";

export function tagChoices(
  deps: ContextDeps,
  current: () => string[],
  toggle: (next: string[]) => void,
): ContextEntry[] {
  if (deps.tags.tags.length === 0) {
    return [{ id: "no-tags", label: t.context.noTags, disabled: true }];
  }
  const ids = current();
  return deps.tags.tags.map((tag) => ({
    id: `tag:${tag.id}`,
    label: tag.name,
    dot: tag.colorHex,
    checked: ids.includes(tag.id),
    keepOpen: true,
    run: () =>
      toggle(
        ids.includes(tag.id)
          ? ids.filter((id) => id !== tag.id)
          : [...ids, tag.id],
      ),
  }));
}
