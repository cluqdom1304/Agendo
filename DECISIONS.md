# DECISIONS.md — Registro de decisiones

Formato de cada entrada: contexto, decisión, consecuencias y alternativas descartadas.
Estados: `PROPOSED` (pendiente de confirmación humana), `ACCEPTED`, `SUPERSEDED por D-YY`.

Cambiar una decisión `ACCEPTED` requiere un ADR nuevo que la sustituya; nunca editar su contenido.

---

## Versiones fijadas

Fijadas en F0-01 (2026-10-31). No actualizar sin tarea dedicada.

| Componente | Versión exacta | Origen | Fecha de fijado |
|------------|----------------|--------|-----------------|
| Tauri (crate `tauri`) | 2.12.5 | Cargo | 2026-10-31 |
| tauri-build (crate) | 2.7.4 | Cargo (build) | 2026-10-31 |
| tauri-plugin-opener (crate) | 2.7.2 | Cargo (plantilla) | 2026-10-31 |
| @tauri-apps/api | 2.12.4 | npm | 2026-10-31 |
| @tauri-apps/plugin-opener | 2.7.2 | npm (plantilla) | 2026-10-31 |
| @tauri-apps/cli | 2.12.4 | npm (plantilla) | 2026-10-31 |
| tauri-plugin-notification | pendiente (F4-05) | Cargo + npm | |
| serde | 1.0.229 | Cargo | 2026-10-31 |
| serde_json | 1.0.154 | Cargo | 2026-10-31 |
| Svelte | 5.57.3 | npm | 2026-10-31 |
| @sveltejs/kit | 2.70.3 | npm (plantilla) | 2026-10-31 |
| @sveltejs/vite-plugin-svelte | 7.3.1 | npm (plantilla) | 2026-10-31 |
| svelte-check | 4.7.7 | npm (plantilla) | 2026-10-31 |
| Vite | 8.3.5 | npm | 2026-10-31 |
| TypeScript | 6.0.3 | npm (plantilla) | 2026-10-31 |
| Tailwind CSS (`tailwindcss`) | 4.3.3 | npm (F0-02) | 2026-10-31 |
| @tailwindcss/vite | 4.3.3 | npm (F0-02) | 2026-10-31 |
| TipTap | pendiente (F3-01) | npm | |
| rusqlite | pendiente (F1-01) | Cargo | |
| tokio | pendiente (F4-01) | Cargo | |
| chrono | pendiente (F1-04) | Cargo | |
| Vitest | pendiente (F1 o F2) | npm | |

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

## D-12 — SvelteKit en modo SPA
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** la plantilla oficial de Tauri para Svelte usa SvelteKit. La especificación (§3) pedía Svelte 5 con Vite.
- **Decisión:** usar SvelteKit con `adapter-static` en modo SPA (sin SSR). Las rutas no se usan para navegación; la app es una única ventana. Svelte 5 y Vite siguen siendo los motores reales.
- **Consecuencias:** no se usa SSR ni prerenderizado de páginas. `svelte.config.js` debe configurar el adaptador estático con `fallback` a `index.html`. Claude no debe añadir `+page.server.ts` ni endpoints.
- **Alternativas descartadas:** recrear el proyecto con Svelte puro (más trabajo sin beneficio para una app de escritorio).

## D-13 — Plugin opener de la plantilla
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Decisión:** se mantiene `tauri-plugin-opener` porque lo genera la plantilla y sirve para abrir enlaces `http`, `https` y `mailto` (F3-06). Sus permisos se limitan a esos esquemas en `capabilities/default.json`.
- **Alternativas descartadas:** eliminarlo y gestionar enlaces a mano (más código propio sin ventaja).

## D-14 — Tailwind CSS v4 con plugin de Vite
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** SPEC §3 exige Tailwind CSS v4 mediante su plugin de Vite, solo para utilidades de layout; los tokens de diseño son CSS propio (F0-08).
- **Decisión:** `tailwindcss` y `@tailwindcss/vite` en `devDependencies`, ambos en 4.3.3 con versión exacta (sin `^`). El plugin va antes de `sveltekit()` en `vite.config.js`. Sin `tailwind.config.js` ni PostCSS: la configuración es CSS (`src/app.css`, `@import "tailwindcss"`), importado desde `src/routes/+layout.svelte`.
- **Consecuencias:** no se versiona CSS generado; coste en el bundle CSS a vigilar con `size-check` (F0-09).
- **Alternativas descartadas:** Tailwind v3 con PostCSS (contradice la especificación); CSS puro (más código propio para layout).

---

## Decisiones pendientes de producto

Estas decisiones **no** pueden tomarse por el agente. Se siguen en `BLOCKERS.md`.

- **Acción "Marcar como hecha" en la notificación** para v1 o posterior.
- **Formato de exportación** además de `.sqlite` (p. ej. JSON).
