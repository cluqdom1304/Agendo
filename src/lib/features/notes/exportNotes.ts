// Exporting one note or one folder from its menu (F14-01). The settings dialog has the same
// action for everything; both ask the backend to write the files into a folder the user picks.
import { open } from "@tauri-apps/plugin-dialog";

import * as backend from "../../api";
import { t } from "../../i18n";
import { describeError } from "../../stores/settings.svelte";
import type { Notice } from "../../stores/notes.svelte";
import type { ExportFormat, ExportKind, ExportReport } from "../../types";

export interface ExportDeps {
  pickFolder(): Promise<string | null>;
  exportNotes(
    format: ExportFormat,
    kind: ExportKind,
    id: string | null,
    path: string,
  ): Promise<ExportReport>;
}

const realDeps: ExportDeps = {
  pickFolder: async () => {
    const chosen = await open({ directory: true, multiple: false });
    return typeof chosen === "string" ? chosen : null;
  },
  exportNotes: backend.exportNotes,
};

/** Asks for a destination and exports; `null` when the user backed out. */
export async function exportSelection(
  format: ExportFormat,
  kind: ExportKind,
  id: string | null,
  deps: ExportDeps = realDeps,
): Promise<Notice | null> {
  const path = await deps.pickFolder();
  if (!path) return null;
  try {
    const report = await deps.exportNotes(format, kind, id, path);
    return {
      kind: "info",
      message: t.notesApp.exportedTo(report.notes, report.files),
    };
  } catch (error) {
    return {
      kind: "error",
      message: `${t.notesApp.exportFailed} ${describeError(error)}`,
    };
  }
}
