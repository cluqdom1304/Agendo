# PROGRESS.md

**Fase activa:** — (Versión 1.0.3 cerrada)
**Estado:** DONE   <!-- IN_PROGRESS | AWAITING_HUMAN | BLOCKED | DONE -->
**Siguiente tarea:** ninguna. Validar los ítems `[H]` pendientes de `docs/manual-pruebas.md` (incluidas las secciones 1.0.1 y 1.0.2) y ejecutar F6-06 (E2E) en un equipo con `tauri-driver`
**Última actualización:** 2026-11-09 (cierre de la V1.0.3, etiqueta `v1.0.3`)

> Reglas: marcar `[x]` solo al pasar la batería de verificación (CLAUDE.md §4). Los ítems `[H]` nunca se marcan como verificados por el agente.

---

## Fase 0 — Esqueleto y medición base
- [x] F0-01 Crear proyecto con `npm create tauri-app@latest` (Svelte + TypeScript); registrar versiones en DECISIONS.md
- [x] F0-02 Tailwind CSS v4 mediante plugin de Vite
- [x] F0-03 ESLint, Prettier, svelte-check, rustfmt, clippy configurados
- [x] F0-04 `tauri.conf.json`: `decorations: false`, `transparent: true`, identifier estable, tamaños 1100×720 / mínimo 720×480
- [x] F0-05 `capabilities/default.json` con permisos mínimos
- [x] F0-06 `TitleBar` con `data-tauri-drag-region` y `WindowControls`
- [x] F0-07 `window/effects.rs`: Mica → Acrylic → none, comando `window_set_effect`
- [x] F0-08 Tokens CSS y tema oscuro/claro (SPEC §8.3)
- [x] F0-09 `scripts/size-check.mjs` con presupuesto N-04
- [x] F0-10 Log de arranque con marcas de tiempo (solo debug)

