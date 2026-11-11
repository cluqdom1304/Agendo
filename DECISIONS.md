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
- **Estado:** ACCEPTED (confirmada al cerrar la V1, 2026-11-09) · **Fecha:** 2026-10-31
- **Contexto:** D-01 pide `set_effects` de Tauri y que `window/effects.rs` "devuelva el efecto realmente aplicado". En `tauri` 2.12.5, `Window::set_effects` ignora el error del sistema (`let _ = ...set_window_effects(...)`) y devuelve siempre `Ok`, por lo que con él no se puede detectar que Mica ha fallado ni encadenar el fallback a Acrylic.
- **Decisión:** llamar a `window_vibrancy::apply_mica` / `apply_acrylic` / `clear_*` (0.8.1, solo `cfg(windows)`), la misma librería que usa Tauri por debajo y cuyas funciones devuelven `Result`. El acceso queda tras el trait `EffectBackend`, así la lógica de fallback se prueba con un backend simulado (DoD de F0). El permiso `core:window:allow-set-effects` se mantiene (SPEC §8.1).
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

## D-24 — `tokio` directo para el scheduler
- **Estado:** ACCEPTED · **Fecha:** 2026-11-07
- **Contexto:** SPEC §6.1 exige `tokio::select!`, `sleep_until` y `Notify`. `tauri::async_runtime` solo expone el runtime, no esas primitivas.
- **Decisión:** `tokio 1` (ya presente en el árbol por Tauri, versión 1.54.1) con `default-features = false` y solo `sync`, `time`, `macros`, `rt`.
- **Consecuencias:** sin crate nuevo en el binario final; solo se declara como dependencia directa.
- **Alternativas descartadas:** bucle con `std::thread::sleep` + `Condvar` (reimplementa `select!` y es más difícil de probar con reloj simulado).

---

## D-25 — `tauri-plugin-notification` para avisos nativos
- **Estado:** ACCEPTED · **Fecha:** 2026-11-07
- **Contexto:** F-13 y F4-05 piden notificaciones nativas de Windows 11.
- **Decisión:** `tauri-plugin-notification 2.8.0`. Solo se usa desde Rust (`app.notification().builder()`), por lo que no se concede ningún permiso de frontend en `capabilities/default.json`. Sin acción «Marcar como hecha» en v1 (B-01).
- **Consecuencias:** el plugin no hace red. En `tauri dev` Windows atribuye el aviso al host de PowerShell; con el ejecutable instalado usa el `identifier` de la app (comprobación [H]).
- **Alternativas descartadas:** `notify-rust` directo (pierde la integración con el identifier de Tauri); `windows` crate/WinRT directo (más superficie y `unsafe`).

---

## D-26 — Semántica de los recordatorios
- **Estado:** ACCEPTED · **Fecha:** 2026-11-07
- **Contexto:** SPEC §5.3 y §6 no cubren varios casos límite.
- **Decisión:** (1) un recordatorio con `offsetMinutes` exige que la tarea tenga fecha **y** hora; `notify_at` = fecha/hora local convertida a UTC (`Local`, vía `domain/dates.rs`) menos el offset. (2) Solo disparan tareas no completadas y no eliminadas; los de tareas completadas quedan pendientes y se reevalúan si se reabre la tarea. (3) Al quitar fecha u hora a una tarea se borran sus recordatorios con offset aún no enviados; los personalizados y los enviados no se tocan. (4) Backoff 30 s, 60 s, 120 s, 120 s (5 intentos); el estado de reintento vive en memoria. (5) La caducidad de 24 h (§6.4) se aplica en cada vuelta del bucle, no solo al arrancar, para cubrir suspensiones largas.
- **Consecuencias:** el arranque no necesita código aparte: la primera vuelta del bucle recupera.
- **Alternativas descartadas:** asumir una hora por defecto para tareas sin hora (decisión de producto no pedida).

---

