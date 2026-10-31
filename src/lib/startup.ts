import { startupMark } from "$lib/api";

/** Logs `first_paint` after the browser has had two frames to paint the initial UI. */
export function markFirstPaint(): void {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      startupMark("first_paint").catch(() => {
        // Sin backend de Tauri (vista previa en navegador) no hay dónde registrar la marca.
      });
    });
  });
}