**Verificaciones [A]:**
- F0-02: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (0 tests), `npm run check` (0 errores, 0 avisos) y `npm run build` en verde. El CSS generado incluye el banner `tailwindcss v4.3.3`.
- F0-03: `npm run lint` (Prettier + ESLint) en verde; comprobado que ESLint falla ante `TODO` y `any`. `cargo clippy` con `unwrap_used`/`expect_used`/`todo`/`unimplemented` en `deny`.
- F0-04: `src-tauri/tests/tauri_conf.rs` (3 tests) comprueba ventana `main` sin decoraciones, transparente, con sombra, 1100×720 / mínimo 720×480 e identifier. `tauri-build` acepta la configuración (clippy/test compilan). La apariencia real es [H] (manual-pruebas § Fase 0).
- F0-05: `src-tauri/tests/capabilities.rs` (4 tests) fija el conjunto exacto de permisos, sin `*:default`, solo ventana `main` y `opener` limitado a `http`/`https`/`mailto`. Los permisos de notificaciones (F4-05) y diálogo de archivos (F6-04) se añadirán con sus plugins, porque un permiso de un plugin no instalado rompe el build. `core:event` se añadirá en F4-08 y cualquier permiso extra de ventana (p. ej. `allow-is-maximized`) se añadirá en la tarea que lo use, actualizando el test.
- F0-06: `TitleBar` y `WindowControls` en `src/lib/shell/` (SvelteKit, D-12; SPEC §10 indica `src/shell/`), textos en `src/lib/i18n.ts`. Lint, `svelte-check` y build en verde. Sin pruebas de componente: Vitest aún no está instalado y la tarea no exige test [A]; el comportamiento real (arrastre, botones) es [H]. Los estilos usan tokens (`--bg-hover`, `--danger`, etc.) que define F0-08; hasta entonces no tienen valor.
- F0-07: `window/effects.rs` + `commands/window.rs` (`window_set_effect` → `EffectApplied { requested, applied }`). 9 tests de fallback con backend simulado (Mica→Acrylic→none, `none` solo limpia, error `INTERNAL:`, serialización en minúsculas). Clippy/tests compilan la ruta nativa de Windows, por lo que `cargo check --target x86_64-pc-windows-msvc` queda cubierto en este equipo. Que el efecto se vea realmente es [H].
- F0-08: `src/styles/tokens.css` (SPEC §8.3), `src/styles/base.css` (raíz `.app-root` con radio 10 px y `.native-effect`, scrollbars de 4 px, `prefers-reduced-motion`), `src/lib/api.ts` (único punto IPC), `src/lib/stores/windowEffect.svelte.ts` (runes; pide `mica` al montar y fija `.native-effect` según el efecto devuelto). Lint, `svelte-check`, build y Rust en verde; el CSS compilado contiene tokens, `.native-effect` y utilidades de Tailwind. Se eliminó el demo de la plantilla (comando `greet`, logos y la página con estilos `:root` que habrían tapado los tokens y el efecto). Apariencia, tema y scrollbars reales son [H].
- F0-09: `scripts/size-check.mjs` + `npm run size` suman el JS de `build/` en gzip (nivel 9, KB = 1024 B), informan por fichero y fallan (exit 1) si superan 250 KB; `--budget-kb` permite ajustar el límite (comprobado: con 1 KB falla). Perfil `release` alineado con SPEC §10.1. `npx tauri build --no-bundle` genera `miagendaapp.exe` (DoD de F0: ejecutable generado, tamaño registrado). Se usa `--no-bundle` porque el instalador es F6-07.
- F0-10: `src-tauri/src/startup.rs` con marcas `process_start`, `setup_done`, `page_load_started/finished` (Rust) y `first_paint` (frontend, 2 × `requestAnimationFrame`, vía comando `startup_mark` validado con `VALIDATION:`). Todo se compila a no-op sin `debug_assertions`; `cargo clippy --release -D warnings` en verde. 5 tests (formato, validación de etiquetas, `mark` antes de `init`). La salida real en `tauri dev` es [H].
- Fin de Fase 0, batería final: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` (14 de lib + 4 + 3 de integración), `npm run lint`, `npm run check`, `npm run build`, `npm run size` en verde.
- No existen aún (no cuentan como fallo): `npm run test` (Vitest, F1/F2), `npm run size` (F0-09).

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 0

---

## Fase 1 — Persistencia y dominio
- [x] F1-01 `db/`: conexión con pragmas (SPEC §5.4), migraciones por `user_version`, test de migración desde vacío
- [x] F1-02 Migración `001_init.sql` (SPEC §5.2)
- [x] F1-03 `repo/tasks.rs`, `repo/tags.rs`, `repo/fts.rs` con sincronización FTS en transacción
- [x] F1-04 `domain/dates.rs`: validación `YYYY-MM-DD` / `HH:MM`, conversión local/UTC
- [x] F1-05 `domain/ordering.rs`: funciones puras de orden (D-05) con tests de tabla
- [x] F1-06 `domain/validation.rs`: longitudes, profundidad de etiquetas (D-06)
- [x] F1-07 Comandos `tasks_*`, `tag_*`, `task_tags_set`, `trash_list`, `task_restore`
- [x] F1-08 `error.rs` con prefijos de código (D-08)
- [x] F1-09 `api.ts` y tipos DTO sincronizados con Rust (test de serialización)

**Verificaciones [A]:**
- F1-01: `src-tauri/src/db/` (`mod.rs`: `open`, `open_in_memory`, `DbError`; `pragmas.rs`: SPEC §5.4; `migrate.rs`: `Migration`, `migrate` por `user_version`). Cada migración corre en su propia transacción `IMMEDIATE` junto con el `user_version`; si falla se revierte, y una base más nueva que la app se rechaza (`SchemaTooNew`). 16 tests: migración desde vacío, idempotencia, solo pendientes, rollback de una migración fallida (memoria y disco), lista no consecutiva, pragmas reales en fichero (WAL, `foreign_keys`, `synchronous=NORMAL`, `busy_timeout=5000`, `temp_store=MEMORY`), reapertura sin reaplicar. Los tests usan migraciones de prueba; la lista real (`001_init.sql`) llega en F1-02. Dependencia `rusqlite` registrada como D-17. `cargo fmt --check`, `clippy --all-targets -D warnings`, `cargo test` (30 de lib + 4 + 3), `npm run lint`, `check`, `build` y `size` (32,8 KB) en verde; `npm run test` aún no existe.
- F1-02: `src-tauri/migrations/001_init.sql` (copia literal de SPEC §5.2) incrustado con `include_str!` en `db::MIGRATIONS`. `db/schema_tests.rs` (12 tests) comprueba migración desde vacío a `user_version = 1`, tablas e índices, valores por defecto, `CHECK` de título/fechas/hora/color/nombre (incluidos límites 500 y 60), unicidad de etiquetas sin distinguir mayúsculas, claves foráneas, cascadas (`ON DELETE CASCADE` y `SET NULL`) y FTS5 con y sin acentos, mayúsculas y prefijo. Batería completa en verde (42 tests de lib + 4 + 3; bundle JS 32,9 KB).
- F1-03: `repo/{model,tasks,tags,fts}.rs`. Todas las escrituras reciben `&Transaction` (el FTS se actualiza en la misma transacción por construcción) y `repo::with_tx` ejecuta una operación lógica en una transacción `IMMEDIATE`. El repositorio no lee el reloj ni genera ids: el llamador pasa `id` y `now` (UTC). Devuelve `rusqlite::Result`; "no encontrado" es `None`/`false` y los errores de restricción los traducirá `error.rs` (F1-08). `Task` y `Tag` ya serializan en camelCase según SPEC §7.2 (tags y recordatorios se leen con subconsultas JSON, sin N+1). 40 tests nuevos: CRUD, posición al final de la sección, parches (incluido borrar fecha/hora), completar/reabrir, soft delete, restaurar, papelera, purga a 30 días (límite exacto, cascada), listados todo/agenda/rango con filtro por etiqueta, etiquetas (upsert, contadores de pendientes, borrado, `set_task_tags`), FTS (insert/sync/rebuild/rollback) y **10 000 operaciones aleatorias** tras las que `integrity_check = ok`, `foreign_key_check` vacío y `tasks_fts` idéntica a las tareas vivas. Batería completa en verde (82 tests de lib + 4 + 3).
- F1-04: `domain/dates.rs` (dependencia `chrono`, D-18). `parse_date`/`parse_time`/`parse_instant` son estrictos (`YYYY-MM-DD` con fecha de calendario real, `HH:MM` 00:00–23:59, `YYYY-MM-DDTHH:MM:SSZ`; rechazan espacios, signos, dígitos no ASCII y formatos laxos) y sus errores `DateError` tienen mensaje en español. `local_to_utc` / `utc_to_local` reciben la zona como parámetro (`chrono::Local` en producción); no se lee el reloj. Reglas de DST fijadas y probadas con una zona simulada tipo Madrid: hora inexistente de primavera pasa a la hora siguiente (02:30 → 03:30), hora repetida de otoño = primera aparición; fechas extremas devuelven `OutOfRange` en lugar de provocar el pánico de `chrono`. 17 tests (99 de lib en total + 4 + 3). La zona horaria real del equipo solo se ejercita con un test de ida y vuelta fuera de transiciones.
- F1-05: `domain/ordering.rs` con el trait `Sortable` (implementado para `repo::model::Task`, así `domain` no depende de `repo`), `sort_todo`, `sort_day` y `group_by_day` (`DayGroup<T>`, serializa `{ date, tasks }`; deja fuera las tareas sin fecha). Implementa D-05: día más reciente primero; dentro del día, con hora (ascendente) → sin hora (`position`) → completadas por `completed_at`; `id` desempata, de modo que el resultado no depende del orden de entrada (comprobado con todas las permutaciones). Tests de tabla (11 casos de día, 5 de POR HACER), agrupación, serialización y uno con datos reales de SQLite: 7 tests nuevos (106 de lib + 4 + 3). D-05 no dice qué pasa con las completadas sin fecha: se aplicó la misma regla de subsección y se registró la decisión pendiente como B-03.
- F1-06: `domain/validation.rs` con `ValidationError` (mensajes en español; `error.rs` añadirá el prefijo `VALIDATION:`). `validate_title` (≤ 500 caracteres, no bytes), `validate_tag_name` (devuelve el nombre recortado, 1–60), `validate_color_hex` (`#RRGGBB`), `validate_content_json` (objeto JSON de tipo `doc`), `validate_schedule` (fecha y hora válidas vía `dates.rs`) y `validate_tag_parent` (D-06: no ser su propio padre, el padre no puede ser subetiqueta y una etiqueta con hijos no puede pasar a ser hija; el llamador carga el padre y `has_children`). Dos reglas no estaban explícitas en SPEC y se añadieron por coherencia con el esquema: una hora exige fecha (POR HACER no tiene horario) y `content_json` debe ser un documento `doc`; los límites de `content_json`/`content_text` no existen en SPEC y no se inventaron. 10 tests de tabla (116 de lib + 4 + 3).
- F1-08 (hecha antes que F1-07 porque los comandos la necesitan para devolver `Result<T, String>`): `error.rs` con `AppError` (`NotFound`, `Validation`, `Conflict`, `Db`, `Io`, `Notify`, `Internal`), `code()`, `message()` y `Display` = `CODIGO: mensaje`. `From<AppError> for String` permite usar `?` en el cuerpo de un comando. Conversiones: `rusqlite::Error` (UNIQUE/PRIMARY KEY → `CONFLICT`, FOREIGN KEY → `NOT_FOUND`, CHECK/NOT NULL → `VALIDATION`, sin filas → `NOT_FOUND`, resto → `DB` con detalle), `DbError`, `ValidationError`, `DateError` e `io::Error`. Los tests provocan violaciones reales en SQLite. 8 tests (124 de lib + 4 + 3). Los errores con cadena propia de F0 (`startup`, `window`) ya usan el mismo formato y no se han migrado a `AppError`.
- F1-07: `commands/tasks.rs` (`tasks_list`, `task_create`, `task_update`, `task_set_completed`, `task_move`, `task_delete`, `task_restore`, `trash_list`) y `commands/tags.rs` (`tag_list`, `tag_upsert`, `tag_delete`, `task_tags_set`), registrados en `lib.rs`. Cada comando es un envoltorio `async` que ejecuta una función normal (la que prueban los tests) en `spawn_blocking`; las escrituras van en una transacción `IMMEDIATE` y los errores salen como `CODIGO: mensaje`. Piezas nuevas: `state.rs` (`AppState` con `Db` = conexión de escritura + conexión de solo lectura en WAL, `Clock` e `IdGenerator`; al abrir purga la papelera de más de 30 días), `ids.rs` (UUID v7 desde el `Clock`, D-19), `domain/content.rs` (`content_text` derivado de `content_json`, D-03), `repo::tasks::move_task` (reordena y renumera la sección destino; pasar a POR HACER borra fecha y hora), `repo::tags::has_children`, `db::open_read_only` y `scheduler/clock.rs` (trait `Clock` con `SystemClock` y `ManualClock`, adelantado porque los comandos necesitan "ahora"; F4-01 sigue sin marcar hasta revisarlo en su fase). `task_update` distingue campo ausente de `null`; quitar la fecha quita también la hora. `tasks_list` con `range` devuelve `Task[]` plano ascendente y valida `from ≤ to`. El evento `tasks:changed` (SPEC §7.4) no se emite aún: necesita `core:event` (F4-08). Batería completa en verde (183 tests de lib + 4 + 3; bundle JS 32,9 KB); `npm run test` aún no existe.
- F1-09: `src/lib/types.ts` (DTO de SPEC §7.2 más `TagWithCount`, `DayGroup`, entradas de comandos y `ErrorCode`) y `src/lib/api.ts` con un envoltorio tipado por comando (`tasksList`, `taskCreate`, `taskUpdate`, `taskSetCompleted`, `taskMove`, `taskDelete`, `taskRestore`, `trashList`, `tagList`, `tagUpsert`, `tagDelete`, `taskTagsSet`) y `parseError` (separa `CODIGO: mensaje`; códigos desconocidos → `INTERNAL`). Test de contrato en dos mitades: `src-tauri/src/contract_tests.rs` compara la serialización real de los DTO con `src-tauri/fixtures/dto.json` (se regenera con `UPDATE_DTO_FIXTURE=1 cargo test contract`) y `src/lib/types.test.ts` (Vitest, D-20) comprueba el fixture contra los tipos con `Record<keyof T, true>`, de modo que añadir o quitar un campo en cualquier lado rompe un test. Cubre `null` en opcionales, `pendingCount` aplanado y las dos formas de `tasks_list`. Vitest 5.1.0 instalado; `npm run test` existe ya. Batería completa en verde: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (184 de lib + 4 + 3), `npm run lint`, `check` (0 errores), `test` (8), `build`, `size` (32,9 KB).
- **Fin de Fase 1:** todas las tareas [A] (F1-01…F1-09) en verde. `cargo check --target x86_64-pc-windows-msvc` queda cubierto en este equipo (Windows): clippy y tests compilan la ruta nativa; la fase no toca `window/` ni `notify/`.
**Ítems [H]:** ninguno en esta fase (`docs/manual-pruebas.md` no tiene sección de Fase 1)