## D-27 — Colores de los tokens ajustados para cumplir WCAG AA (N-08)
- **Estado:** ACCEPTED (confirmada al cerrar la V1, 2026-11-09) · **Fecha:** 2026-11-09
- **Contexto:** la revisión automatizada de F6-02 (`src/lib/a11y/tokens.test.ts`) mostró que varios colores de SPEC §8.3 y los añadidos por el agente en F0-08 no llegan a 4,5:1 como texto: en el tema claro `--text-secondary`, `--accent`, `--danger`, `--success` y las etiquetas 1–4, 6 y 7; en el oscuro, `--accent` y `--danger`. El blanco fijo sobre `--accent` tampoco cumple en oscuro.
- **Decisión:** se oscurecen (claro) o aclaran (oscuro) esos tokens lo mínimo necesario (valores en `tokens.css`) y se añaden `--on-accent` y `--on-danger` para el texto sobre fondos de acento y de peligro. El test falla si algún token vuelve a bajar de 4,5:1 sobre las superficies de la ventana.
- **Consecuencias:** el azul de acento claro pasa de `#0071e3` a `#0061c3` y el oscuro de `#0a84ff` a `#2692ff`; en oscuro el texto sobre botones de acento es casi negro. No edita `docs/SPEC.md`: espera confirmación humana. La medición real sobre Mica es `[H]`.
- **Alternativas descartadas:** conservar los colores de la especificación y limitar el texto a tamaño grande (3:1), porque el acento y los avisos se usan con texto de 12–14 px.

---

## D-28 — `tauri-plugin-dialog` para elegir el archivo de la copia de seguridad
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** F-16 y F6-04 necesitan «Guardar como» y «Abrir» nativos.
- **Decisión:** `tauri-plugin-dialog 2.8.4` (y `@tauri-apps/plugin-dialog ^2.8.4`). Permisos concedidos a la ventana principal: solo `dialog:allow-open` y `dialog:allow-save` (el test `capabilities.rs` fija el conjunto). El plugin arrastra `tauri-plugin-fs` como dependencia interna, **sin** conceder ningún permiso `fs:`: la lectura y escritura de la copia las hace Rust.
- **Alternativas descartadas:** `rfd` directo (duplica lo que el plugin ya integra con Tauri); campo de texto para escribir la ruta (peor experiencia y más errores).

## D-29 — `tauri-plugin-autostart` para «Iniciar con Windows»
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** F6-05 pide iniciar con Windows, desactivado por defecto.
- **Decisión:** `tauri-plugin-autostart 2.7.0`, usado solo desde Rust (`autostart_get` / `autostart_set`), por lo que no necesita permisos de frontend. El estado vive en el registro de Windows (clave `Run` del usuario), no en `settings`; la app nunca se registra sin que el usuario lo pida. Se consulta al abrir Ajustes, no al arrancar (N-01).
- **Alternativas descartadas:** escribir la clave de registro con `winreg` (código `unsafe`-adyacente y propio para algo que el plugin resuelve).

## D-30 — Restaurar con la API de copia de SQLite y feature `backup` de `rusqlite`
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** D-09 exige validar con `integrity_check` antes de sustituir y conservar la base anterior. Con la base abierta en WAL no se puede reemplazar el archivo sin cerrar las conexiones.
- **Decisión:** `rusqlite` con la feature `backup` (misma versión; sin crate nueva). `restore` (1) valida el archivo en solo lectura: cabecera SQLite, `integrity_check`, `foreign_key_check`, `user_version` entre 1 y la actual, tablas de la aplicación y mismo `page_size`; (2) guarda la base actual en `agenda.before-restore.sqlite` con `VACUUM INTO`; (3) copia las páginas a la conexión escritora con la API de backup de SQLite (transaccional: si falla, quedan los datos actuales); (4) migra si la copia es anterior y reaplica los pragmas. Una copia de un esquema más nuevo se rechaza. `backup_create` escribe en un `.tmp` y renombra, y nunca sobrescribe la base en uso ni sus `-wal`/`-shm`.
- **Alternativas descartadas:** reemplazar el archivo `.sqlite` (obliga a cerrar y reabrir el estado compartido y es vulnerable a cortes); copiar el archivo en caliente (D-09).

## D-31 — WebdriverIO para las pruebas E2E y `MIAGENDA_DATA_DIR`
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** F6-06 pide E2E con WebdriverIO + `tauri-driver`, que solo corren en Windows.
- **Decisión:** `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework` y `@wdio/spec-reporter` `10.2.0` (solo dev, versión exacta). Los tests viven en `e2e/` (no entran en `npm run test`). Para no tocar nunca datos reales, la aplicación acepta la variable de entorno `MIAGENDA_DATA_DIR` como carpeta de datos alternativa; la configuración de E2E crea una carpeta temporal por sesión. `tauri-driver` y `msedgedriver` se instalan aparte (ver `e2e/README.md`).
- **Consecuencias:** la variable también la podría poner un usuario; solo cambia dónde se guarda la base y no abre ningún acceso nuevo.
- **Alternativas descartadas:** Playwright (no se integra con `tauri-driver`); exigir `APPDATA` temporal (Tauri usa la API de carpetas conocidas de Windows y la ignora).

