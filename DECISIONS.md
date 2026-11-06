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
| window-vibrancy (crate, solo Windows) | 0.8.1 | Cargo (F0-07, ya era dependencia transitiva de `tauri`) | 2026-10-31 |
| eslint | 10.14.0 | npm (F0-03) | 2026-10-31 |
| @eslint/js | 10.0.1 | npm (F0-03) | 2026-10-31 |
| typescript-eslint | 8.72.1 | npm (F0-03) | 2026-10-31 |
| eslint-plugin-svelte | 3.23.1 | npm (F0-03) | 2026-10-31 |
| eslint-config-prettier | 10.1.8 | npm (F0-03) | 2026-10-31 |
| globals | 17.13.0 | npm (F0-03) | 2026-10-31 |
| prettier | 3.9.10 | npm (F0-03) | 2026-10-31 |
| prettier-plugin-svelte | 4.1.1 | npm (F0-03) | 2026-10-31 |
| TipTap | pendiente (F3-01) | npm | |
| rusqlite (con `bundled` y `cache`) | 0.41.1 (`Cargo.lock`; libsqlite3-sys 0.39.1) | Cargo (F1-01) | 2026-10-31 |
| tokio | pendiente (F4-01) | Cargo | |
| chrono (solo feature `clock`) | 0.4.45 | Cargo (F1-04) | 2026-10-31 |
| uuid (features `v7`, `std`) | 1.29.0 | Cargo (F1-07, ya era dependencia transitiva de `tauri`) | 2026-11-06 |
| Vitest | 5.1.0 | npm (F1-09) | 2026-11-06 |
| @testing-library/svelte | 5.4.2 | npm (F2-01) | 2026-11-06 |
| jsdom | 31.0.0 | npm (F2-01) | 2026-11-06 |

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

## D-15 — Herramientas de lint y formato
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** F0-03 exige ESLint, Prettier, `svelte-check`, `rustfmt` y `clippy`, y CLAUDE.md §6 prohíbe `TODO`/`FIXME`, `unwrap()`/`expect()`, `todo!()` y `unimplemented!()`.
- **Decisión:**
  - Frontend: ESLint 10 (flat config) con `typescript-eslint` y `eslint-plugin-svelte`; Prettier con `prettier-plugin-svelte`; `eslint-config-prettier` evita conflictos; `globals` aporta los entornos. Todas en versión exacta, en `devDependencies`. `npm run lint` = `prettier --check . && eslint .`; `npm run format` aplica Prettier. La regla `no-warning-comments` falla ante `todo`/`fixme`.
  - Rust: `[lints.clippy]` en `Cargo.toml` con `unwrap_used`, `expect_used`, `todo` y `unimplemented` en `deny`; `clippy.toml` los permite en tests (CLAUDE.md §6). `rustfmt.toml` fija edición 2024 y ancho 100.
  - Prettier usa `endOfLine: auto` porque en Windows `autocrlf` convierte los finales de línea; `.md`, `docs/` y `src-tauri/` quedan fuera de Prettier.
- **Alternativas descartadas:** Biome (sustituiría ESLint y Prettier, pero el soporte de Svelte 5 es menos completo); lint solo con `svelte-check` (no cubre reglas de estilo ni de los límites de CLAUDE.md).

## D-16 — Efectos de ventana mediante `window-vibrancy` directo
- **Estado:** PROPOSED · **Fecha:** 2026-10-31
- **Contexto:** D-01 pide `set_effects` de Tauri y que `window/effects.rs` "devuelva el efecto realmente aplicado". En `tauri` 2.12.5, `Window::set_effects` ignora el error del sistema (`let _ = ...set_window_effects(...)`) y devuelve siempre `Ok`, por lo que con él no se puede detectar que Mica ha fallado ni encadenar el fallback a Acrylic.
- **Decisión propuesta:** llamar a `window_vibrancy::apply_mica` / `apply_acrylic` / `clear_*` (0.8.1, solo `cfg(windows)`), la misma librería que usa Tauri por debajo y cuyas funciones devuelven `Result`. El acceso queda tras el trait `EffectBackend`, así la lógica de fallback se prueba con un backend simulado (DoD de F0). El permiso `core:window:allow-set-effects` se mantiene (SPEC §8.1).
- **Consecuencias:** una dependencia directa nueva, pero con versión idéntica a la ya bloqueada en `Cargo.lock` (sin crates nuevos en el árbol). Afecta a F0-07, F0-08 (el frontend usa el efecto devuelto para la clase `.native-effect`) y F6-05. Si se rechaza, solo hay que sustituir `NativeBackend`.
- **Alternativas descartadas:** `set_effects` de Tauri (no informa del fallo, incumple la consecuencia de D-01); declarar el efecto aplicado sin comprobarlo (la UI mostraría un tinte incorrecto).
- **Acción humana:** confirmar o rechazar. El trabajo continúa porque la sustitución está aislada en un solo tipo.