---

## Fase 2 — Vista principal tipo documento
- [x] F2-01 Componentes `SectionHeader`, `DayGroup`, `TaskRow`, `Checkbox`, `InlineTitle`
- [x] F2-02 Stores con runes (`tasks.svelte.ts`) y agrupación con `$derived`
- [x] F2-03 Secciones "POR HACER" y "AGENDA" con fechas `DD/MM/YYYY` descendentes
- [x] F2-04 Creación rápida (Ctrl+N, Enter en título)
- [x] F2-05 Completar con animación y reubicación en "Completadas"
- [x] F2-06 Edición inline de título (guardado en Enter o pérdida de foco)
- [x] F2-07 Eliminación con deshacer de 5 s (F-02)
- [x] F2-08 Atajos de SPEC §9 para estas acciones (`lib/keymap.ts`)
- [x] F2-09 Estados vacíos (`EmptyState`)

**Verificaciones [A]:**
- Fase 2 completa (F2-01…F2-09, un único commit porque las piezas son interdependientes y los tests de componentes necesitan todas). Código en `src/lib/`: `ordering.ts` (port de `domain/ordering.rs`, así las actualizaciones optimistas caen donde las dejaría una recarga), `dates.ts`, `keymap.ts` (tabla única de SPEC §9 con contexto de foco: título / lista / otro), `stores/tasks.svelte.ts` (`TasksStore`, runes; tareas en `$state.raw` inmutables, bloques `todoBlock`/`agendaBlocks` con `$derived`, `settling` como `SvelteSet`), `features/todo/{Checkbox,InlineTitle,TaskRow,SectionHeader,DayGroup,TodoView}.svelte`, `shell/{EmptyState,Toast}.svelte`. `+page.svelte` carga el store real; los textos están en `i18n.ts` y los códigos de error se traducen por prefijo (nunca se muestra `DB:` crudo).
- Comportamiento: Ctrl+N crea en el día de la tarea seleccionada o en POR HACER, Ctrl+Mayús+N con fecha de hoy. La fila nueva es un borrador no persistido (no se crean tareas vacías); Intro guarda y abre el siguiente borrador, Esc o perder el foco vacío lo descarta, y si el guardado falla el borrador conserva su texto. Completar: animación de 220 ms (0 con `prefers-reduced-motion`), la fila pasa a "Completadas" sin esperar al backend, que luego confirma o revierte. Renombrar y eliminar son optimistas con reversión. Eliminar usa `task_delete` (papelera) y ofrece 5 s de deshacer con `task_restore` (botón del aviso o Ctrl+Z); varios borrados se deshacen del más reciente al más antiguo.
- Tests (DoD de F2): 56 de Vitest (antes 8) — orden y agrupación (`ordering.test.ts`), atajos (`keymap.test.ts`), store con API simulada incl. actualización optimista y reversión si falla `invoke` (`tasks.store.test.ts`) y componentes con jsdom (`TodoView.test.ts`): agrupación por día y orden, completar mueve a "Completadas", deshacer con botón y Ctrl+Z, Ctrl+N / Intro / Esc, edición, navegación con flechas y un test de renders de fila (un título que cuenta sus lecturas demuestra que renombrar o completar una tarea no re-renderiza las demás).
- Batería: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` (184 de lib + 4 + 3; sin cambios de Rust en la fase), `npm run lint`, `npm run check` (0 errores, 0 avisos), `npm run test` (56), `npm run build` y `npm run size` (JS 41,3 KB gzip de 250) en verde.
- Dependencias nuevas: `@testing-library/svelte` 5.4.2 y `jsdom` 31.0.0 (D-21), solo desarrollo.
- Desviaciones: SPEC §10 sitúa los componentes en `src/features/` y `src/shell/`; con SvelteKit (D-12) viven en `src/lib/features/todo/` y `src/lib/shell/`. `TodoView` es una vista nueva y `DayGroup` sirve también para el bloque POR HACER (`block.date === null`) para no añadir un componente fuera del inventario de SPEC §8.6. La subsección "Completadas" también existe en POR HACER (opción 1 de B-03, sigue abierto). No hay arrastre para reordenar (`task_move` no se usa hasta que una tarea lo pida) ni etiquetas en la fila (llegan con el Sidebar). El evento `tasks:changed` aún no existe (F4-08), así que el store no se refresca por eventos.

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 2 (8 ítems; los 8 validados por un humano el 2026-11-06; dos fallaron al principio (sin forma de poner fecha ni botón de eliminar) y se corrigieron en `0da79b4` con selector de fecha y botón de eliminar en la fila)

---

## Fase 3 — Editor de notas y búsqueda
- [x] F3-01 `NoteEditor` con TipTap (carga diferida, D-10) y `NoteToolbar`
- [x] F3-02 Extensiones: negrita, cursiva, código, enlace, listas, historial
- [x] F3-03 Guardado con debounce de 400 ms y `content_text` derivado
- [x] F3-04 Sub-puntos indentados bajo la tarea (24 px)
- [x] F3-05 Spotlight (`Ctrl+K`) con búsqueda FTS y navegación por teclado
- [x] F3-06 Saneado de enlaces (solo `http`, `https`, `mailto`)

**Verificaciones [A]:**
- Fase 3 completa (F3-01…F3-06, un commit: editor, guardado y buscador comparten store y `TodoView`). Backend: `repo/search.rs` (FTS5 con cada palabra entrecomillada y como prefijo, `bm25` con el título pesando 4× las notas, `snippet()`), `commands/search.rs` (`search { query, limit? }`, límite 50 por defecto y 200 máximo, SPEC §7.3) y `validate_content_json` rechaza cualquier marca `link` cuyo `href` no sea `http`/`https`/`mailto` (en cualquier nivel del documento). `content_text` ya se derivaba en cada guardado desde F1 (`domain/content.rs`). Frontend: `features/notes/{editor.ts,NoteEditor,NoteToolbar}`, `noteSaver.ts`, `content.ts`, `links.ts`, `shell/Spotlight.svelte`; atajos `Ctrl+K` (tabla de `keymap.ts`), `Ctrl+B/I/E` (extensiones de TipTap) y `Ctrl+L` (campo de enlace).
- Comportamiento: cada fila tiene un botón de notas (visible al pasar el ratón o con notas); abre el editor bajo la tarea, de una en una. Las listas de la nota llevan 24 px de sangría. Guardado a los 400 ms de la última pulsación, secuencial (nunca se solapan dos guardados), con vaciado al cerrar la nota; si falla, el texto se conserva y se reintenta. Completar una tarea cierra sus notas antes de moverla. Ctrl+clic en un enlace lo abre con `opener` (ya permitido solo para http/https/mailto); un clic normal nunca navega la ventana. Spotlight descarta respuestas antiguas, captura el teclado mientras está abierto y, al elegir un resultado, selecciona la tarea y abre sus notas si las tiene.
- Tests de Rust: 194 de lib (antes 184; +10: enlaces, tabla de esquemas, búsqueda con y sin acentos en ambos sentidos, prefijos, entrada hostil, borradas, ranking, rendimiento, comando) + 4 + 3. **F-09: búsqueda en 20 ms en el peor caso con 10 000 tareas** (límite 100 ms). Contrato Rust↔TS ampliado con `SearchHit` (`fixtures/dto.json`).
- Tests de Vitest: 105 (antes 56): `links.test.ts` (saneado: `javascript:`, `file:`, `data:`, tabuladores intercalados…), `noteSaver.test.ts` (debounce, orden, reintento), `NoteEditor.test.ts` (TipTap real en jsdom: carga diferida, guardado a los 400 ms, vaciado al cerrar, enlaces rechazados), `Spotlight.test.ts` (flechas, Intro, Esc, respuestas desordenadas, error), `TodoView.test.ts` (abrir/cerrar notas, Ctrl+K → Intro, atajos bloqueados con el buscador abierto) y store.
- Batería: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test`, `npm run lint`, `npm run check` (0 errores, 0 avisos), `npm run test`, `npm run build` y `npm run size` en verde. JS total 159,8 KB gzip de 250: **bundle inicial ≈ 45 KB; TipTap vive en un chunk diferido de 114,6 KB** que solo se pide con `import()` desde `NoteEditor` (comprobado en `build/`).
- Dependencias nuevas: TipTap 3.33.0 (D-22), extensiones sueltas en lugar de `starter-kit`.
- Desviaciones: el botón de notas no está en SPEC §8.6; es la única forma de abrir el editor con el ratón y con teclado hasta que haya panel de detalle. `Ctrl+Z` dentro del editor lo gestiona el historial de TipTap, no el deshacer de borrado. La resolución humana de B-03 se aplicó después como D-23 (commit aparte).

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 3 (6 ítems; ninguno verificado por el agente)