---

## D-32 — Recordatorios periódicos (migración 002)
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** petición del usuario tras F6: recordatorios que se repitan como en Google Calendar (cada N unidades, un número de veces o hasta una fecha) y que los ya sonados no desaparezcan. D-07 deja fuera la recurrencia **de tareas** (`tasks.rrule`, que sigue sin usarse); esto afecta solo a los avisos.
- **Decisión:** migración `002_reminder_repeat.sql` con `repeat_every`, `repeat_unit` (`hour|day|week|month`), `repeat_count` (2–1000), `repeat_until` (fecha local, inclusive) y `occurrence`. Solo se repiten los avisos de **hora concreta**; los «X antes de la tarea» siguen a la fecha de la tarea. Terminar por número **o** por fecha, nunca ambos. Al sonar, ceder o caducar un aviso de una serie se queda como historial (`is_sent = 1`) y, **en la misma transacción**, se crea el siguiente con `occurrence + 1` (si el aviso falla, el rollback deshace también el siguiente). Si la app estuvo cerrada, los avisos perdidos se saltan pero cuentan para el número de veces. Días, semanas y meses conservan la hora local (DST); el día 31 se ajusta al último día del mes, sin recuperar el 31 en los meses siguientes. El frontend muestra los avisos ya sonados en gris, bajo «Ya avisados» (los últimos 5).
- **Consecuencias:** `Reminder` gana `repeat` en el contrato IPC (fixture y tipos actualizados). Una copia de seguridad anterior a esta versión se migra al restaurarse.
- **Alternativas descartadas:** RRULE completa (más superficie y UI de la necesaria); repetir también los avisos relativos (cambian cada vez que se mueve la tarea); generar de antemano todas las repeticiones (series sin fin).

---

## D-33 — Duración de las tareas con hora (migración 003) y vista de detalle
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** petición del usuario: poder fijar el fin de una tarea (o su duración) y que arrastrar un tramo en el calendario lo guarde. Resuelve B-04 con la opción 2.
- **Decisión:** migración `003_task_duration.sql` con `tasks.duration_minutes` (1–1440, nullable). El fin no se guarda: es `due_time` + duración y **siempre dentro del mismo día** (validación `validate_duration`; no hay tareas que crucen medianoche). Sin hora no hay duración: quitar la hora o la fecha la borra, y cambiar la hora la conserva mientras quepa en el día. Arrastrar un tramo de más de un paso (15 min) guarda su longitud; un clic simple solo elige la hora de inicio. Sin duración un bloque ocupa 30 min visuales. `reminders_set` rechaza (`VALIDATION`) añadir avisos a una tarea hecha; los pendientes se conservan y vuelven a valer al reabrirla (el planificador ya ignoraba las tareas hechas). El diálogo de «Nueva tarea» sirve también para editar (sin descripción: las notas con formato se editan en la vista de detalle) y desde el calendario una tarea abre su vista de detalle (hecha/reabrir, editar, eliminar con botón y Supr).
- **Consecuencias:** `Task` gana `durationMinutes` en el contrato IPC (fixture y tipos actualizados). Copias de seguridad anteriores se migran al restaurarse.
- **Alternativas descartadas:** `due_end_time` (permite fin anterior al inicio y obliga a mantener dos campos coherentes); tareas de varios días (bloques partidos en el calendario, recordatorios ambiguos).

---

