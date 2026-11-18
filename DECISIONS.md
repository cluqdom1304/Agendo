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

## D-41 — Etiqueta antes del título e instalador que reemplaza la versión anterior sin preguntar
- **Estado:** ACCEPTED (petición del usuario) · **Fecha:** 2026-11-12 · **Validación:** pendiente `[H]`
- **Decisión:** en la lista, la etiqueta de una tarea se muestra entre la casilla y el título; sin etiqueta no se reserva hueco y el botón de asignar sigue a la derecha, visible al pasar el ratón. El instalador NSIS usa una copia local de la plantilla oficial de `tauri-cli` 2.12.4 (`src-tauri/windows/installer.nsi`, clave `bundle.windows.nsis.template`) con dos cambios: si hay una versión instalada, se activa el modo pasivo (sin páginas ni confirmaciones) y se fuerza «desinstalar antes de instalar». Los datos de usuario se conservan (el desinstalador pasivo no los borra).
- **Consecuencias:** al subir de versión de `tauri-cli` conviene volver a copiar la plantilla y reaplicar los dos cambios (marcados con «MiAgendaApp»). El MSI ya se actualiza solo por su UpgradeCode. Sin dependencias nuevas.
- **Alternativas descartadas:** `/P` o `/UPDATE` por línea de comandos (el usuario abre el `.exe` con doble clic); `NSIS_HOOK_PREINSTALL` (se ejecuta después de la pantalla de confirmación).

## D-42 — Nombre oficial de la aplicación: Agendo
- **Estado:** ACCEPTED (petición del usuario) · **Fecha:** 2026-11-12 · **Validación:** pendiente `[H]`
- **Decisión:** el producto pasa a llamarse **Agendo** (`productName`, título de ventana, textos de interfaz, crate `agendo` / lib `agendo_lib`, binario `agendo.exe`, paquete npm). Los marcadores «MiAgendaApp» de D-41 en `installer.nsi` pasan a «Agendo».
- **Identificador:** pasa de `com.cristian.miagendaap` (heredado de F0-01, con una `p` menos) a `com.cristian.agendo`. Cambia la carpeta de datos (`%APPDATA%\com.cristian.agendo`).
- **Consecuencias:** Agendo arranca con una base vacía. Para conservar las notas: crear una copia en la versión antigua (Ajustes → «Crear copia…»), desinstalarla, instalar Agendo y usar «Restaurar copia…». La clave de desinstalación y la carpeta de instalación también cambian, así que el instalador no reemplaza una instalación previa de MiAgendaApp.
- **Alternativas descartadas:** mantener el identificador antiguo (arrastra el typo y el nombre viejo en la ruta de datos); migración automática de la carpeta antigua (código para un único usuario, la copia manual basta).

---

## D-43 — Carpetas de notas en árbol
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Contexto:** V2 añade notas estilo Evernote (Stack > Notebook). Un árbol genérico cubre ambos niveles.
- **Decisión:** tabla `folders` con `parent_id`, profundidad máxima 5, sin ciclos (validado en `domain`). Cada nota está en una carpeta o en la «Bandeja de entrada» (`folder_id` NULL).
- **Alternativas descartadas:** dos niveles fijos (rígido); una nota en varias carpetas (ambiguo; para eso están las etiquetas).

## D-44 — Varias etiquetas por tarea y por nota
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Contexto:** D-34 limitó la interfaz a una etiqueta por tarea; el modelo ya admite varias. Con notas, la etiqueta es el eje transversal.
- **Decisión:** una sola tabla `tags`; `note_tags` gemela de `task_tags`. La interfaz admite varias en ambas; el color del evento en el calendario usa la primera. Sustituye la parte de D-34 «una por tarea».
- **Alternativas descartadas:** tablas de etiquetas separadas para notas (incoherencia entre ambas).