---

## Fase 4 — Recordatorios y notificaciones
- [x] F4-01 `scheduler/clock.rs`: trait `Clock` y dos implementaciones
- [x] F4-02 `scheduler/mod.rs`: bucle `select!` con tope de 60 s y reprogramación por `Notify`
- [x] F4-03 Disparo transaccional con reintento y backoff (SPEC §6.3)
- [x] F4-04 Recuperación al arrancar (ventana 24 h, SPEC §6.4)
- [x] F4-05 `notify/mod.rs` con `tauri-plugin-notification`
- [x] F4-06 `ReminderPopover` con opciones de F-12
- [x] F4-07 Recalcular recordatorios al cambiar fecha/hora de la tarea
- [x] F4-08 Evento `reminder:fired` y toast in-app

**Verificaciones [A]:**
- F4-01: `scheduler/clock.rs` (trait `Clock` con `now()`, `SystemClock` y `ManualClock`) ya existía desde F1 porque los comandos necesitan «ahora»; la firma es `now()` y no `now_utc()` como en SPEC §6.2 (misma semántica). Único lugar con `Utc::now()`.
- F4-02/03/04: `scheduler/mod.rs`. `Engine::tick` (una pasada: caduca lo vencido hace > 24 h sin avisar, dispara lo demás) y `Engine::wait` (hasta el próximo aviso o reintento, tope de 60 s); `run` es el bucle con `tokio::select!` entre `sleep` y `AppState.wake` (`Notify`). El marcado `is_sent` y la notificación van en **una sola transacción**: si la notificación falla, o el proceso muere, se revierte y se reintenta (30, 60, 120, 120 s; a los 5 intentos se registra y se marca como enviado). La recuperación al arrancar no necesita código aparte: la primera vuelta del bucle la hace (D-26).
- F4-05: `notify/mod.rs` (`Notifier`, `FiredSink`, `TauriNotifier`, `TauriFiredSink`); título = tarea, cuerpo = etiquetas · hora. Sin acción «Marcar como hecha» en v1 (B-01, resuelto por el humano). Plugin `tauri-plugin-notification 2.8.0` (D-25); se usa solo desde Rust, así que no se añade ningún permiso de frontend.
- F4-07: `repo::reminders::recalculate` se llama en la misma transacción desde `task_update`, `task_set_completed` y `task_move` (completar cambia `due_date`, D-23). Nuevo comando `reminders_set` (`commands/reminders.rs`, SPEC §7.3): reemplaza los no enviados, valida (offset 0–30 días, requiere fecha y hora para offsets, no admite avisos en el pasado, máx. 10) y es atómico.
- F4-06/08 (frontend): `ReminderPopover.svelte` (4 opciones de F-12, hora personalizada, lista con quitar, y campo de hora de la tarea, porque sin hora no hay avisos relativos y la interfaz no tenía otra forma de ponerla), botón de campana en `TaskRow`, `lib/reminders.ts` (funciones puras), `store.setReminders/setDueTime/onReminderFired`. El evento `reminder:fired` se escucha en `+page.svelte` (permisos `core:event:allow-listen/unlisten` añadidos y fijados en `tests/capabilities.rs`); refresca las tareas y muestra un toast informativo (`Notice.kind`).
- Decisiones: D-24 (tokio directo), D-25 (plugin de notificaciones), D-26 (semántica: solo tareas abiertas y no borradas disparan; al quitar fecha u hora se borran los offsets no enviados; caducidad de 24 h en cada vuelta, no solo al arrancar). B-01 y B-02 pasan a RESUELTO con la resolución humana ya escrita.
- DoD [A] de la fase: disparo exacto (`fires_exactly_when_the_time_comes_and_only_once`), reintento tras fallo (`a_failed_notification_is_rolled_back_and_retried_with_backoff`), no duplicados (mismo test + `_gives_up_after_five_attempts_`), recuperación < 24 h y > 24 h (+ borde exacto de 24 h), cambio de hora reprograma (`changing_the_time_of_a_task_reschedules_its_relative_reminders`, `a_new_zone_offset_is_seen_by_recalculation`), bucle real reprogramado por `Notify` (`the_loop_fires_after_a_wake_up_...`). **Test de corte:** `tests/crash_recovery.rs` lanza un proceso hijo que hace `abort()` en mitad de una transacción; el padre comprueba `integrity_check = ok` y que lo no confirmado no existe.
- Tests: Rust 228 de lib + 4 (capabilities) + 1 (corte) + 3 (tauri.conf); Vitest 133. Batería completa en verde: `cargo fmt --check`, `cargo clippy --all-targets -D warnings` (y `--release`), `cargo test`, `npm run lint`, `npm run check` (0/0), `npm run test`, `npm run build`, `npm run size` (JS 162,2 KB gzip de 250). `cargo check` para Windows queda cubierto: este equipo es Windows y compila las rutas `cfg(windows)`.
- Desviaciones: el campo de hora y la campana no están en SPEC §8.6 (necesarios para poder usar F-12). La zona horaria se lee con `chrono::Local` en el comando; los tests de comandos comparan contra la misma conversión, y los de `domain`/`repo` usan zonas fijas simuladas.

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 4 (9 ítems originales + 3 nuevos; ninguno verificado por el agente)