## D-34 — Etiquetas de tareas: una por tarea, filtro en barra lateral
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** petición del usuario: etiquetas con color en la lista y el calendario, y un menú lateral para filtrar por ellas. El backend (tablas `tags` y `task_tags`, comandos `tag_*` y `task_tags_set`) existía desde F1; faltaba toda la interfaz. **No hay migración.**
- **Decisión:** la interfaz admite **una** etiqueta por tarea (la primera de `tagIds`); el modelo y el contrato IPC siguen admitiendo varias. El color es `#RRGGBB`: 8 predefinidos (`TAG_PRESETS`), selector nativo y campo de texto (acepta `#RGB` y sin `#`, se normaliza a mayúsculas). Los chips usan el color al 20 % de fondo y el texto mezclado con `--text-primary`, así se leen en tema claro y oscuro sin calcular contraste a mano. En el calendario el evento toma el color de la etiqueta (fondo y borde izquierdo) y muestra su nombre en pequeño. La barra lateral (224 px, plegable; empieza plegada por debajo de 900 px de ancho de ventana) tiene una casilla por etiqueta, **siempre todas y en el orden del backend**, con su contador de tareas pendientes (0 si no hay), y una entrada fija «Sin etiqueta». Incluye «Todas», «Ninguna», «Solo» por fila y crear, editar y eliminar etiquetas (eliminar pide confirmación; las tareas se quedan sin etiqueta). Un único filtro sirve a la lista y al calendario. El filtro **no se persiste**: al arrancar se ve todo, para que nada parezca perdido. Si se guarda una tarea que el filtro oculta, se avisa. Abrir una tarea oculta desde el calendario en la lista restablece el filtro.
- **Asignar etiqueta (ampliación):** además del diálogo, desde la fila (menú al pulsar el chip o el icono de etiqueta) y arrastrando la tarea, o un evento del calendario, sobre una etiqueta de la barra lateral. El arrastre usa eventos de puntero propios (`tagDrag.svelte.ts`) y no HTML5 drag and drop, porque la ventana de Tauri intercepta ese para soltar archivos; así no hace falta tocar `tauri.conf.json`.
- **Consecuencias:** `TasksStore` gana `filter`, `visibleTasks` y `allFiltered`, y `setTag`; `createDetailed` y `saveDetails` aceptan `tagId`. Etiquetas anidadas (`parentId`) no tienen interfaz.
- **Alternativas descartadas:** varias etiquetas por tarea (el color del evento sería ambiguo); persistir el filtro en `settings` (exigiría ampliar la lista blanca de valores del backend y arranca con tareas ocultas); un filtro distinto para cada vista (confunde).

---

## D-35 — Hora por defecto de los avisos y avisos de una semana
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** petición del usuario para la 1.0.1. Sustituye la alternativa descartada de D-26 («asumir una hora por defecto»), que entonces no estaba pedida. **No hay migración.**
- **Decisión:** ajuste `default_notify_time` (`HH:MM`, 09:00 si no existe) en Ajustes. Una tarea con fecha pero sin hora solo admite avisos de **días o semanas antes** (múltiplos de 1440 min; «El mismo día» = 0) y suena a esa hora por defecto del día resultante; los avisos en minutos u horas se rechazan (`VALIDATION:`). La hora supuesta solo sirve para calcular `notify_at`: la tarea sigue sin hora y en el calendario permanece en «Todo el día». Nuevo preset de una semana antes (10080 min, también con hora). Al cambiar el ajuste se recalculan los avisos relativos pendientes de las tareas sin hora.
- **Consecuencias:** `validate_untimed_offset` en `domain/reminders.rs`, `repo::reminders::recalculate_untimed`; la lista blanca de `settings` admite la nueva clave.
- **Alternativas descartadas:** guardar la hora supuesta en la tarea (la colocaría en el calendario); permitir avisos en horas sin hora de tarea (sin sentido).

---

## D-36 — Tema claro opaco
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** en el tema claro Mica/Acrílico dejaban ver el escritorio oscuro bajo una capa clara semitransparente y el contraste era malo (el oscuro sí se ve bien).
- **Decisión:** con el tema claro (elegido o resuelto desde el sistema) la ventana se pide sin material (`none`) y los fondos `--bg-window*` y `--bg-elevated` son opacos; el borde sutil sube a 12 %. El ajuste de efecto se conserva y vuelve a aplicarse en oscuro; Ajustes lo indica. Con «Sistema» se reaplica al cambiar `prefers-color-scheme`.
- **Alternativas descartadas:** subir solo la opacidad de la capa (el material sigue asomando en las esquinas redondeadas); confiar en la variante clara de Mica (no es fiable según el equipo).

---