## D-17 — `rusqlite` con SQLite embebido
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** SPEC §3 fija SQLite vía `rusqlite` con feature `bundled`; F1-01 es la primera tarea que lo usa.
- **Decisión:** `rusqlite = { version = "0.41", default-features = false, features = ["bundled", "cache"] }`. `bundled` compila SQLite (con FTS5) dentro del ejecutable, de modo que no depende de ninguna DLL del sistema y la versión de SQLite es la misma en todos los equipos. `cache` da `prepare_cached` para las consultas repetidas de los repos. Se desactivan las features por defecto porque la única que añaden, `ffi-sqlite-wasm-rs`, arrastra crates solo para WebAssembly. El esquema, las migraciones y los pragmas se gestionan en `db/` sin ORM ni crate de migraciones.
- **Consecuencias:** compilar la dependencia requiere el compilador de C del toolchain (MSVC en Windows, ya necesario para Tauri); primera compilación más lenta. Añade `libsqlite3-sys`, `hashlink`, `fallible-iterator`, `fallible-streaming-iterator` y `vcpkg` (build) al árbol.
- **Alternativas descartadas:** `sqlx` (async y macros innecesarios, árbol de dependencias mayor); `refinery`/`rusqlite_migration` (SPEC §5 define migraciones por `user_version`, que son ~60 líneas propias y se prueban sin dependencia); SQLite del sistema (versión y soporte de FTS5 no garantizados en Windows).

## D-18 — `chrono` para fechas y conversión local/UTC
- **Estado:** ACCEPTED · **Fecha:** 2026-10-31
- **Contexto:** `domain/dates.rs` (F1-04) debe validar fechas de calendario reales y convertir entre hora local y UTC respetando el horario de verano (SPEC §5.1); hacerlo a mano es propenso a errores.
- **Decisión:** `chrono = { version = "0.4.45", default-features = false, features = ["clock"] }`. `clock` aporta `Local` (zona del sistema, en Windows vía `windows-link`). Las funciones reciben la zona como parámetro genérico `TimeZone`, de modo que producción usa `chrono::Local` y los tests usan una zona simulada con DST sin depender de la configuración del equipo. Nunca se llama a `Utc::now()` ni `Local::now()` fuera de la implementación real de `Clock` (F4-01). Hueco de primavera (hora inexistente): se interpreta con el desplazamiento previo al salto (02:30 pasa a 03:30). Hora repetida de otoño: se toma la primera aparición.
- **Consecuencias:** añade `chrono`, `num-traits` y `windows-link` (ya presente en el árbol de Tauri) al árbol. `oldtime`, `wasmbind` y `serde` quedan desactivadas. El formato de transporte sigue siendo texto `YYYY-MM-DD`, `HH:MM` y `YYYY-MM-DDTHH:MM:SSZ`; `chrono` no se expone en los DTO.
- **Alternativas descartadas:** `time` (solo conoce el desplazamiento local actual, no el de una fecha arbitraria, así que no resuelve el DST de fechas futuras); `chrono-tz` como dependencia de pruebas (base IANA completa y tiempo de compilación elevado para cubrir solo dos transiciones, que se simulan en el test); aritmética manual de fechas.

## D-19 — `uuid` para identificadores v7
- **Estado:** ACCEPTED · **Fecha:** 2026-11-06
- **Contexto:** SPEC §5.1 fija ids UUID v7 en texto; F1-07 es la primera tarea que crea filas desde un comando y necesita generarlos.
- **Decisión:** `uuid = { version = "1.29.0", default-features = false, features = ["v7", "std"] }`. `ids::UuidV7Generator` construye el timestamp a partir del `Clock` (nunca de la hora del sistema) y usa `ContextV7` para que los ids del mismo milisegundo sigan siendo crecientes. Los comandos reciben el generador por `AppState` (trait `IdGenerator`), y los tests usan ids secuenciales predecibles.
- **Consecuencias:** sin coste de árbol: `uuid` 1.29.0 ya estaba en `Cargo.lock` como dependencia transitiva de `tauri-utils`. Solo se activan `v7` y `std` (aleatoriedad vía `getrandom`, ya presente).
- **Alternativas descartadas:** generar UUID v7 a mano (formato de bits, contador monótono y aleatoriedad son fáciles de equivocar); `ulid` (formato distinto al de SPEC §5.1); `uuid` con feature `serde` (los ids se transportan como `String`).

