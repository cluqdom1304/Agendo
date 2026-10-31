# DECISIONS.md — Registro de decisiones

Formato de cada entrada: contexto, decisión, consecuencias y alternativas descartadas.
Estados: `PROPOSED` (pendiente de confirmación humana), `ACCEPTED`, `SUPERSEDED por D-YY`.

Cambiar una decisión `ACCEPTED` requiere un ADR nuevo que la sustituya; nunca editar su contenido.

---

## Versiones fijadas

Completar en F0-01. No actualizar sin tarea dedicada.

| Componente | Versión exacta | Fecha de fijado |
|------------|----------------|-----------------|
| Tauri (crate `tauri`) | pendiente | |
| `@tauri-apps/api` | pendiente | |
| `@tauri-apps/cli` | pendiente | |
| `tauri-plugin-notification` | pendiente | |
| Svelte | pendiente | |
| Vite | pendiente | |
| TypeScript | pendiente | |
| Tailwind CSS (+ plugin Vite) | pendiente | |
| TipTap | pendiente | |
| rusqlite | pendiente | |
| tokio | pendiente | |
| chrono | pendiente | |
| Vitest | pendiente | |

---

## D-01 — Efectos Mica/Acrylic nativos
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** la especificación combinaba efecto nativo y `backdrop-filter` en CSS, lo que duplica el desenfoque y consume GPU.
- **Decisión:** usar `set_effects` de Tauri con Mica; Acrylic como fallback; sólido como último recurso. Con efecto nativo activo, el CSS usa tinte sin `backdrop-filter` (clase `.native-effect`).
- **Consecuencias:** `window/effects.rs` debe devolver el efecto realmente aplicado. Tests de fallback con backend simulado.
- **Alternativas descartadas:** `backdrop-filter` sobre ventana transparente (más coste y comportamiento inconsistente entre equipos).

## D-02 — RAM como objetivo medido, no garantizado
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** WebView2 lanza varios procesos hijos; la suma supera fácilmente los 40 MB.
- **Decisión:** N-02 es un objetivo para el proceso principal. Se reporta también el total con procesos WebView2. La verificación es `[H]`.
- **Consecuencias:** las métricas se registran en `PROGRESS.md` con el método usado.
- **Alternativas descartadas:** comprometer la cifra sin medición.

## D-03 — Contenido en JSON ProseMirror + texto derivado para FTS
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** TipTap guarda ProseMirror JSON; la búsqueda necesita texto plano; la compatibilidad futura con Markdown necesita conservar la estructura.
- **Decisión:** `content_json` es la fuente de verdad. `content_text` se deriva en cada guardado y alimenta FTS5.
- **Consecuencias:** cualquier cambio de extracción de texto exige reindexar (comando interno de mantenimiento).
- **Alternativas descartadas:** solo texto plano (pierde formato); solo JSON sin índice (búsqueda lenta).

## D-04 — Scheduler con sleep hasta el próximo aviso
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** el polling cada 20–30 s despierta el sistema sin necesidad; las notificaciones deben ser precisas.
- **Decisión:** `sleep_until(min(próximo_notify_at, ahora + 60 s))` más reprogramación mediante `Notify`. El tope de 60 s detecta suspensión y saltos de reloj.
- **Consecuencias:** toda la lógica temporal usa el trait `Clock`, lo que permite tests deterministas.
- **Alternativas descartadas:** polling fijo (más despertares); solo eventos del sistema (poco fiables).

## D-05 — Orden de la sección AGENDA
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** la especificación pedía fechas descendentes pero no definía el orden interno.
- **Decisión:**
  - POR HACER: `position` ascendente.
  - AGENDA: `due_date` descendente.
  - Dentro de cada día: primero tareas con hora (hora ascendente), luego sin hora (`position`).
  - Completadas: subsección al final del bloque del día, ordenadas por `completed_at` ascendente.
- **Consecuencias:** `domain/ordering.rs` concentra la regla y tiene tests de tabla.
- **Alternativas descartadas:** orden global por creación.

## D-06 — Etiquetas con un nivel de jerarquía
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Decisión:** `tags.parent_id` permite un único nivel (p. ej. "Grado" → asignatura). Validado en `domain/validation.rs`.
- **Alternativas descartadas:** árbol arbitrario (más validación y UI sin necesidad en v1).

## D-07 — Recurrencia fuera de v1
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Decisión:** columna `tasks.rrule` nullable reservada; no se lee ni escribe en v1.
- **Consecuencias:** la implementación futura usará una migración nueva; no se toca `001_init.sql`.

## D-08 — Errores IPC como `String` con prefijo de código
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** la especificación pide `Result<T, String>`.
- **Decisión:** el `String` tiene formato `CODIGO: mensaje`. Códigos: `NOT_FOUND`, `VALIDATION`, `CONFLICT`, `DB`, `IO`, `NOTIFY`, `INTERNAL`. El frontend muestra solo el mensaje humano.
- **Revisión:** si el parseo por prefijo resulta frágil, abrir ADR para `AppError` serializable.
- **Alternativas descartadas:** `AppError` desde el inicio (contradice la especificación sin necesidad demostrada).

## D-09 — Copias de seguridad con `VACUUM INTO`
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Decisión:** `backup_create` usa `VACUUM INTO`; `backup_restore` valida con `PRAGMA integrity_check` antes de sustituir y conserva copia de la base anterior.
- **Alternativas descartadas:** copiar el `.sqlite` en caliente (riesgo de inconsistencia con WAL).

## D-10 — TipTap con carga diferida
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Decisión:** el módulo del editor se carga con `import()` al abrir la primera nota.
- **Consecuencias:** verificar en `size-check` que el bundle inicial no incluye TipTap.

## D-11 — Borrado en FTS5 por `task_id`
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** `tasks_fts` no es external-content; borrar requiere `DELETE ... WHERE task_id = ?` sobre una columna UNINDEXED, que recorre la tabla.
- **Decisión:** aceptable para el volumen objetivo (10 000 tareas). Si N-05 o F-09 fallan, revisar con una columna de `rowid` estable y tabla FTS external-content.
- **Consecuencias:** el test de rendimiento de F1-03 mide esta operación.

---

## Decisiones pendientes de producto

Estas decisiones **no** pueden tomarse por el agente. Se siguen en `BLOCKERS.md`.

- **Acción "Marcar como hecha" en la notificación** para v1 o posterior.
- **Formato de exportación** además de `.sqlite` (p. ej. JSON).