## D-37 — Casillas (subtareas) en las notas de las tareas
- **Estado:** ACCEPTED · **Fecha:** 2026-11-09
- **Contexto:** petición del usuario: poder poner casillas dentro de la descripción de una tarea y marcarlas sin editar el texto. **No hay migración ni dependencia nueva.**
- **Decisión:** el editor de notas añade `TaskList` y `TaskItem` (sin anidar) de `@tiptap/extension-list 3.33.0`, que ya estaba instalado (D-22), y un botón «☑» en la barra. Marcar una casilla es un clic sobre ella: no hace falta enfocar ni escribir, y se guarda con el mismo guardado diferido de las notas. El documento sigue siendo ProseMirror JSON (`taskList`/`taskItem` con `attrs.checked`); el backend ya lo admite y el texto de búsqueda contiene el texto de cada casilla en su línea.
- **Consecuencias:** las casillas marcadas se muestran tachadas y en gris secundario. Marcarlas no completa la tarea ni cambia su estado.
- **Alternativas descartadas:** subtareas como entidad propia (migración y contrato IPC nuevos, fuera del alcance de una 1.0.x); casillas anidadas.

---

## D-38 — Notas desplegadas por defecto y recordadas
- **Estado:** ACCEPTED · **Fecha:** 2026-11-11
- **Contexto:** tras reiniciar, las notas de las tareas aparecían ocultas y había que pulsar el icono en cada una. Solo podía haber una abierta a la vez. **Sin migración ni dependencias nuevas.**
- **Decisión:** las notas con texto se muestran por defecto; ocultarlas con el icono se recuerda en `localStorage` (`agenda.notesCollapsed`, solo ids). Pueden estar abiertas varias a la vez. Las tareas sin notas siguen cerradas hasta que se pulsa el icono. Un botón «Ocultar/Mostrar todas las notas» aparece cuando hay más de una tarea con notas. Elegir una tarea desde la búsqueda la despliega.
- **Consecuencias:** al completar, cambiar la fecha o editar una tarea, sus notas se ocultan un instante para que el editor guarde antes de remontarse, y vuelven solas.
- **Alternativas descartadas:** guardar el estado en SQLite (migración para un detalle de interfaz); mostrar siempre todas sin poder ocultarlas.

---

## D-39 — Rediseño de la interfaz: estética y movimiento tipo Apple
- **Estado:** ACCEPTED (petición del usuario) · **Fecha:** 2026-11-11 · **Validación visual:** pendiente `[H]`
- **Contexto:** el usuario pide una interfaz moderna, cercana a las guías de diseño de Apple, con respuesta inmediata y animaciones rápidas y fluidas, sin perder ligereza en RAM ni fluidez. **Sin dependencias, migraciones, comandos IPC ni componentes nuevos**; la lógica y los textos no cambian.
- **Decisión:**
  - **Barra unificada.** `TitleBar` solo contiene los botones de ventana, flotando arriba a la derecha (32 px, por encima de los fondos de diálogo). La barra de herramientas de la página y la cabecera de la barra lateral (48 px) son la zona de arrastre (`data-tauri-drag-region`); la barra lateral ocupa toda la altura con un tinte propio.
  - **Tokens nuevos** en `tokens.css` (en los tres bloques de tema): `--bg-popover`, `--bg-sidebar`, `--bg-control`, `--bg-control-hover`, `--bg-active`, `--thumb`, `--separator`, `--backdrop`, `--shadow-raised`, `--shadow-popover`, `--shadow-modal`; radios `--radius-small/card/panel`; `--font-display` para títulos y `--font-ui` con el corte de texto primero. Los colores de D-27 no cambian y el test de contraste sigue en verde.
  - **Movimiento:** `--motion-instant` (90 ms, hover y pulsación), `--motion-fast` (150), `--motion-base` (260), `--motion-spring` (420) y dos muelles amortiguados muestreados en `linear()` (`--ease-spring`, `--ease-bounce`). Solo se animan `transform` y `opacity` (más el ancho de la barra lateral, cuyo contenido tiene ancho fijo). Todo es CSS: ni JavaScript de animación ni `backdrop-filter` (D-01 sigue vigente). `prefers-reduced-motion` pone todas las duraciones a 0.
  - **Controles compartidos** en `base.css` (`.btn`, `.icon-btn`, `.toggle-chip`, `.ui-form`) en lugar de una copia por componente; todos responden a la pulsación con una escala breve.
  - **Componentes:** control segmentado con pastilla deslizante; casilla que dibuja la marca; diálogos y búsqueda con entrada de muelle y superficie opaca; resultado activo de la búsqueda en color de acento; aviso (`Toast`) en cápsula centrada abajo; títulos de sección grandes en tipo oración (el texto sigue siendo «POR HACER» / «AGENDA», se transforma con CSS); ajustes en tarjetas con interruptor; calendario dentro de una tarjeta con esquinas redondeadas; filtro de etiquetas con punto de color relleno/hueco.