## D-45 — Tareas en notas por referencia
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Decisión:** nodo `taskRef` (`attrs.taskId`) en el JSON de TipTap; la tarea vive solo en `tasks` y `tasks.source_note_id` (`ON DELETE SET NULL`) guarda su origen. Las casillas de D-37 se mantienen como checklist ligera con acción «Convertir en tareas».
- **Alternativas descartadas:** copiar la tarea dentro de la nota (dos fuentes de verdad); sustituir las casillas de D-37.

## D-46 — Enlaces entre notas
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Decisión:** sintaxis `[[título]]` resuelta a `note_id`; tabla `note_links` para backlinks, recalculada al guardar.

## D-47 — Adjuntos en disco y copia `.agendo`
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Decisión:** ficheros en `attachments/` nombrados por SHA-256 (deduplicados) y tabla `attachments`. La copia de seguridad pasa a `.agendo` (zip con la base de datos y los adjuntos); restaurar sigue aceptando `.sqlite` anteriores. Puede requerir una dependencia de zip (entrada propia antes de añadirla).
- **Alternativas descartadas:** imágenes en BLOB dentro de SQLite (infla la base y las copias).

## D-48 — Historial de versiones de notas
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Decisión:** instantáneas de `content_json` espaciadas en el tiempo, con máximo N por nota; restaurar crea una versión nueva.

## D-49 — Presupuesto del bundle en V2
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-12
- **Decisión:** se mantiene N-04 (250 KB gzip de JS). Notas y editor en carga diferida (D-10); si se supera, ADR nuevo antes de subir el límite. **Límite ampliado por D-56 (300 KB) para la 2.1.0.**

## D-50 — Menús contextuales propios y botón flotante de nueva tarea
- **Estado:** ACCEPTED (el usuario pidió continuar con la Fase 11B; «archivar» queda fuera) · **Fecha:** 2026-11-12
- **Decisión:** el clic derecho abre un menú dibujado por la aplicación (mismo componente y estética que los menús actuales), no un menú nativo de Windows. Cada elemento (tarea, nota, carpeta, etiqueta, enlace) y el fondo de cada sección ofrecen sus acciones; el menú del navegador solo se conserva en campos de texto. «Nueva tarea» pasa a un botón flotante en la esquina inferior derecha de la Agenda.
- **Motivo:** un menú nativo de Tauri no admite el aspecto ni las animaciones del resto de la interfaz, ni submenús con casillas (etiquetas), y su comportamiento en WebView2 no se puede comprobar aquí. El menú propio se prueba con jsdom.
- **Archivar:** no se incluye. No existe ese estado en el modelo (tareas ni notas) y exigiría una migración y una vista nueva; los equivalentes actuales son «hecha» (tareas) y «fijar» / papelera (notas). Si se quiere, ADR aparte.
- **Alternativas descartadas:** menú nativo de Tauri (`tauri::menu`); mantener el botón en la barra superior.

## D-51 — Dependencias de la Fase 12 (adjuntos y editor)
- **Estado:** ACCEPTED (consecuencia directa de D-47) · **Fecha:** 2026-11-12
- **Rust:** `sha2` 0.11 (SHA-256 de los adjuntos; la biblioteca estándar no lo trae) y `zip` 9.5 con solo la característica `deflate` (copia `.agendo`). Alternativas descartadas: un contenedor propio (riesgo de corrupción sin ventaja) y guardar las imágenes en la base (D-47).
- **Editor (se añadirán al usarlos, todas de la misma familia y versión 3.33.0 que las actuales):** `@tiptap/extension-table`, `-blockquote`, `-horizontal-rule`, `-highlight`, `-code-block` y `-heading`. Alternativas descartadas: `lowlight`/`highlight.js` (pesan demasiado para el presupuesto de 250 KB, D-49): el resaltado de código se hace con un tokenizador propio y pequeño; la imagen y la búsqueda dentro de la nota son nodos y complementos propios, sin dependencia.
- **Imágenes:** solo PNG, JPEG, GIF y WebP, identificadas por su firma y no por el nombre; SVG queda fuera (puede llevar scripts). Máximo 20 MB. Los bytes viajan como cuerpo binario de IPC, no como JSON.

