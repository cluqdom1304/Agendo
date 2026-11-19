# AUDITORIA.md — Plan de auditoría de Agendo

**Estado:** IN_PROGRESS — A0, A1 y el carril frontend (A6, A10, A12), A8 parcial, A9 y A2–A4 hechos `[A]`. Carril de datos (A2–A4) integrado. Pendientes `[H]`: A1-01, A12-03/04, A6-06.
**Objetivo:** aplicación ligera, bien estructurada, escalable, fácil de depurar y optimizar, y pulida para el usuario final.
**Reglas:** cada tarea sigue el ciclo de CLAUDE.md §2 (plan corto, cambio coherente, batería §4, commit convencional). Lo que dependa de Windows es `[H]` y nunca se marca como verificado. Cada dependencia o cambio de esquema lleva ADR en `DECISIONS.md`; las migraciones son siempre aditivas.

## Diagnóstico de partida

La base es sólida: reglas de CLAUDE.md §6 cumplidas, pragmas SQLite correctos, consultas parametrizadas, listas virtualizadas, vistas con carga diferida, migraciones transaccionales. Lo pendiente es escalabilidad, estructura y observabilidad.

**Comprobado:**
- `tauri.conf.json:28` → `"csp": null`.
- `tasks_fts` / `notes_fts` con id `UNINDEXED`: cada sincronización borra con escaneo completo de la tabla FTS (autosave O(N)).
- `repo/tasks.rs` y `repo/notes.rs` sin `LIMIT`: todo viaja por IPC en cada carga.
- Solo `eprintln!` (10 sitios), sin log persistente; en la versión instalada no hay consola.

**Por verificar (reportado por muestreo, no confirmado):**
- Comandos `async` sin `run_blocking` (los conteos sugieren que casi todos sí lo usan).
- Tailwind prescindible (569 atributos `class`; solo se vio una clase utilitaria).
- Cobertura real de `prefers-reduced-motion`, duplicados de CSS, cuántas listas implementan estados vacíos a mano.
- Los "archivos gigantes" de Rust son en buena parte tests (`commands/tasks.rs`: ~540 de código y ~1000 de tests).

## Fases

Orden de ejecución: A0, A1, A2, A3, A5, A4, A6, A10, A7, A8, A9, A11, A12.

### A0 — Línea base (riesgo nulo)
- [x] A0-01 Medir y registrar en `docs/metricas.md`: tamaño de bundle y binario, arranque, `cargo test -- --report-time`, `npm run audit`.
- [x] A0-02 Ya existen `scale_tests` a 10 000 filas; el salto a 50 000 pasa a A2-05.

### A1 — Seguridad (riesgo bajo)
- [x] A1-01 (`[H]` pendiente, ver manual-pruebas) CSP real en `tauri.conf.json` (`default-src 'self'`, `img-src 'self' data: blob:`, `connect-src ipc: http://ipc.localhost`). `[H]` en Windows.
- [x] A1-02 Capabilities revisadas: todo se usa desde JS salvo `allow-set-effects`, que D-02/SPEC §8.1 mantienen a propósito; se deja como está (quitarlo exigiría un ADR).
- [x] A1-03 Sin cambios: `backup_create` ya recibe `protected_files` y `export_notes` solo escribe archivos `.md`/`.html` dentro de una carpeta (hallazgo inicial descartado).
- [x] A1-04 Los límites ya existían (`MAX_DEPTH`, `MAX_INLINE_DEPTH`, `MAX_XML_DEPTH`, `MAX_BLOCK_DEPTH`); añadido test hostil de anidamiento inline (énfasis, tachado, enlaces, imágenes, código). Sin límite de nodos por `.enex`: el tamaño total ya está acotado (MAX_ENEX_BYTES), se revisa en A4-02.