## D-20 — Vitest para pruebas del frontend
- **Estado:** ACCEPTED · **Fecha:** 2026-11-06
- **Contexto:** SPEC §3 y §13 fijan Vitest; F1-09 necesita el primer test de contrato IPC y `npm run test` (CLAUDE.md §4).
- **Decisión:** `vitest@5.1.0` exacto como devDependency, con `vitest.config.ts` propio (entorno `node`, `src/**/*.test.ts`) para no cargar los plugins de SvelteKit/Tailwind en los tests. `@testing-library/svelte` y jsdom se añadirán en F2 con su propia entrada, cuando haya componentes que probar. El contrato Rust ↔ TypeScript se prueba con `src-tauri/fixtures/dto.json`: un test de Rust compara la serialización real con el fixture y Vitest lo comprueba contra los tipos de `src/lib/types.ts`.
- **Consecuencias:** solo afecta a desarrollo (no entra en el bundle). `npm audit` informa avisos de dependencias de desarrollo ya presentes; no se ha forzado ninguna actualización.
- **Alternativas descartadas:** Jest (configuración de ESM/TypeScript más pesada con Vite); generar los tipos desde Rust con `ts-rs`/`specta` (dependencia nueva y generación en build; el fixture detecta la misma divergencia sin ella).

## D-21 — Pruebas de componentes: `@testing-library/svelte` y jsdom
- **Estado:** ACCEPTED · **Fecha:** 2026-11-06
- **Contexto:** el DoD de F2 exige tests de componentes (agrupación, completar, deshacer) y un test de renders de fila. D-20 dejó esta entrada pendiente para F2.
- **Decisión:** `@testing-library/svelte@5.4.2` y `jsdom@31.0.0` exactos como devDependencies. `vitest.config.ts` añade `@sveltejs/vite-plugin-svelte` (ya instalado) y el plugin `svelteTesting` de la librería (condición `browser` y limpieza automática). El entorno por defecto sigue siendo `node`; cada test de componente pide jsdom con `// @vitest-environment jsdom`. No se instala `@testing-library/jest-dom` ni `user-event`: bastan `fireEvent` y aserciones sobre el DOM.
- **Consecuencias:** solo desarrollo; no entra en el bundle. `npm audit` mantiene los avisos de herramientas de desarrollo ya conocidos (D-20).
- **Alternativas descartadas:** `happy-dom` (jsdom es la referencia de la librería y de sus ejemplos); Playwright component testing (navegador completo para pruebas que no lo necesitan; los E2E llegan en F6-06).

---

## D-22 — TipTap 3 con extensiones sueltas
- **Estado:** ACCEPTED · **Fecha:** 2026-11-06
- **Contexto:** F3-01/F3-02 exigen el editor de notas de D-10 con negrita, cursiva, código, enlace, listas e historial, y nada más.
- **Decisión:** dependencias exactas `3.33.0` de `@tiptap/core`, `@tiptap/pm`, `@tiptap/extension-document`, `-paragraph`, `-text`, `-bold`, `-italic`, `-code`, `-link`, `-list` (listas con y sin orden), `-hard-break` y `@tiptap/extensions` (historial `UndoRedo`). Se cargan solo desde `NoteEditor` mediante `import()` (D-10).
- **Consecuencias:** entran en un chunk diferido, no en el bundle inicial. El presupuesto de `npm run size` cuenta todos los `.js`, así que se vigila su suma. El enlace se valida en el cliente (`links.ts`) y otra vez en el backend (`validate_content_json`).
- **Alternativas descartadas:** `@tiptap/starter-kit` (arrastra encabezados, citas, bloques de código y tachado que la especificación no pide); Lexical/CodeMirror (otro modelo de documento: D-03 fija ProseMirror JSON).

---

## D-23 — Completar una tarea la mueve al día de completado
- **Estado:** ACCEPTED · **Fecha:** 2026-11-06
- **Contexto:** B-03. D-05 no decía qué pasa con una tarea completada que no tiene fecha. Resolución humana: pasa a AGENDA, al día en que se completó.
- **Decisión (ampliada el 2026-11-06 por petición del usuario: antes solo las tareas sin fecha):** `task_set_completed` acepta `completedDate` (fecha local de hoy, `YYYY-MM-DD`, que solo conoce el frontend); al completar una tarea, sea cual sea su día, `due_date` toma ese valor en la misma transacción (`repo::tasks::set_completed_on`). Solo se asigna fecha, no hora: dentro del día las completadas ya se ordenan por `completed_at`. Reabrir conserva la fecha. Sin `completedDate` la tarea sigue sin fecha.
- **Consecuencias:** POR HACER ya no recibe tareas completadas desde la interfaz (la subsección «Completadas» de ese bloque solo aparecería con datos antiguos); `sort_todo` no cambia.
- **Alternativas descartadas:** dos llamadas (completar y luego actualizar la fecha; no atómico); calcular la fecha local en Rust (el backend no conoce la zona horaria del usuario).

---

## Decisiones pendientes de producto

Estas decisiones **no** pueden tomarse por el agente. Se siguen en `BLOCKERS.md`.

- **Acción "Marcar como hecha" en la notificación** para v1 o posterior.
- **Formato de exportación** además de `.sqlite` (p. ej. JSON).
