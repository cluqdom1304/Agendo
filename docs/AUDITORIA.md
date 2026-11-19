# AUDITORIA.md — Plan de auditoría de Agendo

**Estado:** IN_PROGRESS — A0, A1 y el carril frontend (A6, A10, A12) hechos `[A]`. Carril de datos (A2–A4) en curso en su rama. Pendientes `[H]`: A1-01, A12-03/04, A6-06.
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
- [ ] A2-01 Migración nueva: FTS sin escaneo O(N) al sincronizar (external-content o mapeo id→rowid) con reindexado. ADR.
- [ ] A2-02 Pragmas: `cache_size`, `mmap_size`, `journal_size_limit`, `wal_autocheckpoint`, `PRAGMA optimize` tras migrar y al cerrar, `ANALYZE` tras importaciones grandes.
- [ ] A2-03 Filtro de fechas por rango sobre la columna (sin `substr`) para usar `ix_notes_updated`.
- [ ] A2-04 Revisar índices (`reminders.task_id`, `tasks.is_completed`, `notes.pinned`, `folders.position`, `attachments.created_at`) con `EXPLAIN QUERY PLAN` y tests de plan.
- [ ] A2-05 `scale_tests` a 50k filas con presupuestos de tiempo.

### A3 — Consultas y paginación (riesgo medio)
- [ ] A3-01 Ventana o paginación en `list` de tareas, notas y papelera.
- [ ] A3-02 Contadores de etiquetas con una sola agregación (sin COUNT correlacionados).
- [ ] A3-03 `find.rs`: constructor SQL común para tareas y notas, `prepare_cached`, sin duplicación FTS/no-FTS.
- [ ] A3-04 `reminders::expire_before` con `UPDATE ... RETURNING`; eliminar `format!` de nombres de tabla.
- [ ] A3-05 Comandos IPC `tasks_list_all` y `note_titles(ids)`.

### A5 — Observabilidad (riesgo bajo)
- [ ] A5-01 Log a fichero con rotación en la carpeta de datos (ADR por dependencia); sustituir los `eprintln!`.
- [ ] A5-02 Trazas de consultas lentas (umbral) y duración de migraciones en `startup.rs`.

### A4 — Concurrencia y memoria (riesgo medio)
- [ ] A4-01 Pool de 2–3 lectores en WAL, o conexión propia para copias, para que una copia o búsqueda larga no bloquee el resto.
- [ ] A4-02 Importación ENEX en streaming y límite de tamaño revisado.
- [ ] A4-03 Restauración de copias: escritura a fichero temporal, límite de entradas y de tamaño total acumulado.
- [ ] A4-04 Adjuntos: evitar copias dobles y lecturas/hashes repetidos en exportación; `collect_orphans` por lotes.
- [ ] A4-05 Apagado ordenado y backoff en `autobackup` y en el tick del scheduler.
- [ ] A4-06 Verificar uno a uno los comandos `async` sin `run_blocking`.
- [ ] A4-07 Política de poda de `note_versions`.

### A6 — Frontend: rendimiento (riesgo bajo)
- [ ] A6-01 (espera a A3-05) `refresh` de tareas con fusión por id (conserva identidad) y debounce; una sola llamada IPC.
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
- [ ] A7-01 Mover tests grandes a archivos propios (`commands/tasks`, `notify`, `scheduler`).
- [ ] A7-02 Dividir `domain/markdown.rs` (read, write, inline, table) y `exchange.rs` (export, import, catalog, paths).
- [ ] A7-03 Un único módulo de saneado de rutas y nombres de Windows.
- [ ] A7-04 Extraer lógica de negocio de `commands/tasks`, `notes`, `reminders` y `tags` a servicios; los comandos solo deserializan y llaman a `run_blocking`.
- [ ] A7-05 Validación unificada de DTO (`TryFrom<Input>`).
- [ ] A7-06 Código muerto (`fts::rebuild`) y envolturas CRUD repetidas.

### A8 — Estructura frontend (riesgo medio)
- [~] A8-01 `notes.svelte.ts` 836→731 líneas (enlaces, plantillas, versiones, recordatorios en módulos). `tasks.svelte.ts` pendiente: se divide junto a A6-01.
- [x] A8-02 `+page.svelte` 596→461 (`shortcuts.ts`, `backendEvents.ts`, `dialogs.svelte.ts`, `DialogHost.svelte`).
- [x] A8-03 `editor.ts` 728→~520, `entries.ts` 574→60 (5 módulos), sugerencias de `NoteEditor` a `noteSuggestions.svelte.ts`.
- [ ] A8-04 (aplazado: sin Windows no se verifica el aspecto) `InlineEdit`, `EmptyState` y `CalendarChip` compartidos.
- [~] A8-05 Hecho `SettingsAbout`. Descartado `FolderSidebar` (CSS acoplado) y carga por pestaña (no hay pestañas). Pendiente `CalendarWeek` y `TaskDialog`.
- [x] A8-06 Descartado: los dominios diferidos suman ~2,5 KB gzip (3 % del inicial) y exigiría reescribir `t.*` en ~60 archivos.

### A9 — Paridad front/back (riesgo bajo)
- [x] A9-01 Vectores en `src-tauri/fixtures/parity/*.json` leídos por `parity_tests.rs` y `parity.test.ts` (recurrencia, orden, fechas con DST, avisos, duración). Divergencia real corregida: `parseRule` (TS) aceptaba entradas que Rust rechaza.
- [x] A9-02 D-80 `ACCEPTED`: el backend es la fuente de verdad de recurrencia y orden.

### A11 — Tests (riesgo bajo)
- [ ] A11-01 Fixtures compartidos y deduplicación de tests solapados.
- [ ] A11-02 Test Rust `search_is_fast_with_ten_thousand_tasks` falló una vez bajo carga (compilación en paralelo) y pasa aislado: hacerlo robusto. `vi.useFakeTimers` en tests TS largos; detectar tests Rust lentos.
- [ ] A11-03 Ampliar e2e: calendario, recordatorios, paleta, etiquetas, copias e importación.

### A12 — Pulido final
- [x] A12-01 `EmptyState` con variantes carga/error; corregido bug real: `TodoView` mostraba «Sin tareas» si la carga fallaba.
- [x] A12-02 Revisado, sin cambios; teclado `[H]`.
- [ ] A12-03 Instalador: cierre del proceso al actualizar y diff documentado contra la plantilla de Tauri `[H]`.
- [ ] A12-04 Lista `[H]` final para Windows 11 en `docs/manual-pruebas.md`.

## Criterio de cierre de cada fase
Batería de CLAUDE.md §4 en verde, métricas antes/después en `docs/metricas.md`, ADR donde corresponda, `Estado: AWAITING_HUMAN` y parada hasta confirmación.