### A2 — Esquema, índices y FTS (riesgo medio, mayor impacto)
- [x] A2-01 FTS por rowid con mapa id→rowid (migración 012, D-62): `fts::sync` a 50 000 filas 21,9 ms → 44 µs.
- [x] A2-02 Pragmas de caché/mmap y `PRAGMA optimize` (D-63).
- [x] A2-03 Filtro de fechas por rango: 21 ms → 0,08 ms.
- [x] A2-04 Índices (migración 013): `reminders.task_id` 13,4 s → 0,1 s en agenda de 50 000 tareas; índices de papelera parciales. Descartados `is_completed`, `folders.position`, `attachments.created_at`. La 013 hace `DROP INDEX` de dos índices y los recrea parciales (no toca datos).
- [x] A2-05 `scale_tests` a 50 000 filas sin regresión a 10 000.

### A3 — Consultas y paginación (riesgo medio)
- [x] A3-01 Ventanas `limit`/`offset` en notas y papeleras e `ix_notes_order` (migración 014, D-64): ventana de 200 sobre 50 000 notas 75 ms → 0,3 ms. Tareas: sin ventana, se sirve entera por `tasks_list_all`.
- [x] A3-02 Contadores de etiquetas agregados: mejora pequeña (189 → 154 ms con 100 000 enlaces); un contador cacheado queda como deuda.
- [x] A3-03 Solo `prepare_cached` en `find.rs`; el constructor SQL común no se unificó (sin mejora medida, deuda de mantenimiento).
- [x] A3-04 `UPDATE … RETURNING` en `expire_before`.
- [x] A3-05 `tasks_list_all(tag_id?)` y `note_titles(ids)` (máx. 1000), con contrato. Falta el wrapper en `api.ts` (A6-01).

### A5 — Observabilidad (riesgo bajo)
- [x] A5-01 Registrador propio con `std` (sin dependencias): `agendo.log` con rotación en la carpeta de datos; sustituye los `eprintln!` (D-90).
- [x] A5-02 Trazas de consultas lentas y duración de migraciones en `startup.rs`.

### A4 — Concurrencia y memoria (riesgo medio)
- [x] A4-01 Pool de 3 lectores; copias sobre conexión propia (D-65).
- [x] A4-02 ENEX en streaming (límite 256 MiB por nota).
- [x] A4-03 `.agendo` en flujo a fichero temporal, máx. 200 000 entradas y 16 GiB.
- [x] A4-04 Una sola copia por imagen al exportar; `collect_orphans` en una transacción.
- [x] A4-05 Señal de cierre, backoff creciente y espera de hasta 5 s a una copia en curso (D-66).
- [x] A4-06 Descartado: los 54 comandos `async` ya usan `run_blocking`.
- [x] A4-07 Tope de 16 MiB por nota en versiones (ya había 30 versiones por nota).

### A6 — Frontend: rendimiento (riesgo bajo)
- [x] A6-01 `tasksListAll`: 1 IPC por refresco (antes 2), fusión por id con identidad estable y refrescos coalescidos; `noteTitles` en lote.
- [x] A6-02 Ids únicos como `$derived` (hecho). Carga en lote pendiente de A3-05 (`note_titles`).
- [x] A6-03 Solo era real el reloj del calendario (hecho); el resto ya existía (índice por día, `Map` de etiquetas, `slice` derivado).
- [x] A6-04 Descartado: `base.css:272` ya tiene la guardia global.
- [x] A6-05 `.catch` en los listeners; no había carrera.
- [ ] A6-06 Perfil del editor con notas grandes `[H]`.

### A10 — Dependencias y build (riesgo bajo)
- [x] A10-01 Los duplicados de `cargo tree -d` son transitivos (Tauri/`sha2`), no accionables; las features de `tokio` (`macros`, `rt`, `sync`, `time`) se usan todas.
- [x] A10-02 Ya decidido con datos en F16-03/F16-05: `s` + LTO fat ahorra 4,9 MB y la escala sigue ≪ límites; sin cambios. `panic = "abort"` descartado (D-24).
- [x] A10-03 Medido: Tailwind cuesta 2,56 KB gzip, el preflight es necesario. D-70 `ACCEPTED`: se mantiene.
- [x] A10-04 `npm run tokens` (9 de 67 sin uso; no se ha quitado nada).
- [x] A10-05 `[profile.dev.package."*"] opt-level = 2`: tests lib 10,5 s → 8,0 s (−24 %).