## D-52 — Recordatorios de nota: tabla propia sobre el mismo motor
- **Estado:** ACCEPTED (propuesta del agente en F13-04; aceptada el 2026-11-16 al pedir el usuario cerrar lo pendiente de la V2; no contradice `docs/SPEC.md`) · **Fecha:** 2026-11-16
- **Contexto:** `reminders.task_id` es `NOT NULL REFERENCES tasks`. Admitir notas ahí exige reconstruir la tabla (crear, copiar, `DROP`, renombrar), y `CLAUDE.md` §7 prohíbe `DROP` sobre datos de usuario.
- **Decisión:** tabla `note_reminders` (migración 010; un aviso pendiente por nota, `ON DELETE CASCADE`). El motor es el mismo: `repo/reminders.rs` (`due`, `next_after`, `mark_sent`, `expire_before`) consulta las dos tablas y el planificador, los reintentos, la caducidad de 24 h y el despertar no cambian. `DueReminder` y `Fired` llevan `task_id` o `note_id`. Los avisos de nota no se repiten ni tienen antelación: una sola hora.
- **Alternativas descartadas:** reconstruir `reminders` (viola §7); tarea «sombra» por nota (ensucia listas, búsqueda y papelera); un segundo planificador (duplica el motor, lo que F13-04 prohíbe).
- **Consecuencias:** si se rechaza, solo hay que retirar la migración 010 y sus comandos; nada más depende de ella.

## D-53 — Datos de desarrollo separados de los de la aplicación instalada
- **Estado:** ACCEPTED (petición del usuario) · **Fecha:** 2026-11-16 · **Validación:** `[A]` con tests de `resolve_data_dir`; `[H]` abrir `tauri dev` y comprobar la carpeta
- **Contexto:** `tauri dev` y la app instalada compartían `%APPDATA%\com.cristian.agendo`. Ejecutar una versión en desarrollo migró la base de datos (esquema 5) y la 1.0.7 instalada dejó de abrir («schema version 5 is newer than the supported version 3»). Las migraciones solo avanzan.
- **Decisión:** `data_dir` (`src-tauri/src/lib.rs`) elige la carpeta así: 1) `MIAGENDA_DATA_DIR` si está definida (D-31); 2) en compilaciones de depuración (`cfg!(debug_assertions)`, es decir `tauri dev`, `cargo run`), `%APPDATA%\com.cristian.agendo.dev`; 3) en release, `%APPDATA%\com.cristian.agendo`. Las compilaciones release (instaladores y `tauri build`) nunca usan la carpeta `.dev`.
- **Consecuencias:** desarrollo empieza con una base vacía y propia; se puede borrar `com.cristian.agendo.dev` sin riesgo. Para probar con datos reales, restaurar una copia de seguridad en la app de desarrollo. El almacenamiento de la WebView no guarda datos de usuario (solo se limpia una clave heredada), por lo que no se separa. **Regla:** nunca hacer que una compilación de depuración abra la carpeta real.
- **Alternativas descartadas:** identificador distinto para desarrollo (`tauri.dev.conf.json`: duplica configuración y cambia permisos y avisos); solo variable de entorno (se olvida y vuelve el problema); copiar la base real a desarrollo al arrancar (acopla ambas).