---

## Fase 5 — Vistas de calendario
- [x] F5-01 `SegmentedControl` (Lista / Día / Semana / Mes), persistido en `settings`
- [x] F5-02 `CalendarMonth`, `CalendarWeek`, `CalendarDay`
- [x] F5-03 Creación rápida arrastrando un tramo horario (F-11)
- [x] F5-04 Carga diferida del módulo de calendario (`import()`)
- [x] F5-05 Primer día de semana según sistema (`Intl.Locale`)

**Verificaciones [A]:**
- Fase 5 completa (F5-01…F5-05, un commit: las vistas comparten store, cuadrícula y creación). Backend: `repo/settings.rs` y `commands/settings.rs` (`settings_get`/`settings_set`, SPEC §7.3) con lista blanca: solo `view_mode` (`list|day|week|month`); clave o valor desconocido → `VALIDATION:` sin guardar nada. 3 tests de Rust (233 de lib en total).
- Frontend: `shell/SegmentedControl.svelte`, `stores/view.svelte.ts` (`ViewStore`: modo persistido, fecha ancla, `shift`, `showDay`; no pinta hasta leer el ajuste para evitar el parpadeo de la lista; si falla leer o guardar sigue funcionando), `features/calendar/{grid.ts,labels.ts,CalendarMonth,CalendarWeek,CalendarDay,CalendarView}`. `CalendarDay` es `CalendarWeek` con una columna. Atajos `Ctrl+1…4` (SPEC §9) en `keymap.ts`. Clic en una tarea del calendario vuelve a la lista y la selecciona; clic en el número de un día (o «+N más») abre la vista Día.
- F5-03: arrastrar en una columna de Día/Semana (saltos de 15 min, también hacia arriba, o un simple clic) muestra el tramo y un campo de título; Intro (o perder el foco con texto) crea la tarea con `dueDate` y `dueTime` = inicio del tramo; Esc o título vacío no crea nada; si el backend falla, el texto se conserva. `store.createScheduled`.
- F5-04: `+page.svelte` carga `CalendarView` con `import()`; en `build/` el calendario es un chunk aparte (≈3,5 KB gzip) que `nodes/2` solo precarga bajo demanda. Lo único que crece en el bundle inicial es el store de vista, el control segmentado y los textos (JS total 162,2 → 169,4 KB de 250).
- F5-05: `systemFirstDay` lee `Intl.Locale(navigator.language).getWeekInfo()` (o `weekInfo`) y cae a lunes si no existe; fechas `DD/MM/YYYY` vía `formatDisplayDate`.
- Tests (DoD de F5): Vitest 173 (antes 133): `grid.test.ts` (días, meses con recorte de día, semanas con distinto primer día, 60 meses × 3 primeros días sin huecos ni repetidos, **DST** en Europe/Madrid, America/New_York y Pacific/Auckland alrededor de los cambios de hora, `Intl` y valor por defecto, trazado de bloques solapados), `view.store.test.ts` y `CalendarView.test.ts` (semana/día/mes, bloques en su hora, creación por arrastre en ambos sentidos, clic, Esc, título vacío, fallo del backend, navegación).
- Batería: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test`, `npm run lint`, `npm run check` (0/0), `npm run test` (173), `npm run build` y `npm run size` (169,4 KB gzip de 250) en verde. `cargo check` de Windows cubierto (el equipo es Windows); la fase no toca `window/` ni `notify/`.
- Desviaciones: (1) las tareas no tienen hora de fin en el esquema, así que el tramo arrastrado solo fija la hora de inicio y los bloques ocupan 30 min visuales → B-04. (2) «Los bloques de cada asignatura usan su color» (manual-pruebas) no es posible todavía: no existe almacén ni interfaz de etiquetas (Sidebar); los bloques usan el color de acento. Llegará con la tarea que cargue etiquetas. (3) Con un calendario visible no hay buscador (Ctrl+K vive en `TodoView`); no se movió para no ampliar el alcance. (4) Sin componentes fuera del inventario de SPEC §8.6.

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 5 (los ítems originales más 3 nuevos; ninguno verificado por el agente)

---

## Fase 6 — Pulido, accesibilidad, copias y empaquetado
- [x] F6-01 Modo claro/oscuro completo y verificado
- [x] F6-02 Revisión automatizada de contraste sobre tokens (N-08)
- [x] F6-03 Navegación por teclado completa y ARIA en controles
- [x] F6-04 `backup_create` / `backup_restore` con validación (F-16, D-09)
- [x] F6-05 Ajustes: tema, inicio con Windows (desactivado por defecto), efecto de ventana
- [ ] F6-06 Tests E2E con WebdriverIO + tauri-driver — escritos y con lint, **falta ejecutarlos `[H]`** (`npm run e2e`, ver `e2e/README.md`)
- [x] F6-07 Instalador NSIS/MSI, icono y metadatos (firma pendiente)
- [x] F6-08 Tabla de métricas N-01 a N-08 en `docs/`

**Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` (242 + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (259), `npm run build` y `npm run size` (173,1 KB gzip de 250) en verde. `npm run tauri -- build` genera `miagendaapp.exe` (12,06 MiB), el NSIS (3,17 MiB) y el MSI (4,48 MiB). Cobertura de `domain/` 98,8 % de líneas (`cargo llvm-cov`). La fase toca `window/` solo por lectura de ajustes; `cargo check` de Windows es el check nativo de este equipo.

**Notas de la fase:**
- F6-01/02: `src/lib/a11y/tokens.test.ts` calcula el contraste WCAG de cada token de texto sobre las superficies de la ventana en los tres bloques de tokens (oscuro, claro forzado y claro del sistema) y exige 4,5:1. Falló en 12 tokens; se corrigieron y se añadieron `--on-accent` / `--on-danger` → **D-27 (`PROPOSED`)**, porque toca colores de SPEC §8.3. El fondo nativo (Mica) se modela como `#202020` / `#f3f3f3`: la medición real es `[H]`. El tema se aplica con `data-theme` desde `settingsStore.load()` en el layout; hasta que se lee el ajuste se ve el tema del sistema (un parpadeo posible solo si el usuario forzó el opuesto).
- F6-03: ya existían nombres accesibles casi en todo; `src/lib/a11y/names.ts` audita botones, campos e interruptores en lista, Día, Semana, Mes, buscador, ajustes y ventana (todos con nombre). Hueco encontrado y cerrado: las columnas de día de Semana/Día solo se podían usar con ratón; ahora son enfocables y `Intro`/`Espacio` abren «Nueva tarea» (hoy: siguiente bloque de 30 min; otros días: 09:00). Se añadió un `:focus-visible` global como red de seguridad.
- F6-04: `db/backup.rs` + `commands/backup.rs` (D-30). Restaurar valida (cabecera, `integrity_check`, `foreign_key_check`, versión, tablas, `page_size`), guarda lo actual en `agenda.before-restore.sqlite`, copia con la API de backup de SQLite y migra si la copia es anterior. Resuelve B-02 (solo `.sqlite`, importable). Cierra la nota de F1-01: la copia previa a migrar sigue sin existir; la migración sigue haciendo rollback.
- F6-05: claves `theme` (`system|light|dark`) y `window_effect` (`mica|acrylic|none`) en la whitelist de `settings_*`; autostart por comandos propios (D-29). Diálogo de ajustes con el `Modal` del inventario de SPEC §8.6 (nuevo componente `SettingsDialog` dentro de `features/settings/`, que SPEC §10 ya prevé). Sin atajo de teclado propio: SPEC §9 no define uno.
- F6-06: `e2e/` (3 specs, `wdio.conf.mjs`, README). La app acepta `MIAGENDA_DATA_DIR` para que las pruebas usen datos temporales (D-31).
- F6-07: `tauri.conf.json` con NSIS (por usuario, español) y MSI (es-ES), publisher, copyright y descripciones; un test fija objetivos, metadatos e iconos. Firma pendiente `[H]`. El identifier sigue siendo `com.cristian.miagendaap` (ver nota de 2026-10-31): **decidir antes de distribuir**, porque cambiarlo después mueve los datos de los usuarios.
- F6-08: `docs/metricas.md`. N-03 a N-06 con valores reales; N-01, N-02 y la parte real de N-07/N-08 quedan `[H]`.

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 6

**Mejoras posteriores a la Fase 6 (2026-11-09, a petición del usuario):**
- Botón «Nueva tarea» (Ctrl+N sigue creando la fila en línea) que abre `NewTaskDialog`: título, fecha, hora, descripción (texto → párrafos) y recordatorios con el mismo `ReminderEditor` que el popover de cada fila. Si la creación va bien pero los avisos fallan, la tarea queda y se informa del error.
- Los avisos ya sonados se muestran en gris («Avisado», bajo «Ya avisados», últimos 5) en vez de desaparecer.
- Avisos periódicos (D-32, migración 002): cada N horas/días/semanas/meses; termina nunca / tras N veces / en una fecha. Solo para avisos de hora concreta.
- UX: el diálogo enfoca el título, Intro envía, Ctrl+Intro envía desde cualquier campo, la hora se desactiva sin fecha, el calendario propone el día que se está viendo, avisos pasados se detectan antes de guardar y los botones llevan ayuda con el atajo.
- Verificación [A]: `cargo test` 256+4+1+4, `npm run test` 284, lint/check/build/size en verde (JS 177,1 KB gzip). [H] nuevos en `docs/manual-pruebas.md` § Mejoras posteriores.

---

## Métricas

| Métrica | Objetivo | Valor | Fecha | Notas |
|---------|----------|-------|-------|-------|
| Tamaño ejecutable (N-03) | < 15 MB | 12 641 792 B (≈ 12,06 MiB) | 2026-11-09 | `tauri build` release de F6-07 (antes 8,1 MiB en F0-09); instaladores NSIS 3,17 MiB y MSI 4,48 MiB |
| Bundle JS gzip (N-04) | < 250 KB | 173,1 KB (CSS 8,5 KB) | 2026-11-09 | `npm run size` al cierre de F6 |
| Arranque (N-01) [H] | < 200 ms | | | |
| RAM en reposo (N-02) [H] | < 40 MB (proceso principal) | | | |
| Consulta de un mes, 10 000 tareas (N-05) | < 30 ms | 2,98 ms (434 filas, mejor de 5) | 2026-11-09 | Test `month_query_with_ten_thousand_tasks_is_fast`, build de depuración; el test falla si supera 30 ms |
| Búsqueda, 10 000 tareas (F-09) | < 100 ms | | | |
| `fts::sync` con 10 000 filas (D-11) | < 100 ms (umbral del test) | 2,3 ms (mejor de 5) | 2026-10-31 | Borrado por `task_id` sobre FTS independiente; no hace falta cambiar el diseño de D-11 |

## Discrepancias detectadas
- D-01 pide `set_effects` de Tauri, pero este no informa de errores y no permite el fallback real; se usa `window-vibrancy` directo. Registrado como D-16 (`PROPOSED`, pendiente de confirmación humana).
- (resuelta en F0-09) El perfil `release` de la plantilla difería de SPEC §10.1; ahora usa `lto = "thin"`, `strip = "symbols"`, `panic = "unwind"`.

## Cierre de la Versión 1.0.0
- **Fecha:** 2026-11-09 · **Etiqueta:** `v1.0.0` · **Instaladores:** `MiAgendaApp_1.0.0_x64-setup.exe` (NSIS, recomendado) y `MiAgendaApp_1.0.0_x64_es-ES.msi`.
- **Fases 0 a 6:** todas las tareas `[A]` están hechas y verificadas con la batería de CLAUDE.md §4. Mejoras posteriores a F6, ya incluidas: recordatorios periódicos (D-32), duración y detalle de tarea (D-33), etiquetas con filtro y arrastre (D-34) e icono propio.
- **Cierre por decisión del usuario:** la V1 se da por finalizada sin que se hayan validado los ítems `[H]` ni ejecutado F6-06. Siguen sin marcar a propósito, porque ningún humano los ha comprobado en Windows 11 (CLAUDE.md §7).
- **Pendientes conocidos (no bloquean el uso):**
  - Ítems `[H]` sin validar en `docs/manual-pruebas.md`, en especial los de apariencia, arrastre y etiquetas.
  - F6-06: los E2E están escritos pero no se han ejecutado (`npm run e2e`).
  - Firma de los instaladores (avisará SmartScreen al instalar).
  - Etiquetas anidadas (`parentId`) sin interfaz; el filtro de etiquetas no se guarda entre sesiones (D-34).
- **Actualizar a una versión posterior:** subir la versión, `npx tauri build` y abrir el `.exe` nuevo sobre el instalado; los datos se conservan.

## Notas de sesión
- 2026-10-31 (F1-03, para F4): al mover una tarea a la papelera sus `reminders` no se borran (se restauran con ella). El scheduler (F4-02/F4-03) debe ignorar los recordatorios cuyas tareas tengan `deleted_at` y las consultas de recuperación (F4-04) también.
- 2026-10-31 (F1-03): `TagWithCount` se serializa como `Tag` más `pendingCount` (SPEC §7.3 no define el campo); F1-09 debe reflejarlo en `api.ts`.
- 2026-10-31 (F1-01): SPEC §5.3 menciona la "ruta del backup" cuando falla una migración; la copia previa a migrar depende de `db/backup.rs` (F6-04) y aún no existe. Hasta entonces el fallo deja la base intacta (rollback) y el error indica la versión que falló.
- 2026-10-31: el entorno real es Windows 11, no Linux (CLAUDE.md §8). El terminal con sandbox (WSL) no arranca por falta de `libasound.so.2`; los comandos se ejecutan fuera del sandbox con aprobación del usuario. Por tanto `cargo check --target x86_64-pc-windows-msvc` es posible en este equipo cuando toque (F0-07).
- 2026-10-31: SPEC §8.3 no define los tokens claros cuando `data-theme="light"` y el SO está en oscuro (el bloque claro solo vive dentro de `prefers-color-scheme: light`). `tokens.css` añade el bloque `:root[data-theme="light"]` para que el selector manual de tema (F6-05) funcione. También añade `--bg-window-native`, `--bg-window-solid` (valores de SPEC §8.2, ahora por tema), `--scrollbar-thumb` y `--tag-1`…`--tag-8` (colores propuestos por el agente; su contraste se revisa en F6-02).
- 2026-10-31: hasta F0-06 la ventana no tiene barra de título ni botones (`decorations: false`); para cerrarla en `tauri dev` usar Alt+F4 o detener el proceso.
- 2026-11-09: mejoras posteriores a F6 (D-33): vista de detalle desde el calendario, edición con el diálogo de nueva tarea, borrado desde el detalle, duración de tareas (migración 003), tareas hechas sin recordatorios y lista con las pendientes más destacadas. Verificado `[A]`: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (260), `npm run lint`, `npm run check`, `npm run test` (306). **No** se ejecutaron `npm run build` ni `npm run size` (el usuario genera los instaladores con `build-installers.bat`). Pendientes `[H]` en `docs/manual-pruebas.md` § «Detalle de tarea y duración».
- 2026-11-09: etiquetas de tareas (D-34): barra lateral con filtro por casillas, selector de etiqueta en el diálogo, chips en la lista y eventos de color en el calendario. Sin migración. Batería `[A]` en verde (Vitest 332+, `cargo test` 260 de lib); la apariencia real es `[H]` (manual-pruebas § Etiquetas).
- 2026-11-09: **Versión 1.0.0 lanzada.** Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. Instaladores generados con `npx tauri build`: `MiAgendaApp_1.0.0_x64-setup.exe` (NSIS, 3,2 MiB, instalación por usuario; el recomendado) y `MiAgendaApp_1.0.0_x64_es-ES.msi` (4,5 MiB). Para actualizar basta ejecutar el instalador de una versión mayor sobre la instalada; los datos (`%APPDATA%\com.cristian.miagendaap`) se conservan y las migraciones se aplican al abrir. No mezclar NSIS y MSI en el mismo equipo. Base de datos de desarrollo borrada para una instalación limpia. La validación de los ítems `[H]` sigue pendiente.
- 2026-10-31: el identifier heredado de F0-01 es `com.cristian.miagendaap` (una `p` menos que el nombre del proyecto). Se mantiene y el test lo fija; cambiarlo después mueve el directorio de datos de la app y la identidad de notificaciones. Decidir antes de distribuir (pendiente para el humano).
- 2026-10-31: `PROGRESS.md` tenía F0-01 sin marcar aunque estaba commiteada (`c62a00e`); marcada en la sesión de F0-02.
- 2026-11-09: cambios 1.0.1 (D-35): hora por defecto de avisos en Ajustes, aviso 1 semana antes, avisos por días/semanas para tareas sin hora (sin colocarlas en el calendario), botón guardar de la edición de etiquetas contenido en su cuadro y alineación de columnas de la semana con la cabecera fija (compensa el ancho del scrollbar). Las tareas sin hora ya aparecían en «Todo el día». Batería `[A]` en verde (cargo fmt/clippy/test 263; npm lint/check/test 340/build/size 187 KB). Pendiente `[H]`: manual-pruebas § «Versión 1.0.1».
- 2026-11-09: tema claro opaco (D-36): sin Mica/Acrílico en claro, fondos opacos y borde más marcado; se reaplica al cambiar el tema del sistema. Batería `[A]`: lint, check, Vitest 341, build y size en verde; Rust sin cambios. Apariencia real `[H]`.
- 2026-11-09: **Versión 1.0.1 lanzada** (D-35, D-36). Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. El usuario revisó la interfaz con `tauri dev` (tema claro opaco, hora de aviso por defecto, semana alineada). Instaladores con `npx tauri build`. Los ítems `[H]` de la 1.0.1 siguen sin marcar en `docs/manual-pruebas.md` hasta su validación formal.
- 2026-11-09: **Versión 1.0.2 lanzada** (D-37): casillas (subtareas) en las notas de las tareas, marcables con un clic sin editar el texto. Batería `[A]` en verde (`cargo fmt/clippy/test` 264 de lib; `npm lint/check/test` 342/build/size 187,3 KB). Pendiente `[H]`: manual-pruebas § «Versión 1.0.2». Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y lockfiles; instaladores con `npx tauri build`.
- 2026-11-09: **Versión 1.0.3 lanzada.** Corrección: al teclear la fecha, el navegador notifica años parciales (`0002`, `0020`, `0202`) y la app los confirmaba, dejando la tarea en el año 2 y rompiendo la escritura. `isSettledDate` (`dates.ts`) ignora esos valores en el diálogo de tarea y en la fila. Batería `[A]` en verde (Vitest 346, build, size, Rust 264). Nota: `NoteEditor.test.ts` «loads TipTap lazily» falló una vez por tiempo de carga con la máquina ocupada y pasó en las dos ejecuciones siguientes. Pendiente `[H]`: manual-pruebas § «Versión 1.0.3».