- **Discrepancias con `docs/SPEC.md` §8** (la especificación no se edita): §8.1 «zona de arrastre solo en titlebar» (ahora barra de herramientas y cabecera lateral); §8.3 orden de `--font-ui` y tokens añadidos; §8.4 duración base única de 150 ms (ahora cuatro duraciones y muelles).
- **Consecuencias:** no hay animaciones de salida (los diálogos y menús se cierran al instante): exigirían `svelte/transition`, que en jsdom rompe las pruebas de componentes, y un cierre inmediato es coherente con el objetivo de respuesta rápida. El CSS crece hasta 12,3 KB gzip; el JS queda en 187,9 KB (presupuesto 250). La apariencia, el arrastre de la ventana y la fluidez reales son `[H]` (manual-pruebas § «Rediseño de interfaz»).
- **Alternativas descartadas:** `backdrop-filter` para materiales translúcidos (contradice D-01 y cuesta GPU); una librería de animación o de componentes (peso y dependencia); transiciones de Svelte para las salidas (ver consecuencias).

---

## D-40 — Notas de las tareas: segunda línea discreta, sin foco al abrir
- **Estado:** ACCEPTED (petición del usuario) · **Fecha:** 2026-11-11 · **Validación visual:** pendiente `[H]`
- **Contexto:** con las notas desplegadas por defecto (D-38), al abrir la app el cursor aparecía dentro de la nota de una tarea, porque cada editor tomaba el foco al montarse; además todas mostraban su barra de formato. El usuario pide quitar el botón «Ocultar/Mostrar todas las notas» y una experiencia lo más parecida posible a la de Apple.
- **Decisión:**
  - Se elimina el botón global y su código (`setAllNotesOpen`, `notesCount`, `someNotesHidden`).
  - **Modifica D-38:** todas las notas con texto empiezan desplegadas en cada sesión. Ocultar una con su icono sigue siendo posible, pero ya no se recuerda al reiniciar; la clave `agenda.notesCollapsed` de `localStorage` se borra al arrancar.
  - Un editor solo toma el foco cuando el usuario acaba de abrir esa nota con su icono (`notesFocusId` en el store, de un solo uso). La barra de formato solo aparece con el foco dentro.
  - En la lista (`variant="inline"`) la nota es una segunda línea gris de 13 px alineada con el texto del título, sin borde lateral; al editarla pasa a texto primario con fondo de campo. La barra de formato se abre justo debajo del título de la tarea, encima del texto de la nota. En el detalle de la tarea (`variant="field"`) es un campo de formulario.
  - Texto de ayuda «Añadir nota…» en las notas vacías (`Placeholder` de `@tiptap/extensions`, paquete ya instalado por D-22: no es dependencia nueva). Una nota abierta para escribir y dejada vacía se recoge sola al salir de ella.
  - El aviso «Cargando editor…» solo se muestra si el módulo tarda más de medio segundo. Las casillas de las notas son redondas, como la de la tarea.
- **Discrepancia con `docs/SPEC.md` §8.7:** la nota ya no va sangrada 24 px con borde, sino alineada con el título.
- **Consecuencias:** al enfocar una nota, su texto y las tareas de debajo bajan la altura de la barra de formato. Cada nota visible sigue montando un editor TipTap.
- **Alternativas descartadas:** barra de formato flotante bajo la nota (se probó y el usuario la prefiere bajo el título); quitar el foco con un `blur` tras montar (parpadeo y lectores de pantalla confundidos).

---

## Decisiones pendientes de producto

Estas decisiones **no** pueden tomarse por el agente. Se siguen en `BLOCKERS.md`.

- **Acción "Marcar como hecha" en la notificación** para v1 o posterior.
- **Formato de exportación** además de `.sqlite` (p. ej. JSON).