### A7 — Estructura Rust (riesgo medio-alto)
- [x] A7-01 Tests de `commands/tasks`, `notify` y `scheduler` en archivos propios.
- [x] A7-02 `domain/markdown` (write, read, inline, table) y `exchange` (export, import, catalog, paths) divididos.
- [x] A7-03 Saneado de nombres y rutas de Windows en un único módulo.
- [x] A7-04 Lógica de tareas, notas, recordatorios y etiquetas en `services/`; los comandos solo envuelven.
- [ ] A7-05 Validación unificada de DTO (`TryFrom<Input>`).
- [ ] A7-06 Código muerto (`fts::rebuild`) y envolturas CRUD repetidas.

### A8 — Estructura frontend (riesgo medio)
- [x] A8-01 `notes.svelte.ts` 836→731 y `tasks.svelte.ts` 805→580 líneas, en módulos compuestos con la API intacta.
- [x] A8-02 `+page.svelte` 596→461 (`shortcuts.ts`, `backendEvents.ts`, `dialogs.svelte.ts`, `DialogHost.svelte`).
- [x] A8-03 `editor.ts` 728→~520, `entries.ts` 574→60 (5 módulos), sugerencias de `NoteEditor` a `noteSuggestions.svelte.ts`.
- [ ] A8-04 (aplazado: sin Windows no se verifica el aspecto) `InlineEdit`, `EmptyState` y `CalendarChip` compartidos.
- [x] A8-05 Hecho `SettingsAbout` y `CalendarWeek` (602→479, `weekCompose`). Descartados `FolderSidebar`, `TaskDialog` (≈15 estados con `bind:` y CSS acoplado) y la carga por pestaña (no hay pestañas).
- [x] A8-06 Descartado: los dominios diferidos suman ~2,5 KB gzip (3 % del inicial) y exigiría reescribir `t.*` en ~60 archivos.

### A9 — Paridad front/back (riesgo bajo)
- [x] A9-01 Vectores en `src-tauri/fixtures/parity/*.json` leídos por `parity_tests.rs` y `parity.test.ts` (recurrencia, orden, fechas con DST, avisos, duración). Divergencia real corregida: `parseRule` (TS) aceptaba entradas que Rust rechaza.
- [x] A9-02 D-80 `ACCEPTED`: el backend es la fuente de verdad de recurrencia y orden.

### A11 — Tests (riesgo bajo)
- [x] A11-01 `makeNote`/`makeNoteHit`/`makeTaskHit`/`makeTagHit` compartidos; 2 tests duplicados eliminados; esperas fijas por `vi.waitFor`.
- [x] A11-02 Causa real: sobresuscripción (≥12 tests en paralelo en 16 hilos lógicos hacían la suite más lenta y los tests de tiempos fallaban). `src-tauri/.cargo/config.toml` fija `RUST_TEST_THREADS = 8`: 10,5 s → 4,7 s y 4/4 ejecuciones en verde. En TS, `asyncUtilTimeout` de 5 s para los `findBy*` del editor cargado con `import()`.
- [~] A11-03 e2e escritos (calendario, recordatorios, etiquetas, copias) pero no ejecutados: primera ejecución `[H]`. Paleta e importación siguen sin e2e.

### A12 — Pulido final
- [x] A12-01 `EmptyState` con variantes carga/error; corregido bug real: `TodoView` mostraba «Sin tareas» si la carga fallaba.
- [x] A12-02 Revisado, sin cambios; teclado `[H]`.
- [ ] A12-03 Instalador: cierre del proceso al actualizar y diff documentado contra la plantilla de Tauri `[H]`.
- [ ] A12-04 Lista `[H]` final para Windows 11 en `docs/manual-pruebas.md`.

## Criterio de cierre de cada fase
Batería de CLAUDE.md §4 en verde, métricas antes/después en `docs/metricas.md`, ADR donde corresponda, `Estado: AWAITING_HUMAN` y parada hasta confirmación.