## D-54 — Formatos de intercambio de la Fase 14 (Markdown, HTML, JSON, ENEX)
- **Estado:** ACCEPTED (detalle técnico de F14 dentro del plan aprobado el 2026-11-12; revisable) · **Fecha:** 2026-11-16 · **Validación:** `[A]` con tests de ida y vuelta; `[H]` abrir los ficheros en otros programas y probar un `.enex` real
- **Contexto:** F14 pide exportar a Markdown/HTML, volcar todo a JSON (B-02 se resolvió «JSON en v2»), importar carpetas de Markdown/`.txt` y exportaciones de Evernote. Ningún formato lo fija `docs/SPEC.md`.
- **Decisión:**
  - **Sin dependencias nuevas.** El lector XML (ENEX y ENML), base64 y MD5 (Evernote casa cada `<en-media>` con su recurso por el MD5 de los bytes; solo se usa para emparejar) son propios y pequeños: `domain/enex.rs`, `domain/codec.rs`. Alternativas descartadas: `quick-xml`, `base64`, `md-5`, `pulldown-cmark` (cuatro dependencias para un uso acotado, con el presupuesto de binario y la superficie de ataque en contra).
  - **Markdown:** un `.md` por nota (carpetas de la aplicación = carpetas del disco, nombres saneados y sin repetir) con front matter `title`, `tags`, `created`, `updated`, un `# título` y el cuerpo. Las imágenes se copian a `adjuntos/<sha256>.<ext>` y se enlazan con rutas relativas. Al importar se leen esos mismos campos (también `date`/`modified` y etiquetas en lista, como Obsidian); un primer `# H1` pasa a ser el título.
  - **HTML:** una página autocontenida por nota (imágenes en `data:`, sin scripts ni recursos remotos, texto siempre escapado, solo enlaces `http`/`https`/`mailto`).
  - **JSON:** un solo archivo con carpetas, etiquetas, notas (documento como JSON, no como texto), tareas con recordatorios, plantillas y la lista de adjuntos (**solo metadatos**: las imágenes van en la copia `.agendo` o en la exportación Markdown). Incluye la papelera (`deletedAt`). No hay importación de este JSON: la restauración es la copia `.agendo` (F12-05).
  - **ENEX:** cada archivo es una carpeta con el nombre del archivo (el formato no trae el cuaderno; Evernote nombra así el archivo). Se importan título, fechas, etiquetas, texto con formato, listas, casillas (`en-todo` → lista de tareas), tablas, citas, código e imágenes. Se admiten varios archivos a la vez.
  - **Seguridad de la importación:** no se sigue ningún enlace simbólico ni se leen ficheros ocultos; una imagen solo se toma si está dentro de la carpeta elegida (`..` y rutas absolutas se rechazan) y no se descarga nada de la red; límites de 10 MB por texto, 20 MB por imagen, 800 MB por `.enex`, 50 000 archivos y anidamiento acotado en Markdown, XML y bloques (sin desbordar la pila con entradas hostiles). Los enlaces siguen la regla `http`/`https`/`mailto`.
- **Limitaciones conocidas:** el tachado (`~~`) y el subrayado se pierden (el editor no tiene esas marcas); las referencias `taskRef`/`noteLink` se exportan como texto («Tarea: …», `[[título]]`) y no se reconstruyen al importar; los adjuntos que no son imágenes (PDF, audio) de un `.enex` no se importan y se cuentan como omitidos; importar dos veces la misma carpeta duplica las notas (no hay detección de duplicados); las notas importadas conservan sus fechas originales.
- **Consecuencias:** no cambia el esquema ni hay migración. Quitar F14 solo exige retirar `exchange.rs`, `commands/exchange.rs`, los módulos de `domain/` y `repo/export.rs`, más la sección de Ajustes y las entradas de menú.

