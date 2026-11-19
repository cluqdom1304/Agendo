# AUDITORIA.md — Plan de auditoría de Agendo

**Estado:** AWAITING_HUMAN — A0 y A1 hechas `[A]`; falta validar A1-01 en Windows. No se inicia A2 sin confirmación.
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
- [ ] A6-01 `refresh` de tareas con fusión por id (conserva identidad) y debounce; una sola llamada IPC.
- [ ] A6-02 Títulos de origen como `$derived` de ids únicos, carga en lote.
- [ ] A6-03 Calendario: índice de tareas por día, `Map` de etiquetas en el store, reloj sin reasignar si no cambia el minuto, `slice` como `$derived`.
- [ ] A6-04 Guardia global de `prefers-reduced-motion` en `base.css`.
- [ ] A6-05 Listeners de `onMount` sin carrera ni duplicados.
- [ ] A6-06 Perfil del editor con notas grandes `[H]`.

### A10 — Dependencias y build (riesgo bajo)
- [ ] A10-01 `cargo tree -d` y revisión de features (tokio, etc.).
- [ ] A10-02 Medir `opt-level` "s" frente a 3 (tamaño y arranque); decidir con datos.
- [ ] A10-03 Medir el coste real de Tailwind; decidir mantener o quitar. ADR.
- [ ] A10-04 Script de tokens CSS definidos y no usados.
- [ ] A10-05 `[profile.dev.package."*"]` para acelerar tests.

### A7 — Estructura Rust (riesgo medio-alto)
- [ ] A7-01 Mover tests grandes a archivos propios (`commands/tasks`, `notify`, `scheduler`).
- [ ] A7-02 Dividir `domain/markdown.rs` (read, write, inline, table) y `exchange.rs` (export, import, catalog, paths).
- [ ] A7-03 Un único módulo de saneado de rutas y nombres de Windows.
- [ ] A7-04 Extraer lógica de negocio de `commands/tasks`, `notes`, `reminders` y `tags` a servicios; los comandos solo deserializan y llaman a `run_blocking`.
- [ ] A7-05 Validación unificada de DTO (`TryFrom<Input>`).
- [ ] A7-06 Código muerto (`fts::rebuild`) y envolturas CRUD repetidas.

### A8 — Estructura frontend (riesgo medio)
- [ ] A8-01 Dividir `notes.svelte.ts` y `tasks.svelte.ts` por responsabilidad, con los `*.store.test.ts` como red.
- [ ] A8-02 `+page.svelte`: atajos, eventos y `DialogHost` en módulos propios.
- [ ] A8-03 `NoteEditor` en controladores `.svelte.ts`; `editor.ts` y `context/entries.ts` por dominio.
- [ ] A8-04 `InlineEdit`, `EmptyState` y `CalendarChip` compartidos.
- [ ] A8-05 Dividir `FolderSidebar`, `SettingsDialog` (carga por pestaña), `CalendarWeek` y `TaskDialog`.
- [ ] A8-06 Trocear `i18n.ts` por dominio si `npm run audit` demuestra ganancia.

### A9 — Paridad front/back (riesgo bajo)
- [ ] A9-01 Vectores JSON compartidos de recurrencia, orden y fechas, ejecutados por vitest y por cargo test.
- [ ] A9-02 Decidir y documentar la fuente de verdad del orden (ADR).

### A11 — Tests (riesgo bajo)
- [ ] A11-01 Fixtures compartidos y deduplicación de tests solapados.
- [ ] A11-02 Test Rust `search_is_fast_with_ten_thousand_tasks` falló una vez bajo carga (compilación en paralelo) y pasa aislado: hacerlo robusto. `vi.useFakeTimers` en tests TS largos; detectar tests Rust lentos.
- [ ] A11-03 Ampliar e2e: calendario, recordatorios, paleta, etiquetas, copias e importación.

### A12 — Pulido final
- [ ] A12-01 Estados vacío/carga/error coherentes en todas las vistas.
- [ ] A12-02 Teclado y foco: revisar filas clicables y los `svelte-ignore` / `tabindex=-1`.
- [ ] A12-03 Instalador: cierre del proceso al actualizar y diff documentado contra la plantilla de Tauri `[H]`.
- [ ] A12-04 Lista `[H]` final para Windows 11 en `docs/manual-pruebas.md`.

## Criterio de cierre de cada fase
Batería de CLAUDE.md §4 en verde, métricas antes/después en `docs/metricas.md`, ADR donde corresponda, `Estado: AWAITING_HUMAN` y parada hasta confirmación.