## D-55 — Repaso de diseño previo a la 2.0: una sola línea visual en lo añadido en la V2
- **Estado:** ACCEPTED (petición del usuario) · **Fecha:** 2026-11-16 · **Validación visual:** pendiente `[H]`
- **Contexto:** lo añadido en las fases 8 a 14 (notas, «Hoy», historial, plantillas, menús) se escribió después de D-39 y parte no seguía sus tokens: variables inexistentes (`--radius-sm`, `--surface-hover`, `--space-5`), colores fijos, botones sin la tipografía de la app y dos estilos de menú distintos. El usuario pide pulirlo con la línea de Apple de D-39, sin generar aún la versión.
- **Decisión (amplía D-39; solo CSS y marcado, sin lógica, dependencias, IPC ni textos nuevos):**
  - **Tokens nuevos** en los tres bloques de tema: `--highlight`, `--highlight-strong`, `--on-highlight` (resaltado y búsqueda en la nota), `--code-string/number/keyword/type` (colores del código) y `--diff-added/removed` (historial). Sustituyen a colores fijos y a un `@media (prefers-color-scheme)` que ignoraba el tema forzado en Ajustes.
  - **Menús:** `Menu` y `TagMenu` resaltan la entrada bajo el puntero en color de acento, igual que el menú contextual (D-50) y la paleta; la marca de selección es un trazo, no el carácter «✓».
  - **`Modal` admite `wide`** (760 px) para contenido a dos columnas; lo usa el historial, que antes se recortaba dentro del diálogo de 520 px.
  - **«Hoy»:** fecha sobre un título grande y bloques en tarjetas agrupadas con filas separadas por una línea fina (como Ajustes); la nota del día es una fila con icono y flecha; el aviso de día vacío va dentro de su tarjeta.
  - **Notas:** historial con la versión elegida teñida de acento y las diferencias en una tarjeta; plantillas en tarjeta con acciones secundarias discretas; enlaces entrantes y tareas de la nota con filas que responden al puntero; cita, regla y tablas con línea más visible; buscador de la nota y campo de enlace con el mismo anillo de foco que los formularios.
  - **General:** selección de texto en el tinte de acento; cierre del aviso (`Toast`) dibujado en vez del carácter «×».
- **Consecuencias:** CSS 18,6 KB gzip; JS 246,4 KB de 250 (**quedan 3,6 KB para la Fase 15**). La apariencia real es `[H]` (manual-pruebas § «Repaso de diseño previo a la 2.0»).
- **Alternativas descartadas:** animaciones de salida y `backdrop-filter` (siguen descartadas por D-39 y D-01); rehacer los componentes de notas ya repasados (lista, panel y barra lateral ya seguían D-39).

## D-56 — Presupuestos de la 2.1.0 y descarte de la sincronización con Google
- **Estado:** ACCEPTED (aprobado por el usuario) · **Fecha:** 2026-11-18
- **Contexto:** el JS gzip pasó de 173,1 KB (1.0.0) a 246,4 KB (2.0.0) y el ejecutable de 12,06 MiB a 14,65 MiB (≈ 15,36 MB decimales, ya por encima de los 15 MB si se cuenta en MB). D-49 exige un ADR antes de subir N-04. El usuario pidió ampliar los límites a algo razonable sin perder la filosofía de aplicación ligera, rápida y eficiente.
- **Decisión (no se edita `docs/SPEC.md`; este ADR prevalece sobre los valores de N-03 y N-04 de SPEC §2, CLAUDE.md §1):**
  - **N-04 (JS gzip):** 250 → **300 KB**. Objetivo de cierre de la 2.1.0: **≤ 285 KB** tras la carga diferida de la Fase 16; el margen restante se reserva para la 2.2.
  - **N-03 (ejecutable release):** 15 → **18 MB** (medido en MB decimales, igual que el resto). Antes de gastarlo, F16-03 intenta recuperar tamaño con el perfil de compilación.
  - **N-01 (arranque < 200 ms) y N-02 (RAM < 40 MB):** **sin cambios**. Nunca se han medido; se miden en F16-06/F16-09 y, si el valor real lo exige, un ADR posterior los ajusta con datos.
  - **Regla:** ninguna función nueva entra en la carga inicial si puede cargarse al usarse (D-10); cada fase anota en la tabla de Métricas lo que consume.
  - **Aplicación:** el script `npm run size` y los valores de la tabla de Métricas se actualizan en F16-08 (código; no se toca antes de empezar la 2.1.0). Mientras tanto la batería sigue usando el límite actual.
- **Sincronización con Google (Calendar y Tasks): descartada por el usuario** (2026-11-18); no se planifica en ninguna versión. Motivos recogidos en la conversación: contradice «sin cuentas y sin red en tiempo de ejecución», exigiría una entidad de *evento* distinta de la tarea, un cliente HTTP/TLS (peso), OAuth de escritorio con un proyecto propio en Google Cloud y guardar tokens. Solo se reconsidera si el usuario lo pide expresamente, con un ADR nuevo.
- **Alternativas descartadas:** subir N-04 a 350 KB o más (quita la presión que mantiene la app ligera); mantener 250/15 sin cambios (obligaría a retirar funciones o a recortar la 2.1.0).

---

## Decisiones pendientes de producto

Estas decisiones **no** pueden tomarse por el agente. Se siguen en `BLOCKERS.md`.

- **Acción "Marcar como hecha" en la notificación** para v1 o posterior.
- **Formato de exportación** además de `.sqlite` (p. ej. JSON).

## D-57 — Recurrencia ampliada de tareas: días de la semana y «N veces» (F17-01)
- **Estado:** ACCEPTED (dentro del plan de la 2.1.0 aprobado; revisable) · **Fecha:** 2026-11-18 · **Validación:** `[A]` con tests de dominio, de comando y de interfaz; `[H]` uso en Windows 11 (F17-10)
- **Contexto:** F13-05 ya admite `FREQ`, `INTERVAL` y `UNTIL` en `tasks.rrule` (el «cada N» y el «hasta fecha» existían). Faltaban elegir días de la semana y terminar tras N repeticiones. D-32 descartó RRULE completa y esa decisión se mantiene.
- **Decisión:** dos partes más del mismo subconjunto de RFC 5545, sin migración (la columna es texto):
  - `BYDAY=MO,TU,…` solo con `FREQ=WEEKLY`. Las semanas empiezan en lunes y `INTERVAL` cuenta semanas desde la de la tarea actual, de modo que completar tarde no descuadra el ciclo. Sin `BYDAY` se conserva el comportamiento anterior.
  - `COUNT=n` (1–999) son las ocurrencias que quedan **contando la tarea que lleva la regla**; al completarla, la siguiente nace con `n-1` y con `COUNT=1` la serie termina. Completar tarde no consume repeticiones. `UNTIL` y `COUNT` son excluyentes (como en el RFC).
- **Consecuencias:** sin dependencias ni cambios de esquema ni de contrato IPC (`rrule` sigue siendo `string`). Las reglas guardadas antes se leen igual. La interfaz ofrece siete botones de día y un campo «Número de veces», que se vacían entre sí con «Hasta».
- **Alternativas descartadas:** RRULE completa (`BYSETPOS`, `BYMONTH` libre, `EXDATE`…); guardar la fecha de fin calculada en lugar de un contador (cambiaría al mover la tarea).

## D-58 — Tachado y subrayado en el editor (F17-04)
- **Estado:** ACCEPTED (dentro del plan de la 2.1.0) · **Fecha:** 2026-11-18 · **Validación:** `[A]` tests de ida y vuelta; `[H]` abrir los `.md`/`.html` exportados en otros programas
- **Decisión:** dos marcas nuevas, `strike` y `underline`, en el editor de notas y en el de tareas.
  - **Dependencias** (CLAUDE.md §6): `@tiptap/extension-strike` y `@tiptap/extension-underline`, ambas `3.33.0` como el resto de TipTap. Alternativa descartada: escribir las marcas a mano (más código propio que mantener para algo que la librería ya hace con sus atajos Ctrl+U y Ctrl+Mayús+S).
  - **Markdown:** tachado `~~texto~~` (estándar de GFM, que el lector ya aceptaba y descartaba) y subrayado `<u>texto</u>` (Markdown no tiene sintaxis propia). Un `<u>` escrito como texto se escapa y no cambia de significado.
  - **HTML:** `<s>` y `<u>`. **ENEX:** `<s>`, `<strike>`, `<del>`, `<u>`, `<ins>` y los estilos `text-decoration: line-through | underline`.
- **Consecuencias:** cierra la limitación de D-54 sobre el tachado y el subrayado. Sin cambio de esquema ni de IPC (`content_json` ya guarda marcas arbitrarias). JS +0,6 KB gzip.
