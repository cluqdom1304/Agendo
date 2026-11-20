# PROGRESS.md

**Fase activa:** versión 3.0.0, Fase 19 (Fase 18 hecha; la 2.1.0 no se publica); Fases 16 y 17 con sus tareas `[A]` hechas, falta la validación humana F17-10; la 2.0.3 es la última publicada
**Estado:** IN_PROGRESS   <!-- IN_PROGRESS | AWAITING_HUMAN | BLOCKED | DONE -->
**Siguiente tarea:** Fase 18 terminada (`[A]` en verde, sin ítems `[H]`; F17-10 validada por el usuario); el usuario decide si empieza la Fase 19 (F19-01). No se inicia ninguna fase sin confirmación. Pendiente de validación humana en Windows 11: los ítems `[H]` de F8 a F16 y la primera ejecución de `npm run e2e`.
**Última actualización:** 2026-11-20 (Fase 18 completada: cimientos de datos, migraciones 015 a 018)

> Reglas: marcar `[x]` solo al pasar la batería de verificación (CLAUDE.md §4). Los ítems `[H]` nunca se marcan como verificados por el agente.

---


## Índice de versiones
| Versión | Estado | Fases | Contenido |
|---------|--------|-------|-----------|
| 1.0.x | publicada (1.0.0 a 1.0.7) | 0–6 | Agenda con recordatorios, calendario, etiquetas, copia de seguridad |
| 2.0.x | publicada (2.0.0 a 2.0.3) | 7–15 (+11B) | Notas con carpetas, editor, adjuntos, historial, enlaces, recurrencia, interoperabilidad; 2.0.1–2.0.3 corrigen menús cortados |
| 3.0.0 | **planificada** (la 2.1.0 no se publica; incluye lo hecho en 16–17) | 16–27 | Rendimiento, mejoras de lo existente, rediseño tipo «Woday», prioridades, eventos, hábitos, lenguaje natural, planificar el día, enfoque y resumen semanal |
| Anexos | — | — | Métricas, discrepancias, cierre de 1.0.0 y registro cronológico de sesión |

---

# Versión 1.0.0 — Agenda (publicada; 1.0.1 a 1.0.7 son mejoras posteriores)

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


---

# Versión 2.0.0 — Notas + agenda unificadas (publicada; 2.0.1 a 2.0.3 son correcciones)

> **Estado:** EN CURSO. El usuario aprobó el 2026-11-12 todo el plan: D-43 a D-49 `ACCEPTED`, una sola carpeta por nota, varias etiquetas por tarea (D-44), recurrencia, acción de notificación e importación ENEX dentro de 2.0.0. Orden de recorte si hace falta: F14-04, F13-04/05/06, F12-03 (F7–F11 son el mínimo).
> **Principio:** una sola tabla `tags` para tareas y notas; una tarea vive solo en `tasks` y las notas la referencian; una sola búsqueda, papelera y copia de seguridad.

## Fase 7 — Fundamentos: modelo de datos y contrato
- [x] F7-01 ADR D-43…D-49 aprobados (o ajustados) por el humano
- [x] F7-02 Migración `004_notes.sql`: `folders`, `notes`, `note_tags`, `note_tasks`, `note_links`, `notes_fts`
- [x] F7-03 Migración `005_task_source.sql`: `tasks.source_note_id` (`ON DELETE SET NULL`)
- [x] F7-04 `repo/folders.rs`, `repo/notes.rs`, `repo/note_tags.rs` con FTS en la misma transacción
- [x] F7-05 `domain/`: validación de carpetas (profundidad, ciclos al mover), título de nota, extracción de `content_text`, `taskRef` y `[[enlaces]]`
- [x] F7-06 Comandos `folder_*`, `note_*`, `note_tags_set`, `note_move`; papelera y restaurar con notas; DTOs y fixtures de contrato
- [x] F7-07 Test de 10 000 operaciones aleatorias con notas, carpetas, etiquetas y referencias (`integrity_check`, claves foráneas, FTS)
- [x] F7-08 Test de migración desde una copia v1.0.x (los datos se conservan)

**Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (320 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (352), `npm run build` y `npm run size` (188,2 KB gzip de 250) en verde. `cargo check --target x86_64-pc-windows-msvc` queda cubierto: el equipo es Windows y clippy/test compilan la ruta nativa.

**Notas de la fase:**
- F7-02/03: `004_notes.sql` (`folders`, `notes`, `note_tags`, `note_tasks`, `note_links`, `notes_fts`) y `005_task_source.sql` (`tasks.source_note_id`, `ON DELETE SET NULL`). Solo añaden tablas y una columna nula: no tocan datos. Se añadió `note_tasks` (nota → tareas que referencia), que la hoja de ruta no listaba: sin ella, saber qué notas referencian una tarea (F10-06) obligaría a recorrer todos los JSON. Dos carpetas hermanas no pueden llamarse igual (índice único sin distinguir mayúsculas). Borrar una carpeta con `DELETE` deja las notas en la Bandeja y asciende a las hijas (`SET NULL`); el borrado normal de la aplicación es `folders::delete_tree`. `db/notes_schema_tests.rs` (10 tests) cubre tablas, índices, `CHECK`, cascadas, unicidad y FTS.
- F7-04: `repo/folders.rs` (árbol con CTE recursivas: `depth`, `height`, `is_in_subtree`, `delete_tree`), `repo/notes.rs` (`list` por ámbito `Todas|Bandeja|Fijadas|Carpeta` —la carpeta incluye subcarpetas—, etiqueta y orden; fijadas siempre primero) y `repo/note_tags.rs`. `fts.rs` gana `insert_note`/`sync_note`/`rebuild_notes`. Las referencias de un documento (`note_tasks`, `note_links`) se recalculan en cada guardado y se **omiten** si el destino ya no existe o es la propia nota, en vez de fallar: un `taskRef` puede apuntar legítimamente a una tarea purgada. Fijar una nota no cambia `updated_at`; moverla sí. La lista devuelve un extracto de 200 caracteres, no el documento.
- F7-05: `domain/validation.rs` (`validate_folder_name` 1–100, `validate_folder_placement`: máx. 5 niveles y sin ciclos, D-43) y `domain/content.rs` (`references_of`: nodos `taskRef` con `attrs.taskId` y `noteLink` con `attrs.noteId`, sin repetidos y en orden; una referencia sin id se rechaza). **Fijan el formato de esos dos nodos para F10/F11.**
- F7-06: comandos `folder_list`, `folder_upsert`, `folder_delete`, `notes_list`, `note_get`, `note_create`, `note_update`, `note_set_pinned`, `note_move`, `note_tags_set`, `note_delete`, `note_restore`, `notes_trash_list`; `task_create` acepta `sourceNoteId` y `Task` gana `sourceNoteId`. `trash_list` sigue devolviendo solo tareas (cambiarlo rompería la interfaz actual): la papelera unificada es de F8. `AppState::purge_trash` purga también notas y devuelve el total. DTO nuevos en `fixtures/dto.json`, `src/lib/types.ts`, `api.ts` y `types.test.ts`.
- F7-07: segundo test de 10 000 operaciones (`random_ops_tests.rs`) con tareas, notas, carpetas (crear, mover, borrar con las mismas reglas que el comando), etiquetas compartidas y referencias; comprueba `integrity_check`, `foreign_key_check`, `notes_fts` = notas vivas sin filas obsoletas, sin autoenlaces y árbol sin ciclos ni más de 5 niveles. El evento `notes:changed` se movió a F8-08 (ver arriba).
- F7-08: `upgrading_a_v1_database_keeps_every_row` migra una base v3 con tareas, etiquetas, avisos y ajustes hasta la v5 sin perder filas. La restauración de una copia antigua ya migraba sola (D-30) y su test sigue en verde.
- `task_create` lleva `#[allow(clippy::too_many_arguments)]`: el contrato IPC pasa cada campo como argumento propio (SPEC §7.3).

**Ítems [H]:** ninguno (la fase no tiene interfaz). Nada nuevo en `docs/manual-pruebas.md`.

## Fase 8 — Sección Notas: interfaz esencial
- [x] F8-01 Navegación principal Agenda | Notas (`Ctrl+Mayús+1/2`: `Ctrl+1…4` ya eran las vistas de la agenda)
- [x] F8-02 Panel de carpetas: árbol, crear/renombrar/mover (puntero propio, como D-34), Bandeja de entrada, Todas, Fijadas, Papelera
- [x] F8-03 Lista de notas virtualizada (título, extracto, fecha, etiquetas; orden configurable)
- [x] F8-04 Editor de nota con TipTap (D-22), título propio y guardado diferido
- [x] F8-05 Fijar, duplicar, mover, eliminar/restaurar con deshacer
- [x] F8-06 Textos en `i18n.ts`, estados vacíos, teclado y ARIA (N-08)
- [x] F8-07 Recordar carpeta y nota abiertas al reiniciar
- [ ] F8-08 (innecesaria hasta ahora: ver notas de la F10) Evento `notes:changed` (SPEC §7.4) y su escucha en la interfaz; se pospuso desde F7-07 porque hoy nada lo consume (tampoco existe `tasks:changed`)

**Verificaciones [A] de F8:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (321 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (385+), `npm run build` y `npm run size` (196,1 KB gzip de 250) en verde.

**Notas de la fase 8:**
- Sección y estado: `viewStore.section` (`agenda`|`notes`) y ajustes nuevos `app_section`, `notes_scope` y `notes_open` (el segundo y el tercero aceptan texto libre de hasta 64 caracteres sin control). Al arrancar se restaura la sección, la carpeta y la nota; si ya no existen se vuelve a «Todas».
- `stores/notes.svelte.ts` (carpetas, lista por ámbito, nota abierta, deshacer, avisos), `FolderSidebar` (árbol, Bandeja, Fijadas, Papelera; crear, renombrar, eliminar con confirmación; mover por arrastre con puntero propio `folderDrag`), `NoteList` (virtualizada, 78 px por fila), `NotePane` (título + `NoteEditor` variante `page`, fijar, duplicar, mover, eliminar) y `NotesView`. Las notas se cargan al entrar por primera vez en la sección (N-01).
- Duplicar no necesita comando nuevo: crea con el contenido de la original.
- **F8-08 pospuesta otra vez:** sigue sin haber ningún consumidor del evento (una sola ventana, y cada acción ya recarga lo suyo). Propuesta: hacerla en F10, cuando crear tareas desde notas obligue a refrescar ambas listas.
- Atajo de sección `Ctrl+Mayús+1/2` por `event.code`, porque Mayús cambia `event.key`.
- Cada nota guarda con retardo (400 ms) y se vuelca al cerrar o cambiar de nota.

## Fase 9 — Etiquetas unificadas
- [x] F9-01 Varias etiquetas por tarea (D-44): selector, chips, calendario
- [x] F9-02 Etiquetas en notas (editor y lista)
- [x] F9-03 Barra lateral común con contadores de tareas y notas; vista de etiqueta con pestañas Tareas / Notas
- [x] F9-04 Renombrar, recolorear, fusionar y subetiquetas con interfaz (D-06)
- [x] F9-05 Autocompletado de etiquetas con `#` en el editor

**Verificaciones [A] de F9:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (325 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (399), `npm run build` y `npm run size` (199,9 KB gzip de 250) en verde.

**Notas de la fase 9:**
- F9-01 (D-44): una tarea admite varias etiquetas. `TagsStore.tagsOf` y `isVisible` (se ve si alguna etiqueta suya está marcada; sin etiqueta sigue a «Sin etiqueta»); `TasksStore.setTags` / `addTag` (soltar sobre una etiqueta de la barra **añade**, soltar sobre «Sin etiqueta» las quita todas); `TagPicker` y `TagMenu` pasan a multiselección (el menú de la fila no se cierra al marcar). La fila muestra dos chips y `+N`; el calendario usa el color de la primera y `+N` en el nombre; el detalle lista todas. Cada tarea cuenta bajo todas sus etiquetas en la barra.
- Backend: `TagWithCount.noteCount` (notas vivas con la etiqueta; cambia `fixtures/dto.json`) y comando `tag_merge` (`repo::tags::merge`: mueve vínculos de tareas y notas sin duplicar y borra la etiqueta origen; sus subetiquetas quedan en el nivel superior). 3 tests de repo y 1 de comando.
- F9-02: `NotePane` muestra las etiquetas de la nota y un menú para asignarlas (misma `TagMenu` que las tareas); la lista ya mostraba dos chips. Las etiquetas de la nota se guardan con `note_tags_set`.
- F9-03: `FolderSidebar` gana la sección «Etiquetas» (contador de notas; el título del botón añade las tareas pendientes) y el ámbito `tag:<id>`. En la vista de una etiqueta hay pestañas **Notas / Tareas**; la pestaña Tareas lista las tareas con esa etiqueta y al pulsar una abre la agenda y la muestra en la lista. Crear una nota dentro de una etiqueta le pone esa etiqueta. Los contadores se refrescan al cambiar las notas.
- F9-04 (D-06): `TagEditor` suma «Dentro de» (subetiqueta de un solo nivel; bloqueado si ya tiene subetiquetas) y «Fusionar con» con confirmación; la barra de la agenda muestra las subetiquetas sangradas bajo su etiqueta (`TagsStore.ordered`). Marcar una etiqueta no marca sus subetiquetas.
- F9-05: escribir `#` al principio de línea o tras un espacio abre una lista de etiquetas (sin distinguir mayúsculas ni tildes; flechas, Intro/Tab y Esc). Elegir una **asigna la etiqueta a la nota** y borra el `#texto`; no inserta un nodo en el documento ni añade dependencias. Solo en el cuerpo de las notas, no en las descripciones de tareas.

## Fase 10 — Tareas dentro de las notas
- [x] F10-01 Nodo `taskRef` en TipTap: casilla, título, fecha y etiqueta; marcar completa la tarea real
- [x] F10-02 Crear tarea desde una nota (`/tarea`, `Ctrl+Shift+T`) con mini-diálogo de fecha, hora y aviso
- [x] F10-03 «Convertir casillas en tareas» para las listas de D-37
- [x] F10-04 Panel «Tareas de esta nota» con progreso
- [x] F10-05 Chip «Nota de origen» en la lista y el detalle de tarea
- [x] F10-06 Sincronía editar/completar/borrar entre tarea y notas; tarea borrada → referencia tachada con «Restaurar»
- [x] F10-07 «Convertir en nota» y «Crear nota para esta tarea»

**Verificaciones [A] de F10:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (325 de lib + 4 + 1 + 4, sin cambios de backend), `npm run lint`, `npm run check` (0/0), `npm run test` (418), `npm run build` y `npm run size` (205,5 KB gzip de 250) en verde.

**Notas de la fase 10 (D-45):**
- F10-01: nodo `taskRef` de TipTap (`features/notes/taskRef.ts`), bloque atómico que solo guarda `attrs.taskId` (el formato que ya entendía el backend desde F7). Se dibuja con DOM propio y lee la tarea viva a través de un `TaskRefHost` (`taskHost.ts`): casilla (completa la tarea real con la misma lógica que la lista), título (abre el detalle), fecha y hora, y etiquetas. Solo existe en las notas, no en las descripciones de tareas.
- F10-02: `/tarea` + espacio, `Ctrl+Mayús+T` o el botón de la barra abren `NoteTaskDialog` (título, fecha, hora, aviso y etiquetas, que arrancan con las de la nota; el texto seleccionado rellena el título). La tarea se crea con `sourceNoteId` y la referencia se inserta tras el cursor.
- F10-03: «Convertir casillas en tareas» crea una tarea por casilla (las marcadas quedan hechas) y sustituye las listas por sus referencias, en una sola transacción del editor. Si la nota se edita mientras tanto no toca nada (las tareas quedan creadas y aparecen en el panel).
- F10-04: `NoteTasksPanel` bajo el texto: tareas referenciadas + creadas desde la nota, con «n de m» y barra de progreso; casilla y título funcionan como en el resto.
- F10-05: chip de la nota de origen en la fila de la tarea y fila «Nota de origen» en el detalle; abre la nota en la sección Notas (`navigation.ts`). Los títulos se resuelven y cachean (`NotesStore.titles`, `ensureTitles`).
- F10-06: no hace falta evento: tareas y notas viven en la misma ventana y los mismos stores, así que editar o completar se ve a la vez en todas las referencias (el efecto de `NoteEditor` las redibuja). Una tarea eliminada sale tachada con «Restaurar» (`restoreById`); si ya se purgó, avisa. **Por eso F8-08 (`notes:changed`) sigue sin hacerse y propongo retirarla.**
- F10-07: en el detalle de la tarea, «Crear nota para esta tarea» (nota con título y una referencia) y «Convertir en nota» (nota con su descripción y etiquetas; la tarea va a la papelera con el aviso de deshacer de la agenda).
- Bug cazado por los tests: `restoreById` no volvía a meter la tarea en la lista (`#replace` solo sustituye).
- `NotesStore.load` comparte una sola carga entre llamadas simultáneas, porque abrir una nota de origen desde la agenda puede cruzarse con la carga inicial.

### Repaso de la sección Notas (fallos y línea de diseño, 2026-11-12)
A petición del usuario, sin tarea nueva de la hoja de ruta: corregir fallos de F8–F10 y unificar el aspecto de Notas con el resto de la app (ligero, estilo Apple).

**Fallos corregidos:**
- Las notas de la papelera no se podían abrir: `note_get` solo devuelve notas vivas, así que pulsar una dejaba el panel vacío. Comando nuevo de solo lectura `note_trash_get` (`repo::notes::get_trashed`, sin DTO nuevo) y `NotesStore.openInTrash`: la nota se muestra **sin poder editarla** (antes solo se bloqueaba el título) con «Restaurar»; al restaurarla sigue abierta y ya editable; al salir de la papelera se cierra.
- Con la papelera a la vista, una nota viva abierta se trataba como eliminada (título bloqueado y un «Restaurar» que fallaba): «en la papelera» dependía del ámbito, no de la nota.
- Soltar una nota sobre «Carpetas» o una carpeta sobre la Bandeja llamaba al backend con un id inventado («No se pudo completar la acción»). Ahora no hacen nada ni se iluminan como destino.
- La barra de formato fija tapaba el texto con una franja de otro color sobre el fondo translúcido (Mica): ahora vive en la cabecera de la nota, fuera de la zona que se desplaza.
- El contador de una etiqueta desaparecía al pasar el ratón (regla pensada para las carpetas, que muestran botones en su lugar).
- En la lista, las etiquetas de una fila se cortaban por abajo (la fila medía menos que su contenido).
- El menú de etiquetas de la nota se abría hacia fuera del panel y quedaba recortado.
- `Ctrl+N` creaba una nota aunque hubiera un diálogo abierto encima.
- El texto de la nota se anunciaba como «Notas de la tarea» al lector de pantalla; ahora «Texto de la nota».
- El punto de «filtro activo» y el nombre del botón de la barra lateral hablaban de las etiquetas de la agenda también en Notas.
- Un fallo al recargar la lista ya no deja una promesa rechazada sin tratar (cambiar de carpeta u orden) ni hace reintentar un guardado que sí se hizo.

**Diseño y uso:**
- `shell/Menu.svelte`: menú emergente común (teclado, Esc, clic fuera, vuelta del foco) para «Ordenar por», «Mover a…» y «Más acciones». Sustituye al `<select>` nativo y al menú propio de mover; «Mover a…» lista también las subcarpetas, sangradas (`NotesStore.tree`).
- Cabecera de la nota: formato a la izquierda (iconos SVG en vez de letras y emoji; también en las descripciones de las tareas) y a la derecha solo nueva tarea, fijar, mover y «···» (duplicar, convertir casillas, eliminar). En un panel estrecho se ocultan deshacer/rehacer. El campo de enlace flota bajo la barra.
- Nota: fecha completa de la última edición centrada sobre el título, columna de lectura de 760 px como máximo, el texto ocupa todo el alto (clic en cualquier parte), Intro o flecha abajo en el título pasan al texto. «Tareas de esta nota» solo aparece si hay tareas y es un pie con una línea, no una caja.
- Lista: selección con un tinte del color de acento (antes un bloque sólido), fila de 80 px (antes 78) con la fecha delante del extracto, dos líneas de extracto si la nota no tiene etiquetas y `+N` si tiene más de dos. En una etiqueta, las pestañas Notas/Tareas van bajo la cabecera, que ya no desaparece en «Tareas».
- Barra de carpetas: subetiquetas sangradas como en la agenda, contadores a cero atenuados, confirmación de borrado con los botones comunes, la columna de la lista se estrecha en ventanas pequeñas.

**Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (326 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (434 en 36 archivos), `npm run build` y `npm run size` (207,1 KB gzip de 250) en verde. Sin dependencias nuevas. `cargo check --target x86_64-pc-windows-msvc` no aplica: no se toca `window/` ni `notify/`.

**Discrepancia anotada:** `note_trash_get` es un comando añadido al contrato de F7-06 (lectura, sin migración). Si se prefiere que `note_get` devuelva también notas eliminadas, es un cambio pequeño; se eligió un comando aparte para que una nota eliminada no vuelva a aparecer como «nota de origen» de una tarea.

## Fase 11 — Enlaces, búsqueda y navegación rápida
- [x] F11-01 Enlaces `[[nota]]` con autocompletado, creación al vuelo y renombrado seguro — nodo `noteLink` (guarda solo el id; el título se lee de la nota, así que renombrar no rompe nada), lista al escribir `[[`, «Crear la nota «x»» si no existe, enlace tachado «Nota eliminada» si está en la papelera
- [x] F11-02 Panel de backlinks y notas huérfanas — pie «Enlaces a esta nota» (`note_backlinks`) y carpeta inteligente «Notas sin enlaces» (`notes_orphans`)
- [x] F11-03 Búsqueda unificada (tareas, notas, etiquetas) con filtros `tag:`, `in:`, `is:tarea` — `search_all` (`repo/find.rs`): `tag:` acepta subetiquetas y se puede repetir, `in:` es una carpeta (con subcarpetas), `is:` admite tarea, nota, etiqueta, hecha, pendiente y fijada; valores con espacios entre comillas
- [x] F11-04 Paleta de comandos `Ctrl+K` — sustituye al buscador de la agenda y funciona en ambas secciones: busca en todo, `>` para solo comandos (nueva tarea/nota, cambiar de sección o vista, carpetas inteligentes de notas, mostrar todas las etiquetas)
- [x] F11-05 Notas recientes e «Ir a…» (`Ctrl+P`) — las 8 últimas notas abiertas se guardan en el ajuste `notes_recent` (límite de 480 caracteres, el resto de ajustes libres sigue en 64)


### Resultado de la Fase 11
- **Decisiones de interpretación (sin ADR, no tocan la especificación):** en `in:` se eligió la carpeta (lo habitual en un cuaderno de notas) y no una zona de la nota; `is:` completa el filtrado por tipo y estado. El buscador solo de tareas (`search`) se mantiene en el backend, pero la interfaz usa ya `search_all`.
- **Sustitución:** `Spotlight.svelte` y sus tests se retiran; su lógica (respuestas fuera de orden, teclado, foco) pasa a `features/palette/Palette.svelte` con tests equivalentes. Los dos tests de `TodoView` que abrían el buscador se sustituyen por los de la paleta.
- **Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (335 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (468 en 40 archivos), `npm run build` y `npm run size` (211,7 KB gzip de 250) en verde. Sin dependencias nuevas.
- **Pendiente de la V2:** F8-08 (`notes:changed`) sigue sin hacerse (innecesaria con una sola ventana); propuesta de retirarla.

## Fase 11B — Menús contextuales y botón flotante de nueva tarea (añadida a petición del usuario, D-50)
- [x] F11B-01 Componente único de menú contextual (clic derecho y tecla Menú / Mayús+F10): posición sin salirse de la ventana, flechas, Esc, submenús (etiquetas, mover a, fecha), reutilizando `shell/Menu.svelte`; el menú del navegador queda desactivado salvo en campos de texto
- [x] F11B-02 Tareas (lista, calendario, detalle): hecha/reabrir, editar, fecha de hoy/mañana/elegir, aviso, etiquetas (marcar varias), duplicar, crear nota para la tarea, convertir en nota, borrar con Deshacer
- [x] F11B-03 Notas (lista y cabecera): abrir, fijar, mover a…, etiquetas, duplicar, nueva tarea desde la nota, copiar enlace `[[…]]`, borrar; en la papelera: restaurar. Carpetas: nueva nota aquí, subcarpeta, renombrar, borrar. Etiquetas (barras laterales): renombrar, color, subetiqueta, fusionar, borrar
- [x] F11B-04 Fondo de cada sección: Agenda → nueva tarea (con la fecha del día pulsado en el calendario); Notas → nueva nota (en la carpeta actual). Editor: cortar/copiar/pegar, formato y enlace; clic derecho sobre un enlace `[[…]]` o una referencia de tarea → abrir, quitar
- [x] F11B-05 Botón «Nueva tarea» flotante en la esquina inferior derecha de la Agenda (redondo, sombra, sin tapar contenido ni el aviso de Deshacer, `Ctrl+N` intacto, desaparece en Notas); se retira de la barra superior
- [x] F11B-06 Tests de cada menú (acciones disponibles según el elemento, accesibilidad por teclado) y `[H]` en `docs/manual-pruebas.md`

### Resultado de la Fase 11B
- **Diseño (D-50, aceptado):** un único menú dibujado por la aplicación (`features/context/`). Los componentes solo llevan atributos `data-*` (`data-task-id`, `data-note-id`, `data-folder-id`, `data-tag-id`, `data-note-tag-id`, `data-menu-area`, `data-date`); `target.ts` resuelve el elemento señalado y `entries.ts` construye las acciones como función pura de los stores, de modo que las casillas y las etiquetas se actualizan solas. El menú del navegador solo se conserva en campos de texto; el resto de clics derechos lo ocultan, también donde no hay nada que ofrecer.
- **Teclado:** tecla Menú o Mayús+F10 sobre el elemento con foco; flechas, Inicio/Fin, → abre el submenú, ← lo cierra, Esc y Tab cierran, el foco vuelve al origen. Los submenús se abren a la derecha o a la izquierda según el espacio.
- **Qué ofrece cada cosa:** tareas (hecha/reabrir, abrir, editar, fecha hoy/mañana/elegir/quitar, aviso, etiquetas con varias casillas, duplicar, nota de origen o crear/convertir en nota, borrar con Deshacer); notas (abrir, fijar, mover a…, etiquetas, duplicar, borrar; en la papelera, restaurar); carpetas (nueva nota aquí, subcarpeta, renombrar, eliminar); etiquetas de la Agenda (mostrar solo esta, mostrar/ocultar, editar, fusionar) y de Notas (ver notas, nueva nota con la etiqueta, fusionar); fondo de la Agenda (nueva tarea en el día pulsado, ir a ese día, ir a hoy, vista) y de Notas (nueva nota); editor (cortar, copiar, pegar, seleccionar todo) y, sobre un enlace `[[…]]` o una referencia de tarea, abrir y quitar.
- **Desviaciones de lo planeado (anotadas, sin ADR porque no tocan la especificación):** (1) renombrar, subcarpeta, eliminar carpeta y editar etiqueta no tienen campos propios: el menú pide a la barra lateral que abra los suyos (`contextMenu.intent`), así que la confirmación de borrado y el editor de etiquetas son los de siempre; borrar etiqueta y cambiar color/jerarquía están dentro de «Editar etiqueta…». (2) «Copiar enlace `[[…]]`» y «Nueva tarea desde la nota» no se han añadido: un `[[título]]` pegado no se convertiría en enlace, y la tarea desde la nota ya tiene su botón en la nota. (3) «Formato» del editor no está en el menú (la barra de herramientas y los atajos ya lo cubren). (4) El botón flotante no aparece en Notas.
- **Botón flotante:** círculo de 48 px en la esquina inferior derecha de la Agenda (Lista y Calendario), sobre el contenido y bajo los diálogos y el aviso de Deshacer, que va centrado; se retira de la barra superior. `Ctrl+N` no cambia. En el calendario abre el diálogo en el día de referencia.
- **Verificaciones [A]:** `cargo fmt --check` (Rust sin cambios), `npm run lint`, `npm run check` (0/0), `npm run test` (491 en 42 archivos: 21 del menú, 5 de `target`, intents de ambas barras laterales), `npm run build` y `npm run size` (215,7 KB gzip de 250) en verde. Sin dependencias nuevas.

## Fase 12 — Editor avanzado y adjuntos
- [x] F12-01 Almacén de adjuntos con hash, deduplicación y limpieza de huérfanos (D-47). Migración 006 (`attachments`, `note_attachments`), `attachments.rs` (firma de imagen, SHA-256, escritura atómica, limpieza a las 24 h al arrancar), comandos `attachment_add`/`attachment_read` con cuerpo binario, dependencias en D-51. Aviso: F13-05 pasa a ser la migración 007
- [x] F12-02 Imágenes: pegar, arrastrar, elegir desde la barra, redimensionar (esquina o flechas) y vista previa a pantalla completa. Solo PNG/JPEG/GIF/WebP: «vista previa de ficheros» de otro tipo no se hace (nodo `image` propio, `ImageCache`, `ImagePreview`)
- [x] F12-03 Tablas (menú contextual para filas y columnas), bloques de código con resaltado propio, citas, separadores, resaltado de texto y títulos; solo en el cuerpo de las notas (`advanced`)
- [x] F12-04 Atajos Markdown (reglas de entrada de las extensiones: `# `, `> `, ``` ``` ```, `---`, `==texto==`, listas), buscar dentro de la nota (`Ctrl+F`) y contador de palabras
- [x] F12-05 Copia `.agendo` (base de datos + adjuntos); restaurar sigue aceptando `.sqlite` (B-02). `archive.rs`: zip con `agenda.sqlite`, `manifest.json` y `attachments/<sha256>`; se detecta por contenido, no por nombre

### Resultado de la Fase 12
- **Adjuntos (F12-01):** migración 006, almacén `attachments/<sha256>` con escritura atómica, tipo decidido por la firma de los bytes (PNG, JPEG, GIF, WebP; SVG excluido), máximo 20 MB, ids validados antes de tocar rutas, limpieza de huérfanos al arrancar con 24 h de margen (un deshacer del editor no pierde la imagen). Los bytes viajan como cuerpo binario de IPC. Las referencias se sincronizan al guardar la nota (`note_attachments`), como los enlaces.
- **Editor (F12-02 a F12-04):** nodo `image` propio (pegar, soltar, elegir; ancho guardado en el documento; esquina o flechas para redimensionar; clic para vista previa grande), tablas, citas, separadores, títulos de 3 niveles, resaltado, bloques de código con tokenizador propio (D-51: sin `lowlight`), búsqueda en la nota con `Ctrl+F` (sin mayúsculas ni acentos) y contador de palabras y caracteres. Todo ello solo en el cuerpo de las notas (`advanced`): los editores de tarea siguen siendo los de siempre, y solo las notas guardan imágenes (los adjuntos se enlazan a notas). Las filas y columnas de una tabla se cambian con el menú contextual (D-50).
- **Copia `.agendo` (F12-05):** zip con la base de datos y los adjuntos; restaurar valida los nombres de las entradas, limita tamaños al copiar, vuelve a calcular el hash de cada imagen y descarta las que no coincidan o no sean imágenes; la base pasa por las mismas comprobaciones y la misma copia de seguridad previa que un `.sqlite`.
- **Desviaciones (anotadas, sin ADR porque no tocan la especificación):** (1) «vista previa de ficheros»: solo imágenes; otros tipos de fichero no se adjuntan. (2) Los atajos Markdown son las reglas de entrada de las extensiones de TipTap; no se pueden probar con jsdom y quedan en `[H]`. (3) El test de «resaltar» solo comprueba que la orden existe: sin selección real en jsdom no se puede ver la marca.
- **Cambio de tests ajeno a la fase:** `restore_rejects_bad_files…` dañaba 2 KB de la mitad del fichero y, al crecer el esquema, caía en una página libre; ahora daña todo lo posterior a la primera página.
- **Aviso:** F13-05 (tareas recurrentes) pasa a ser la migración 007.
- **Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (349 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (516 en 44 archivos), `npm run build` y `npm run size` (238,8 KB gzip de 250: **queda poco margen** para las fases 13 a 15). Dependencias nuevas, en D-51: `sha2`, `zip`, y las extensiones `table`, `blockquote`, `horizontal-rule`, `highlight`, `code-block` y `heading` de TipTap 3.33.0.

## Fase 13 — Productividad
- [x] F13-01 Nota diaria enlazada a cada día de la agenda
- [x] F13-02 Plantillas de nota editables
- [x] F13-03 Historial de versiones con comparar y restaurar (D-48)
- [x] F13-04 Recordatorios de nota (sin duplicar el motor de avisos)
- [x] F13-05 Tareas recurrentes (`tasks.rrule`, D-07; migración 007)
- [x] F13-06 Acción «Marcar como hecha» en la notificación (B-01)
- [x] F13-07 Vista «Hoy»: tareas del día, nota diaria y notas fijadas

**Notas de la fase 13:**
- **Estado:** IN_PROGRESS. El usuario pidió explícitamente implementar la Fase 13 (confirmación de fin de la F12).
- F13-01: migración `007_daily_note.sql` (`notes.daily_date`, índice único parcial; incluye notas en la papelera, que `note_daily_open` restaura en vez de duplicar). Comando `note_daily_open(date, title)` (busca, restaura o crea en la Bandeja; el título lo pone la interfaz: «Nota del DD/MM/AAAA»). Acceso: botón «Nota del día» en la cabecera del calendario (día ancla) y comando de paleta «Abrir la nota de hoy». `daily_date` no viaja en el DTO `Note`. **Aviso:** las migraciones se numeran por orden de implementación, así que F13-05 ya no será la 007.
- F13-02: migración `008_note_templates.sql` (`note_templates`, nombre único sin distinguir mayúsculas), comandos `template_list`/`template_save`/`template_delete` (crear con documento, renombrar o reemplazar el documento; borrar), DTO `NoteTemplate` en el contrato. «Nueva nota desde plantilla» es el `note_create` de siempre con el documento de la plantilla y su nombre como título. Interfaz: diálogo «Plantillas de nota» (`NoteTemplates.svelte`; usar, renombrar, reemplazar con la nota abierta, eliminar, guardar la nota abierta) desde el menú contextual del fondo de Notas y la paleta. Interpretación de «editables»: renombrar y reemplazar el contenido con otra nota; no hay editor de plantillas aparte (decisión propia, sin ADR: no toca la especificación).
- F13-03: migración `009_note_versions.sql` (`note_versions`, `ON DELETE CASCADE`). `notes::update` guarda como versión el documento que va a reemplazar si cambia, no está vacío y la última instantánea tiene ≥ 10 min (máx. 30 por nota; las más antiguas se descartan); cambiar solo el título no crea versión. Comandos `note_versions_list`, `note_version_get`, `note_version_restore`; restaurar guarda antes el documento actual (siempre, aunque haya una reciente), así que se puede deshacer. Interfaz: «Historial de versiones…» en el menú «Más» de la nota; lista de versiones, comparación por bloques de texto con la nota actual (`versionDiff.ts`, LCS; `+`/`−` además del color) y «Restaurar esta versión» (el editor se reconstruye con `notes.revision`). Antes de abrir se vacían los guardados pendientes de título y texto. Los valores 10 min / 30 son los «espaciadas / máximo N» de D-48 y quedan como constantes en `repo/versions.rs`.
- F13-04: **ADR D-52 `PROPOSED`** (tabla `note_reminders`, migración `010_note_reminders.sql`; motivo: reconstruir `reminders` exigiría `DROP`). `due`/`next_after`/`mark_sent`/`expire_before` cubren tareas y notas, así que el planificador, los reintentos y la caducidad son los mismos; `DueReminder` y `Fired` llevan `taskId` o `noteId`. Comandos `note_reminder_get`/`note_reminder_set` (hora UTC; pasada o nota inexistente se rechaza; `null` quita). Interfaz: «Recordar esta nota…» en el menú «Más» (fecha y hora, por defecto mañana a las 09:00) y aviso en Notas cuando suena. Una nota en la papelera no avisa. Limitación: el aviso dentro de la aplicación solo se ve en la sección Notas; la notificación de Windows sale siempre.
- F13-06 (B-01): los avisos de tarea llevan un botón «Marcar como hecha» (`tauri-plugin-notification` 2.8 admite acciones en escritorio: `register_action_types` + `on_action`, sin dependencias nuevas ni permisos de JS). La notificación lleva `taskId` / `noteId` como datos; `notify::perform_action` (pura y probada) decide: `done` completa esa tarea y emite `tasks:changed {ids}` (SPEC §7.4; la interfaz relee con `tasksStore.refresh()`), un clic (`tap`) muestra la ventana y emite `reminder:open` (la interfaz abre la tarea o la nota). Si la plataforma no dice de qué notificación se trata, no hace nada: nunca adivina qué tarea completar. Los avisos de nota no llevan botón. Lo que Windows informa de un clic depende del toast (según el plugin, solo mientras está en pantalla y no tras pasar al Centro de actividades): **solo verificable a mano (`[H]`)**; si el botón no aparece o no llega el identificador, el resto sigue funcionando como antes.
- F13-07: nueva sección «Hoy» (`Ctrl+Mayús+3`, comando de paleta «Ir a Hoy»; se recuerda como las otras secciones: `app_section` admite `today`). `features/today/TodayView.svelte`: tareas con fecha de hoy (abiertas primero, por hora; casilla para completar; clic abre el detalle), «Abrir la nota de hoy» (la nota diaria de F13-01) y notas fijadas (`notes.pinnedNotes()`, clic abre la nota). Respeta el filtro de etiquetas de la barra lateral y se recalcula al volver el foco a la ventana (por si pasó la medianoche). No incluye tareas vencidas (fuera de lo pedido). El botón «Nueva tarea» flotante no aparece en esta sección.
- F13-05 (B-05, opción 1; el usuario pidió «finalizar lo que no esté» y se tomó como aceptación de la recomendación): `tasks.rrule` ya existía desde la 001, así que **no hay migración** (la 007 prevista en D-07 no hizo falta). `domain/recurrence.rs`: subconjunto de RFC 5545 (`FREQ` diaria/semanal/mensual/anual, `INTERVAL` 1–365, `UNTIL=AAAAMMDD`); `BYMONTHDAY`/`BYMONTH` los pone la aplicación desde la fecha de la tarea para que «cada 31» caiga en el último día de los meses cortos sin derivar al 28 (y el 29 de febrero vuelva el 28 en años comunes). **Fuera:** `COUNT` y `BYDAY` (días de la semana concretos): no se admiten y la regla los rechaza. Al completar una tarea que se repite se crea otra con la fecha siguiente (la primera posterior a la de completarla y a la suya, en su ciclo) con título, notas, etiquetas, hora, duración, nota de origen y avisos relativos; la completada deja de repetirse (reabrirla y completarla otra vez no duplica). Sin fecha no hay repetición (quitar la fecha la termina; mover a «Por hacer» también); cambiar o arrastrar la fecha reancla la regla. Interfaz: bloque «Repetir» en el diálogo de tarea (frecuencia, «cada N», «hasta» opcional) y «Se repite» en el detalle; la lista se relee al completar una tarea repetida. DTO `Task.rrule` en el contrato.
- **Diálogos con scroll horizontal (arreglo pedido por el usuario):** el detalle de tarea (el que se abre al pulsar una tarea de la nota) tenía una fila de 6 botones sin `flex-wrap` y columnas `1fr` que no podían encogerse; ahora los botones saltan de línea, las rejillas usan `minmax(0, 1fr)` y el texto largo se parte. `Modal` oculta además el desbordamiento horizontal (`overflow-x: hidden`) como red de seguridad; `NoteTaskDialog` y el diálogo de recordatorio de nota usan las mismas columnas. **Solo se puede comprobar a mano (`[H]`)**: jsdom no calcula el diseño.
- **Presupuesto:** JS 244,9 KB gzip tras F13-05 (243,5 tras F13-07; queda poco: 5 KB). Antes: 243,5 KB gzip tras F13-07 (242,3 tras F13-06; 242,2 tras F13-04) (241,3 tras F13-03) (239,9 tras F13-02; 238,7 tras F13-01) gzip de 250 tras F13-01; queda poco margen para F13-02…07 (cargar en diferido lo nuevo, D-49).
- Los tests de interfaz fallaron 3 de 517 una vez bajo carga en paralelo y pasaron en la repetición (517/517); no se tocó ninguno.

## Fase 14 — Interoperabilidad
- [x] F14-01 Exportar nota/carpeta a Markdown (con adjuntos) y HTML
- [x] F14-02 Exportar todo a JSON (B-02)
- [x] F14-03 Importar carpeta de Markdown y `.txt`
- [x] F14-04 Importar ENEX de Evernote (cuadernos → carpetas, etiquetas, imágenes, fechas)

**Notas de la fase 14:**
- **Estado:** IN_PROGRESS → AWAITING_HUMAN. El usuario pidió explícitamente implementar la Fase 14 (confirmación de fin de la F13). Decisiones y límites en **D-54** (`ACCEPTED`): sin dependencias nuevas (XML, base64 y MD5 propios), sin migración, sin red.
- Estructura: `domain/markdown.rs` (escribe y lee Markdown, front matter y `.txt`), `domain/html.rs`, `domain/enex.rs` (XML tolerante + ENML → documento), `domain/codec.rs`, `repo/export.rs` (volcado JSON), `exchange.rs` (ficheros y base de datos) y `commands/exchange.rs` (`export_notes`, `export_json`, `import_markdown`, `import_enex`). Interfaz: sección «Exportar e importar» en Ajustes, «Exportar a Markdown/HTML…» en el menú «Más» de la nota y en el clic derecho de una carpeta. Tras importar se recargan etiquetas, tareas y, si estaba cargada, la vista de notas (`onRestored`).
- F14-01: carpeta de destino elegida por el usuario; carpetas de la aplicación = carpetas del disco; imágenes a `adjuntos/` (Markdown, rutas relativas) o dentro del `.html` (`data:`). F14-02: un JSON con todo, papelera incluida; los adjuntos van solo como metadatos. F14-03: ruta de imagen contenida en la carpeta importada; título por front matter, `# H1` o nombre de archivo; fechas del front matter. F14-04: un archivo `.enex` = una carpeta con su nombre (el formato no trae el cuaderno); varios archivos a la vez.
- **Límites conocidos** (también en D-54): se pierden tachado y subrayado; las referencias a tareas/notas se exportan como texto; los PDF de ENEX no se importan; reimportar duplica; el JSON no se reimporta.
- **Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (433 de lib + 4 + 1 + 4, con los nuevos de F14: códecs, Markdown, HTML, ENEX, volcado JSON y exportar/importar con ida y vuelta), `npm run lint`, `npm run check` (0/0), `npm run test` (563 en 51 archivos), `npm run build` y `npm run size` (246,2 KB gzip de 250: quedan 3,8 KB). `cargo check --target x86_64-pc-windows-msvc` no hizo falta aparte: se compila en Windows. Los cuatro `[H]` y el de aspecto están en `docs/manual-pruebas.md`.

## Fase 15 — Cierre de 2.0.0
- [x] F15-01 Rendimiento con 10 000 notas (arranque, lista, búsqueda < 50 ms); registrar en `docs/metricas.md`
- [x] F15-02 Migración real desde una BD v1.0.7 con datos de usuario
- [x] F15-03 E2E ampliados (incluye F6-06 pendiente), manual de pruebas `[H]`, README y capturas
- [x] F15-04 Versión `2.0.0` coherente en los 5 sitios (CLAUDE.md §10) y entrada en `PROGRESS.md`

**Notas de la fase 15:**
- El usuario pidió explícitamente la Fase 15, cerrar lo suelto, generar los instaladores y publicar la 2.0.0 (autoriza push y release, CLAUDE.md §10).
- F15-01: `repo/scale_tests.rs` con 10 000 notas sintéticas: lista completa 31,3 ms, carpeta 1,7 ms, búsqueda 19,9 ms (peor caso), apertura 1,8 ms (build de depuración; los tests fallan por encima de los umbrales). Detalle en `docs/metricas.md`. El rendimiento de la interfaz con miles de notas es `[H]`.
- F15-02: el esquema de la 1.0.7 es la versión 3; el test lo reconstruye con las migraciones 1 a 3, lo rellena (tareas hechas, borradas, con duración, etiquetas, avisos periódicos, ajustes, índice de búsqueda) y lo abre con las diez migraciones: todo intacto, `integrity_check` y claves foráneas limpios. No se probó con la base real de ningún usuario (privada): `[H]`.
- F15-03: `e2e/specs/notes.e2e.mjs` (notas, búsqueda y sección de exportar/importar); **los E2E siguen sin ejecutarse** (necesitan Windows con `tauri-driver`, `[H]`), así que F6-06 sigue pendiente de ejecución aunque ya no de escritura. README actualizado; **sin capturas de pantalla** (el agente no ve la interfaz).
- F15-04: versión 2.0.0 en `package.json`, `Cargo.toml`, `tauri.conf.json`, `package-lock.json` y `Cargo.lock`.
- Cabos sueltos cerrados: D-52 pasa a `ACCEPTED` (el usuario pidió cerrar lo pendiente); retirado un ítem obsoleto del manual (F13-05 ya estaba implementada); el árbol de trabajo queda limpio.
- **Verificaciones [A]:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (436 de lib + 4 + 1 + 4), `npm run lint`, `npm run check` (0/0), `npm run test` (563 en 51 archivos), `npm run build`, `npm run size` (246,4 KB de 250). Instaladores con `npm run tauri -- build`.

**Riesgos:** la copia deja de ser un solo `.sqlite` (F12-05 antes de ofrecer adjuntos); presupuesto N-04 de 250 KB (editor y notas en carga diferida); referencias `taskRef` rotas (F10-06 con test de integridad); rendimiento con miles de notas (lista virtualizada y FTS desde F7/F8).
**Ideas para 2.x:** cifrado de notas, grafo de enlaces, Kanban por etiqueta, captura rápida con atajo global, OCR de imágenes, subpáginas.


## Mantenimiento 2.0.1 a 2.0.3 (publicadas)
- **2.0.1:** el menú contextual ya no se corta contra el borde de la ventana (`placement.ts`, D-50).
- **2.0.2:** Ctrl+N abre el diálogo de tarea, sin subrayado ortográfico tras perder el foco, submenús sin cortes, «Información de la aplicación» y guardado al cerrar.
- **2.0.3:** el menú de etiquetas de la fila de una tarea se coloca respecto a la ventana y se monta bajo `body`; publicada con instaladores NSIS y MSI sin firma. Detalle en «Notas de sesión».

---

# Versión 3.0.0 — Rediseño y productividad (PLANIFICADA; la 2.1.0 no se publica)

> **Estado:** `PLANNED`. Plan aprobado por el usuario el 2026-11-20 (plan completo en la conversación y ADR `PROPOSED` D-91 a D-98 en `DECISIONS.md`). **El código de las Fases 18 a 27 no se toca hasta que el usuario lo ordene.**
> **Decisión del usuario:** no se publica la 2.1.0; se publica directamente la **3.0.0** cuando todo esté hecho. Las Fases 16 y 17 (ya hechas) forman parte de ella; la versión de `package.json` y demás sigue en 2.0.3 hasta F27-02.
> **Objetivo:** que Agendo se parezca en diseño y funcionalidad a la captura de referencia (sidebar oscuro con Hoy/Próximas/Calendario/Notas/Tareas/Eventos, Favoritos, Recientes, Carpetas y Etiquetas; vista Hoy con tira de días, banner de resumen y prioridades; panel «Bandeja de entrada»; barra superior «Buscar o crear»; eventos y hábitos distintos de las tareas), con lenguaje natural, bloques de tiempo, «Planificar mi día», enfoque, progreso de checklist y resumen semanal.
> **Alcance elegido:** eventos **y** hábitos; sidebar oscuro fijo también con tema claro.
> **Principio:** aplicación ligera, rápida y eficiente. Cada función nueva se paga con presupuesto medido; las vistas nuevas se cargan con `import()` y la F18-00 fija la línea base antes de empezar.
> **Presupuestos vigentes (D-56):** JS gzip 300 KB, ejecutable 18 MB, N-01 < 200 ms y N-02 < 40 MB; D-98 (`PROPOSED`) los revisa con la medida real.

## Fase 16 — Rendimiento y base
- [x] F16-01 `[A]` Auditoría del bundle (análisis de tamaños de `vite build`): módulos por peso y qué entra en la carga inicial.
- [x] F16-02 `[A]` Carga diferida (`import()`) de lo que no se usa al arrancar: calendario, ajustes, historial de versiones, plantillas, paleta y diálogos; meta: ≥ 20 KB menos en la carga inicial.
- [x] F16-03 `[A]` Ejecutable: revisar `opt-level` (`s`/`z` frente a `3`), `codegen-units = 1`, `lto = "fat"`, `panic = "abort"` solo si no rompe D-24 y features de dependencias; registrar tamaño antes y después.
- [x] F16-04 `[A]` Comprobar si la agenda está virtualizada (SPEC §4.2: > 200 bloques) y, si no, virtualizarla por ventana; test con 10 000 tareas.
- [x] F16-05 `[A]` Pruebas de escala en build **release** (hoy solo en depuración): 10 000 notas y 10 000 tareas; registrar en `docs/metricas.md`.
- [x] F16-06 `[A]` Script de medida de N-01 (arranque hasta primer pintado) y N-02 (RAM en reposo); la ejecución real es `[H]`.
- [x] F16-07 `[A]` Estabilizar el test intermitente de `NoteEditor` («loads TipTap lazily») sin saltarlo ni subir su tiempo límite a ciegas.
- [x] F16-08 `[A]` Aplicar D-56 (aceptado): actualizar el script de presupuesto (`npm run size`) y la tabla de Métricas (no se edita `docs/SPEC.md`).
- [x] F16-09 `[H]` Medir N-01/N-02 y la fluidez con datos reales en Windows 11; ejecutar por fin `npm run e2e` (F6-06 y F15-03).

## Fase 17 — Mejorar lo que ya existe
- [x] F17-01 `[A]` Recurrencia ampliada (ADR propio: D-32 descartó RRULE completo): días de la semana, «cada N», «hasta fecha» y «N veces»; sin RRULE completo.
- [x] F17-02 `[A]` Notificación con «Posponer» (10 min / 1 h / mañana) junto a «Marcar como hecha»; reutiliza `reminders`, sin tabla nueva si es posible.
- [x] F17-03 `[A]` Selección múltiple en la lista (Mayús/Ctrl + clic): completar, etiquetar, mover de día y borrar en lote, con un único deshacer.
- [x] F17-04 `[A]` Editor: marcas de tachado y subrayado; el import/export de Markdown y ENEX deja de perderlas (cierra una limitación de D-54).
- [x] F17-05 `[A]` Buscar dentro de la nota (Ctrl+F) con resaltado, reutilizando los tokens `--highlight` (D-55).
- [x] F17-06 `[A]` Importación sin duplicados: reconocer notas ya importadas (origen + hash) y omitirlas; migración 011 si hace falta una columna de origen.
- [x] F17-07 `[A]` Etiquetas: interfaz para la jerarquía de un nivel (`parentId` ya existe, D-06); filtrar por la etiqueta padre incluye las hijas.
- [x] F17-08 `[A]` Búsqueda: vistas guardadas (p. ej. `is:pending tag:SMA`) y filtro por rango de fechas.
- [x] F17-09 `[A]` Copia de seguridad automática programada con rotación de las últimas N copias (Ajustes), reutilizando `.agendo` (D-47).
- [x] F17-10 `[H]` Validación visual y de uso de F17-02 a F17-09 en Windows 11.

**Verificaciones [A] de la Fase 17:**
- F17-01 (D-57): `domain/recurrence.rs` gana `BYDAY` (solo semanal) y `COUNT` (1–999, excluyente con `UNTIL`) y `Rule::successor`; `spawn_next_occurrence` guarda la regla con una repetición menos. «Cada N» y «hasta fecha» ya existían desde F13-05. Interfaz: siete botones de día y «Número de veces» en `TaskDialog`; `describeRule` los explica. Tests nuevos: 3 de dominio, 1 de comando (serie con `COUNT=3` y `BYDAY`), 2 de `recurrence.test.ts` y 1 de `TaskDialog.test.ts`. Sin migración.
- F17-02: la notificación de tareas ofrece «Posponer 10 min», «Posponer 1 h» y «Posponer a mañana» (9:00 locales) además de «Marcar como hecha». Posponer añade un recordatorio de hora personalizada (`repo::reminders::insert_custom`) sin tabla ni migración nuevas y sin tocar los existentes; no actúa sobre tareas completadas, borradas o desconocidas. 2 tests nuevos en `notify`. Cuántos botones muestra Windows y su orden es `[H]`.
- F17-03: selección múltiple con Ctrl/Cmd+clic (alternar) y Mayús+clic (rango en el orden de la lista; `store.visibleOrder`); `SelectionBar` con Completar/Reabrir, Etiquetar, Mover al día, Eliminar y Quitar selección; Esc la limpia. La lógica vive en `stores/taskBatch.svelte.ts` (`TaskBatch`) y usa las mismas llamadas IPC que las acciones sueltas, sin comandos nuevos; cada lote deja **un** deshacer (aviso y Ctrl+Z) que revierte todo. Limitación: deshacer «completar» sobre una tarea periódica la reabre pero no borra la siguiente ocurrencia ya creada. 5 tests de store y 2 de componente.
- F17-04 (D-58): marcas `strike` y `underline` en el editor (botones S y U en la barra, Ctrl+U y Ctrl+Mayús+S); Markdown `~~x~~` y `<u>x</u>` (ida y vuelta probada), HTML `<s>`/`<u>`, ENEX `s/strike/del/u/ins` y estilos `text-decoration`. Dependencias `@tiptap/extension-strike` y `-underline` 3.33.0. JS 266,7 KB.
- F17-05: ya estaba hecha antes de la Fase 17 (`NotePane` abre el buscador con Ctrl+F, `noteSearch.ts` resalta con `find-match`/`find-current` sobre los tokens `--highlight` de D-55, y `NotePane.test.ts` y `advanced.test.ts` la cubren: ignora mayúsculas y acentos, recorre y da la vuelta). No se tocó código; se verificó que los tests siguen en verde (596).
- F17-06 (D-59): migración 011 (`notes.import_origin`, `import_hash`, aditiva); las importaciones de Markdown/texto y ENEX omiten las notas vivas con el mismo origen y la misma huella y las cuentan en `ImportReport.duplicates` (aviso de Ajustes). 3 tests nuevos de importación repetida (carpeta, cambio de archivo, nota borrada, misma nota en dos rutas, ENEX) y 1 de Ajustes; los tests de versión pasan a 11.
- F17-07: la interfaz de la jerarquía de un nivel ya existía (selector de padre en `TagEditor`, subetiquetas con sangría en `TagSidebar`, D-06). Lo que faltaba: marcar o desmarcar una etiqueta padre hace lo mismo con sus hijas, «Solo» una etiqueta padre muestra también sus hijas (`TagsStore.childrenOf/withChildren`), y el contador pendiente del padre incluye las tareas de sus hijas sin contarlas dos veces. Una subetiqueta sola no toca a su padre. 2 tests nuevos (store y barra lateral).
- F17-08 (D-60): filtros `desde:` / `hasta:` en la búsqueda unificada (`repo/find.rs`, parámetros enlazados, sin SQL con datos del usuario) y alias `is:pending`/`is:done`; vistas guardadas en la paleta (ajuste `saved_searches`, lista JSON validada de ≤ 20 textos de ≤ 200 caracteres; `stores/savedSearches.svelte.ts`). Tests: 1 de búsqueda con rangos, 1 de ajustes, 3 de store y 1 de paleta.
- F17-09 (D-61): `autobackup.rs` (bucle de fondo, `run_if_due`, rotación) y ajustes `auto_backup` / `auto_backup_keep` / `auto_backup_last`; sección «Copia automática» en Ajustes. 5 tests de Rust (apagada, diaria con rotación y reloj manual, semanal con margen, no toca archivos ajenos, valores por defecto), 1 de validación de ajustes, 1 de store y 1 de diálogo.

## Fase 18 — Cimientos de datos (`[A]`; migraciones 015 a 018, D-93 a D-97)
- [x] F18-00 `[A]` Medir la línea base (`npm run build && npm run size`, tamaño del exe, escala) en `docs/metricas.md` y fijar D-98.
- [x] F18-01 `[A]` Migración 015 (`tasks.priority`, `checklist_done`, `checklist_total`) con test de migración desde el esquema 14 con datos; actualizar `MIGRATIONS` y los tests que esperan 14.
- [x] F18-02 `[A]` Cableado de prioridad y checklist: `repo/model.rs`, `task_sql!`/`map_task`, `services/tasks.rs`, `commands/tasks.rs`, `types.ts`, `api.ts`, tests de contrato y paridad, `repo/export.rs`.
- [x] F18-03 `[A]` Migración 016 (`events`, `event_tags`, `event_reminders`), `repo/events.rs`, servicio, comandos, `domain/recurrence::occurrences_in` y recordatorios de evento en el scheduler.
- [x] F18-04 `[A]` Migración 017 (`habits`, `habit_logs`), `repo/habits.rs`, `domain/habits.rs` (racha con `Clock`), comandos y tests.
- [x] F18-05 `[A]` Migración 018 (`time_entries`), comandos de enfoque y recuperación de entradas huérfanas.
- [x] F18-06 `[A]` Búsqueda unificada y papelera incluyen eventos y hábitos.
- [x] F18-07 `[A]` Copia `.agendo` y exportación JSON/Markdown incluyen las tablas nuevas.

**Verificaciones [A] de la Fase 18:**
- F18-00: línea base en `docs/metricas.md` y D-98 (JS 88,5 KB inicial / 270,7 KB total, exe 10,3 MiB).
- F18-01/02 (D-93): migración 015 (`tasks.priority` 0–3 con `CHECK`, `checklist_done`/`checklist_total`); `domain::content::checklist_progress` cuenta las casillas y `repo::tasks` guarda los contadores al crear y al cambiar el contenido. `priority` entra en `TaskCreateInput`/`TaskPatchInput`/`task_create` (validada 0–3), la siguiente ocurrencia de una tarea periódica la hereda, `export.rs` la incluye y `types.ts`, `testing.ts` y el fixture `dto.json` ganan los tres campos. Tests nuevos: 1 de migración con datos desde el esquema 14, 1 de dominio y 3 de servicio. Rust 503, TS 699, JS 270,7 KB. Nota: `scale_tests::ten_thousand_notes_list_and_search_fast` falla si coincide con una compilación en paralelo (CPU compartida); aislado y en reposo pasa.
- F18-03 (D-94): migración 016 (`events`, `event_tags`, `event_reminders`; fechas y horas locales, `reminder_offsets` JSON, `CHECK` de fechas); `domain::recurrence` gana `Rule::occurrences`, `occurrences_in` y `first_on_or_after` (tope de 20 000 pasos); `services::events` valida (todo el día sin horas, fin no anterior al inicio, avisos de días enteros si no hay hora, ≤ 10), expande las repeticiones por consulta (incluye las que empezaron antes de la ventana y duran dentro) y deja un aviso por antelación en la siguiente ocurrencia; el scheduler lo cierra y programa el siguiente sin tocar los avisos que esperan. Comandos `events_list`, `event_get`, `event_create`, `event_update`, `event_delete`, `event_restore`. `Fired` y `Notification` llevan `eventId`. Pulsar la notificación de un evento aún no abre nada (UI de la F21). 12 tests de servicio, 1 de scheduler, 2 de dominio, 1 de esquema.
- F18-04 (D-95): migración 017 (`habits` con máscara de días `days` 1–127 en lugar de BYDAY, equivalente y más simple; `habit_logs` con clave compuesta); `domain::habits::streak` (días no programados ni cuentan ni rompen, hoy sin marcar no rompe, tope de 10 años) con `today` que el servicio saca del `Clock`; comandos `habits_list`, `habit_create/update/delete/restore`, `habit_set_done` (no admite días futuros) y `habit_logs`. 7 tests de dominio, 9 de servicio, 1 de esquema.
- F18-05 (D-97): migración 018 (`time_entries`, un índice único parcial permite una sesión en marcha); `services::focus` (`focus_start/stop/current/list`), fin de sesión y aviso desde el scheduler (se cierra aunque el aviso falle; un tick tardío cierra en la hora planificada) y `recover_orphans` al arrancar (cierra en `min(ahora, inicio + previsto)`). 8 tests de servicio, 1 de esquema.
- F18-06: `find` gana `FoundItem::Event`/`Habit` (texto sin índice FTS, comparado plegando mayúsculas y acentos sobre ≤ 5 000 filas; filtros `is:evento`, `is:habito`, `tag:` y `desde:`/`hasta:` en eventos), `events_trash_list`, `habits_trash_list` y purga a los 30 días en `AppState::purge_trash`. La paleta ignora de momento esos resultados (UI en F20–F21). 4 tests de búsqueda y 2 de papelera.
- F18-07: la copia `.agendo` ya llevaba todo (instantánea de SQLite); test nuevo de ida y vuelta con las tablas nuevas. La exportación JSON añade `events` (con `tagIds` y `reminderOffsets`), `habits` (con `doneDates`), `timeEntries` y los campos de la tarea (`priority`, contadores de checklist). La exportación Markdown exporta solo notas (no incluía tareas) y no cambia.
- Cierre de F18: Rust 552, TS 699, JS 271,1 KB gzip (+0,4 KB: solo tipos y llamadas de `api.ts`), CSS 26,1 KB. Los tests de escala con 10 000 notas son sensibles a la carga de la máquina (fallan si se ejecutan junto a otra compilación y pasan solos). Sin ítems `[H]` en esta fase: no hay interfaz nueva. Ejecutable release: 11 423 744 B (10,9 MiB, +0,6 MiB sobre la línea base). Dependencias: ninguna nueva.

## Fase 19 — Sistema visual y shell (D-92)
- [x] F19-01 `[A]` Tokens `--sidebar-*`, `--priority-*` y paneles tipo tarjeta; test de contraste ampliado (`src/lib/a11y/tokens.test.ts`).
- [ ] F19-02 `[A]` `AppSidebar.svelte` único (navegación, Favoritos = notas fijadas, Recientes, Carpetas, Etiquetas); se retiran `TagSidebar`, `FolderSidebar` y el control de sección.
- [ ] F19-03 `[A]` Barra superior: plegar, nuevo, atrás/adelante y buscador central que abre la paleta.
- [x] F19-04 `[A]` `viewStore.section` ampliado (`today|upcoming|calendar|notes|tasks|events|habits`) y migración del ajuste `app_section`.
- [ ] F19-05 `[A]` Capas de tarjeta en `.page`, ancho mínimo y plegado automático del sidebar.
- [ ] F19-06 `[H]` Revisión visual en Windows 11 (claro/oscuro, Mica, tamaños).

**Verificaciones [A] de la Fase 19:**
- Inicio: el usuario ordenó empezar la Fase 19 el 2026-11-20 con capturas de referencia de la interfaz objetivo; D-92 pasa a `ACCEPTED`.
- F19-01: tokens compartidos `--sidebar-*` (fuera de los bloques de tema: mismos valores en claro y oscuro), `--panel-gap`, `--sidebar-width`; por tema `--bg-panel` (opaco), `--panel-border` y `--priority-0` a `--priority-3`. `tokens.test.ts` comprueba que ningún tema redefine `--sidebar-*`, que el texto, el acento y las prioridades del sidebar pasan AA sobre fondo sólido, Mica claro y oscuro, también con hover y selección, y que todo el texto pasa AA sobre los paneles (120 tests).
- F19-04 (hecha antes que F19-02 porque el sidebar depende de ella): `Section` = `today | tasks | calendar | notes`; **discrepancia con el enunciado:** `upcoming`, `events` y `habits` entran en el tipo cuando existan sus vistas (F20-05, F21, F22), para no dejar secciones vacías (CLAUDE.md §6). El backend ya acepta los siete valores más `agenda`. La lista vive en «Tareas» y los modos de calendario en «Calendario» (`setMode` y `setSection` se mantienen coherentes); `showAgenda()` va a la lista solo desde fuera de la agenda. Migración del ajuste: un `app_section = agenda` guardado por la 2.0.x se lee como `tasks` o `calendar` según `view_mode` y se reescribe. Sin nada guardado, la app abre en «Hoy». Ctrl+Mayús+1 lleva a «Tareas». Tests: 4 nuevos de store y los de paleta, atajos y ajustes de Rust adaptados. Rust 552, TS 772, JS 271,1 KB.

## Fase 20 — Hoy, Próximas, Bandeja y prioridades
- [ ] F20-01 `[A]` Hoy: título grande, tira de 14 días y banner de resumen (todo `$derived`).
- [ ] F20-02 `[A]` Círculo de prioridad en `Checkbox`/`TaskRow`, selector en el diálogo y el menú contextual, orden por prioridad dentro del día.
- [ ] F20-03 `[A]` Progreso de checklist «2/5» en la fila.
- [ ] F20-04 `[A]` Panel derecho «Bandeja de entrada» (carga diferida, plegable, oculto en ventanas estrechas).
- [ ] F20-05 `[A]` Vista Próximas y vista Tareas con filtros.
- [ ] F20-06 `[H]` Validación en Windows 11.

## Fase 21 — Eventos (D-94)
- [ ] F21-01 `[A]` Diálogo y fila de evento; sección Eventos.
- [ ] F21-02 `[A]` Calendario mixto (eventos y tareas con duración), arrastre y redimensionado.
- [ ] F21-03 `[A]` Eventos en Hoy/Próximas y en el banner; recordatorios con notificación nativa.
- [ ] F21-04 `[A]` Convertir tarea ↔ evento desde el menú contextual.
- [ ] F21-05 `[H]` Validación en Windows 11.

## Fase 22 — Hábitos (D-95)
- [ ] F22-01 `[A]` Diálogo de hábito y vista Hábitos con racha y últimos 7 días.
- [ ] F22-02 `[A]` Hábitos del día en Hoy, marcar en un clic con deshacer y contador en el banner.
- [ ] F22-03 `[H]` Validación en Windows 11.

## Fase 23 — Captura en lenguaje natural (D-96)
- [ ] F23-01 `[A]` `src/lib/naturalInput.ts` (fecha, hora, duración, etiqueta `#`, prioridad `!`, repetición) con tests de casos y `Clock` falso.
- [ ] F23-02 `[A]` Integración en borrador en línea, creación rápida del calendario, diálogo y Bandeja, con vista previa y Esc para ignorar.
- [ ] F23-03 `[A]` «Crear cualquier cosa» en la paleta: tarea, evento, hábito y nota.

## Fase 24 — Planificar mi día y bloques de tiempo
- [ ] F24-01 `[A]` Arrastrar una tarea de la Bandeja a una franja de Día/Semana (fija hora y duración).
- [ ] F24-02 `[A]` Asistente «Planificar mi día»: vencidas, elegir las de hoy, suma de tiempo estimado y aviso de sobrecarga.
- [ ] F24-03 `[H]` Validación en Windows 11.

## Fase 25 — Enfoque (D-97)
- [ ] F25-01 `[A]` Temporizador por tarea en la barra superior (`focus.svelte.ts`) y notificación al terminar.
- [ ] F25-02 `[A]` Tiempo dedicado en la fila y el detalle; entradas manuales.
- [ ] F25-03 `[H]` Validación en Windows 11.

## Fase 26 — Resumen semanal
- [ ] F26-01 `[A]` Consulta y dominio: completadas, pendientes y vencidas por etiqueta, eventos, cumplimiento de hábitos y tiempo de enfoque; inicio de semana configurable vía `domain/dates.rs` y `Clock`.
- [ ] F26-02 `[A]` Vista «Resumen» (carga diferida, también desde la paleta) con navegación entre semanas.
- [ ] F26-03 `[A]` Exportar el resumen a Markdown (reutiliza `exchange.rs`, sin dependencias nuevas).
- [ ] F26-04 `[H]` Revisión visual y de textos en Windows 11.

## Fase 27 — Cierre de 3.0.0
- [ ] F27-01 `[A]` Migración real desde una base 2.0.3 con datos (integridad) y pruebas de escala en release con eventos y hábitos.
- [ ] F27-02 `[A]` Versión `3.0.0` en los 5 sitios (CLAUDE.md §10), entrada en este archivo, `docs/manual-pruebas.md` § «Versión 3.0.0», `docs/metricas.md` y README.
- [ ] F27-03 `[A]` Batería completa de CLAUDE.md §4 en verde; `Estado: AWAITING_HUMAN` con los `[H]` de F16 a F26. Etiqueta, push e instaladores **solo** si el usuario dice «subir a GitHub» / «sacar la nueva versión».

**Riesgos:** es la versión más grande (3 migraciones nuevas, 2 entidades nuevas y rediseño del shell). Los eventos repetidos y sus recordatorios tocan el scheduler (`Clock`, `domain/dates.rs`); el sidebar oscuro fijo exige revisar contraste y Mica en claro (D-36); el presupuesto de JS obliga a cargar las vistas nuevas con `import()`. Cada fase termina en `AWAITING_HUMAN` (CLAUDE.md §3); se pueden agrupar fases en una sola validación si el usuario lo pide.
**Orden de recorte si falta tiempo:** Enfoque (F25) → atrás/adelante (F19-03) → expansión de eventos repetidos (F21) → gráfico de 7 días en hábitos.
**Aplazado a una versión posterior (no incluido):** captura rápida con atajo global del sistema, Kanban por etiqueta, grafo de enlaces, importar/exportar ICS, OCR, subpáginas, cifrado de notas.
**Descartado por el usuario (2026-11-18):** sincronización con Google Calendar y Tasks; no se planifica en ninguna versión (D-56).

---

# Anexos transversales

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
- D-56 (`ACCEPTED`) eleva N-03 a 18 MB y N-04 a 300 KB gzip para la 2.1.0; `docs/SPEC.md` §2 conserva los valores originales (no se edita sin ADR) y el script `npm run size` se actualiza en F16-08.
- (resuelta en F0-09) El perfil `release` de la plantilla difería de SPEC §10.1; ahora usa `lto = "thin"`, `strip = "symbols"`, `panic = "unwind"`.


## Notas de sesión
- 2026-11-18: **2.0.3 (sigue sin publicar), segunda corrección del menú de etiquetas**: la captura `menu-tags-cortado.png` mostró que `fixed` no bastaba, porque la barra lateral aún recortaba su borde izquierdo (los ancestros con scroll/animación de la lista lo atrapan). `TagMenu` ahora se mueve a `document.body` al montarse (`place`) y se elimina al destruirse. Lint, check y Vitest en verde; `[H]` repetir el ítem de § «Versión 2.0.3». Sin commit de versión nuevo, etiqueta, push ni instalador.
- 2026-11-17: **Versión 2.0.3 preparada, sin publicar** (a petición del usuario): el menú de etiquetas de la fila de una tarea (`TagMenu`) seguía cortándose contra la barra izquierda porque era `position: absolute` anclado al botón. Ahora es `fixed` y se coloca con `dropdownPosition` (`placement.ts`, con tests): alineado al botón, dentro de la ventana y encima del botón si no cabe debajo; se cierra al redimensionar o desplazar. La 2.0.2 lo había corregido solo en el submenú contextual. Batería `[A]` en verde (Vitest 583, JS 248,2 KB de 250). `[H]` en manual-pruebas § «Versión 2.0.3». Sin etiqueta, push ni instalador.
- 2026-11-17: **Versión 2.0.2 preparada, sin publicar** (a petición del usuario): (1) Ctrl+N en la agenda abre el diálogo completo de nueva tarea (antes un borrador en línea; Ctrl+Mayús+N sigue creando el borrador de hoy y el comando «Nueva tarea» de la paleta abre el mismo diálogo); (2) el subrayado rojo del corrector desaparece al perder el foco (`spellcheck.ts`: `spellcheck="false"` al salir de un `textarea` o región editable y se restablece al entrar); (3) los submenús del menú contextual (p. ej. «Etiquetas») ya no se cortan contra el borde izquierdo: `subSidePlacement` elige lado y desliza si no cabe en ninguno; (4) Ajustes › «Acerca de» › «Información de la aplicación» (comando `app_info`: versión, compilación, sistema, WebView2, Tauri, SQLite, esquema, carpeta de datos, tamaño y recuentos); (5) arreglo de pérdida de datos: el texto escrito en los últimos 400 ms de una nota se guarda al cerrar con el botón de la ventana y al perder el foco o descargar la página (`flushAllSavers`). Batería `[A]` en verde: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (437 de lib + 13), `npm run lint`, `check` (0/0), `test` (579), `build`, `size` (JS 248,0 KB de 250: **quedan 2 KB**). Sin D-xx nueva: no hay dependencias nuevas. `[H]` en manual-pruebas § «Versión 2.0.2». Sin etiqueta, sin push ni instalador.
- 2026-11-16: **Versión 2.0.1 preparada, sin publicar** (a petición del usuario): corrige el menú contextual (D-50), que se cortaba contra el borde de la ventana. **Causa real** (vista en una captura del usuario): el panel principal también pasaba por `fitSub`, que se ejecuta antes que `place` (los hijos se montan primero) y medía una caja aún sin posición, aplicándole un margen negativo que lo sacaba por arriba; ahora `fitSub` solo actúa en submenús. Además: la posición se calculaba con `getBoundingClientRect` durante la animación de apertura (escala 0,96), lo que subestimaba el tamaño, y el submenú se deslizaba sin límite superior. Ahora usa `offsetWidth/offsetHeight`, `placement.ts` (`clampStart`, `slideToFit`, con tests) y los submenús no superan la altura de la ventana. **No se vio en ejecución:** `[H]` abrir el menú con clic derecho pegado a cada borde y esquina, y abrir un submenú («Mover a») cerca del borde inferior. Batería `[A]` en verde (Vitest 570). Etiqueta local `v2.0.1`; no se ha subido ni generado instalador.
- 2026-11-16: **Versión 2.0.0 lanzada** (cierre de la V2, Fase 15): notas con carpetas, etiquetas, adjuntos, historial, plantillas y enlaces; vista «Hoy»; tareas recurrentes; acción «Marcar como hecha»; exportar/importar (Markdown, HTML, JSON, Evernote); copia `.agendo`. Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. Instaladores sin firma: `Agendo_2.0.0_x64-setup.exe` (3,80 MiB) y `Agendo_2.0.0_x64_es-ES.msi` (5,38 MiB). Las migraciones 004 a 010 se aplican solas al abrir; **no hay vuelta atrás a la 1.0.7** con la base ya migrada (haz una copia antes de actualizar).
- 2026-11-16: **repaso de diseño previo a la 2.0 (D-55)**, a petición del usuario; no pertenece a ninguna fase y **no se ha generado versión** (la Fase 15 sigue pendiente y no la hace este agente). Solo CSS y marcado: tokens de resaltado, código y diferencias; menús con resaltado de acento; `Modal` ancho para el historial (antes recortado); «Hoy» en tarjetas agrupadas; historial, plantillas, enlaces entrantes, tareas de la nota, buscador y campo de enlace alineados con D-39; selección de texto en acento. Corregidas variables CSS inexistentes (`--radius-sm`, `--surface-hover`, `--space-5`) y botones sin la tipografía de la app. Batería `[A]` en verde: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (433 de lib + 4 + 1 + 4), `npm run lint`, `check` (0/0), `test` (563 en 51 archivos), `build`, `size` (JS 246,4 KB de 250, **quedan 3,6 KB para F15**; CSS 18,6 KB). **No se ha visto la interfaz en ejecución:** `[H]` en manual-pruebas § «Repaso de diseño previo a la 2.0».
- 2026-11-12: **Versión 1.0.7 lanzada** (D-42): la aplicación pasa a llamarse **Agendo** (producto, crate, binario, paquete e identificador `com.cristian.agendo`), con README y licencia MIT. Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. Validación `[H]` en `docs/manual-pruebas.md` (§ «Cambio de nombre a Agendo»).
- 2026-11-12: **Versión 1.0.6 lanzada** (D-41): etiqueta antes del título en `TaskRow.svelte` e instalador NSIS que reemplaza la versión anterior sin confirmación. Batería `[A]` en verde: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (264 + 9), `npm run lint`, `check` (0/0), `test` (349), `build`, `size`. Instaladores con `npx tauri build`: `MiAgendaApp_1.0.6_x64-setup.exe` (3,20 MiB) y `MiAgendaApp_1.0.6_x64_es-ES.msi` (4,52 MiB). **`[H]`:** aspecto de la etiqueta y que instalar sobre la 1.0.5 no pida confirmación y conserve los datos (manual-pruebas § «Versión 1.0.6»).
- 2026-11-11: **Versión 1.0.5 lanzada** (D-39, D-40): rediseño de la interfaz con estética y movimiento tipo Apple, y notas de las tareas como segunda línea discreta (sin foco al abrir, barra de formato bajo el título, siempre desplegadas al iniciar, sin botón global). Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. Batería `[A]` en verde: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (264 de lib + integración), `npm run lint`, `check` (0/0), `test` (349), `build` y `size` (JS 188,0 KB; CSS 12,5 KB). Instaladores con `npx tauri build`: `MiAgendaApp_1.0.5_x64-setup.exe` (NSIS, 3,20 MiB, el recomendado) y `MiAgendaApp_1.0.5_x64_es-ES.msi` (4,52 MiB) en `src-tauri/target/release/bundle/`; ejecutable `miagendaapp.exe` 12,1 MiB. Sin migraciones: los datos se conservan al instalar sobre la 1.0.4. **No se ha visto la interfaz en ejecución**: aspecto, arrastre de ventana, fluidez, RAM y arranque son `[H]` (manual-pruebas § «Rediseño de interfaz (D-39)» y § «Notas en la lista (D-40)»). Fuera de la versión por decisión del usuario: no hay plan V2 vigente.
- 2026-11-11: ajustes de D-40 a petición del usuario: la barra de formato deja de flotar y se abre justo debajo del título de la tarea, encima del texto de la nota; todas las notas con texto empiezan desplegadas en cada sesión (ocultar una ya no se recuerda al reiniciar y se borra la clave `agenda.notesCollapsed`; **modifica D-38**). **Retirado el «Plan V2»** de este archivo y las entradas B-05 a B-08 de `BLOCKERS.md`, también a petición del usuario: no hay planificación de V2 vigente. Batería `[A]` en verde (Vitest 349; Rust sin cambios).
- 2026-11-11: corrección de D-40: la nota de la lista usaba la clase `inline`, que coincide con la utilidad `inline` de Tailwind (`display: inline`) y rompía su caja (márgenes, fondo y barra flotante). Renombrada a `in-list` / `as-field`. Revisado el CSS generado: ninguna otra clase de componente choca de forma dañina (`block` y `grid` ya fijan su propio `display`). **Regla:** no usar nombres de utilidades de Tailwind como clases propias. Lint, check, Vitest 349, build y size en verde; Rust sin cambios.
- 2026-11-11: **notas en la lista (D-40)**: eliminado el botón «Ocultar/Mostrar todas las notas»; las notas ya no toman el cursor al abrir la app (solo la que el usuario acaba de abrir); se leen como segunda línea gris bajo el título y solo parecen un campo al editarlas; la barra de formato flota bajo la nota sin mover la lista; texto de ayuda «Añadir nota…»; una nota abierta y dejada vacía se recoge sola; casillas redondas. Batería `[A]`: lint, check (0/0), Vitest 349, build, size (JS 188,1 KB; CSS 12,6 KB) y Rust (fmt, clippy, 264 tests) en verde. Sin dependencias nuevas (`Placeholder` viene en `@tiptap/extensions`, ya instalado). Apariencia `[H]`: manual-pruebas § «Notas en la lista (D-40)».
- 2026-11-11: **rediseño de la interfaz (D-39)**: barra unificada con zona de arrastre en la barra de herramientas y la cabecera lateral, tokens de superficie/sombra/movimiento nuevos, controles compartidos en `base.css` y todos los componentes restilizados; solo CSS y marcado (ni lógica, ni dependencias, ni Rust). Batería `[A]`: `npm run lint`, `check` (0 errores, 0 avisos), `test` (347), `build` y `size` (JS 187,9 KB; CSS 12,3 KB) en verde; `cargo fmt --check`, `clippy -D warnings` y `cargo test` en verde sin cambios en Rust. `NoteEditor.test.ts` «loads TipTap» volvió a agotar el tiempo una vez en la primera pasada (ya anotado en la 1.0.3) y pasó en dos reejecuciones. Discrepancias con SPEC §8 registradas en D-39. **No se ha visto la interfaz en ejecución**: apariencia, arrastre de ventana y fluidez son `[H]`. La versión no se ha subido.
- 2026-10-31 (F1-03, para F4): al mover una tarea a la papelera sus `reminders` no se borran (se restauran con ella). El scheduler (F4-02/F4-03) debe ignorar los recordatorios cuyas tareas tengan `deleted_at` y las consultas de recuperación (F4-04) también.
- 2026-10-31 (F1-03): `TagWithCount` se serializa como `Tag` más `pendingCount` (SPEC §7.3 no define el campo); F1-09 debe reflejarlo en `api.ts`.
- 2026-10-31 (F1-01): SPEC §5.3 menciona la "ruta del backup" cuando falla una migración; la copia previa a migrar depende de `db/backup.rs` (F6-04) y aún no existe. Hasta entonces el fallo deja la base intacta (rollback) y el error indica la versión que falló.
- 2026-10-31: el entorno real es Windows 11, no Linux (CLAUDE.md §8). El terminal con sandbox (WSL) no arranca por falta de `libasound.so.2`; los comandos se ejecutan fuera del sandbox con aprobación del usuario. Por tanto `cargo check --target x86_64-pc-windows-msvc` es posible en este equipo cuando toque (F0-07).
- 2026-10-31: SPEC §8.3 no define los tokens claros cuando `data-theme="light"` y el SO está en oscuro (el bloque claro solo vive dentro de `prefers-color-scheme: light`). `tokens.css` añade el bloque `:root[data-theme="light"]` para que el selector manual de tema (F6-05) funcione. También añade `--bg-window-native`, `--bg-window-solid` (valores de SPEC §8.2, ahora por tema), `--scrollbar-thumb` y `--tag-1`…`--tag-8` (colores propuestos por el agente; su contraste se revisa en F6-02).
- 2026-10-31: hasta F0-06 la ventana no tiene barra de título ni botones (`decorations: false`); para cerrarla en `tauri dev` usar Alt+F4 o detener el proceso.
- 2026-11-09: mejoras posteriores a F6 (D-33): vista de detalle desde el calendario, edición con el diálogo de nueva tarea, borrado desde el detalle, duración de tareas (migración 003), tareas hechas sin recordatorios y lista con las pendientes más destacadas. Verificado `[A]`: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (260), `npm run lint`, `npm run check`, `npm run test` (306). **No** se ejecutaron `npm run build` ni `npm run size` (el usuario genera los instaladores con `build-installers.bat`). Pendientes `[H]` en `docs/manual-pruebas.md` § «Detalle de tarea y duración».
- 2026-11-09: etiquetas de tareas (D-34): barra lateral con filtro por casillas, selector de etiqueta en el diálogo, chips en la lista y eventos de color en el calendario. Sin migración. Batería `[A]` en verde (Vitest 332+, `cargo test` 260 de lib); la apariencia real es `[H]` (manual-pruebas § Etiquetas).
- 2026-11-09: **Versión 1.0.0 lanzada.** Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. Instaladores generados con `npx tauri build`: `MiAgendaApp_1.0.0_x64-setup.exe` (NSIS, 3,2 MiB, instalación por usuario; el recomendado) y `MiAgendaApp_1.0.0_x64_es-ES.msi` (4,5 MiB). Para actualizar basta ejecutar el instalador de una versión mayor sobre la instalada; los datos (`%APPDATA%\com.cristian.miagendaap`) se conservan y las migraciones se aplican al abrir. No mezclar NSIS y MSI en el mismo equipo. Base de datos de desarrollo borrada para una instalación limpia. La validación de los ítems `[H]` sigue pendiente.
- 2026-10-31: el identifier heredado de F0-01 es `com.cristian.miagendaap` (una `p` menos que el nombre del proyecto). Se mantiene y el test lo fija; cambiarlo después mueve el directorio de datos de la app y la identidad de notificaciones. Decidir antes de distribuir (pendiente para el humano). **Sustituido por `com.cristian.agendo` en D-42.**
- 2026-10-31: `PROGRESS.md` tenía F0-01 sin marcar aunque estaba commiteada (`c62a00e`); marcada en la sesión de F0-02.
- 2026-11-09: cambios 1.0.1 (D-35): hora por defecto de avisos en Ajustes, aviso 1 semana antes, avisos por días/semanas para tareas sin hora (sin colocarlas en el calendario), botón guardar de la edición de etiquetas contenido en su cuadro y alineación de columnas de la semana con la cabecera fija (compensa el ancho del scrollbar). Las tareas sin hora ya aparecían en «Todo el día». Batería `[A]` en verde (cargo fmt/clippy/test 263; npm lint/check/test 340/build/size 187 KB). Pendiente `[H]`: manual-pruebas § «Versión 1.0.1».
- 2026-11-09: tema claro opaco (D-36): sin Mica/Acrílico en claro, fondos opacos y borde más marcado; se reaplica al cambiar el tema del sistema. Batería `[A]`: lint, check, Vitest 341, build y size en verde; Rust sin cambios. Apariencia real `[H]`.
- 2026-11-09: **Versión 1.0.1 lanzada** (D-35, D-36). Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y los dos lockfiles. El usuario revisó la interfaz con `tauri dev` (tema claro opaco, hora de aviso por defecto, semana alineada). Instaladores con `npx tauri build`. Los ítems `[H]` de la 1.0.1 siguen sin marcar en `docs/manual-pruebas.md` hasta su validación formal.
- 2026-11-09: **Versión 1.0.2 lanzada** (D-37): casillas (subtareas) en las notas de las tareas, marcables con un clic sin editar el texto. Batería `[A]` en verde (`cargo fmt/clippy/test` 264 de lib; `npm lint/check/test` 342/build/size 187,3 KB). Pendiente `[H]`: manual-pruebas § «Versión 1.0.2». Versión fijada en `package.json`, `Cargo.toml`, `tauri.conf.json` y lockfiles; instaladores con `npx tauri build`.
- 2026-11-09: **Versión 1.0.3 lanzada.** Corrección: al teclear la fecha, el navegador notifica años parciales (`0002`, `0020`, `0202`) y la app los confirmaba, dejando la tarea en el año 2 y rompiendo la escritura. `isSettledDate` (`dates.ts`) ignora esos valores en el diálogo de tarea y en la fila. Batería `[A]` en verde (Vitest 346, build, size, Rust 264). Nota: `NoteEditor.test.ts` «loads TipTap lazily» falló una vez por tiempo de carga con la máquina ocupada y pasó en las dos ejecuciones siguientes. Pendiente `[H]`: manual-pruebas § «Versión 1.0.3».

## Auditoría de calidad (docs/AUDITORIA.md)
- 2026-11-19 · A0 (línea base en `docs/metricas.md`) y A1 `[A]` hechas. CSP real y `devCsp` nulo en `tauri.conf.json` (test `csp_is_restrictive`), test hostil de anidamiento inline en `markdown.rs`. Descartados: quitar `allow-set-effects` (D-02/SPEC §8.1) y proteger `export_notes` (ya seguro). Batería completa en verde (Rust 457+9, TS 604, JS 268,3 KB gzip). Pendiente `[H]`: A1-01. Estado: AWAITING_HUMAN antes de A2.
- 2026-11-19 · Carril frontend integrado (merge 4f4d275): A6-02/03/05, A10-03/04, A12-01..04. Descartados por falsos positivos: A6-04 y gran parte de A6-03. Bug real corregido: `TodoView` ocultaba el error de carga. D-70 `PROPOSED` (Tailwind). `CalendarChip` e `InlineEdit` aplazados (presupuesto JS 248,4/250 KB en la base antigua; sin Windows no se verifica el aspecto).
- 2026-11-19 · A9 integrada (merge 111f4a1): vectores de paridad compartidos Rust/TS; divergencia real en `parseRule` corregida; D-80. `.claude/` excluido de lint.
- 2026-11-19 · A8 parcial integrada: stores de notas, `+page.svelte`, `editor.ts` y `context/entries.ts` modularizados sin cambio de API; JS 269,5 KB gzip (+1 KB). Pendientes: `tasks.svelte.ts` (con A6-01), `CalendarWeek`, `TaskDialog`; `[H]` en «Auditoría A8».
- 2026-11-19 · Carril de datos integrado (A2–A4; migraciones 012–014, esquema 14; D-62 a D-66). Mejoras medidas a 50 000 filas: FTS 21,9 ms→44 µs, agenda con recordatorios 13,4 s→0,1 s, ventana de notas 75→0,3 ms. Pool de 3 lectores, ENEX y `.agendo` en flujo, apagado ordenado. Rust 491 tests, TS 695.
- 2026-11-19 · Integrados el carril frontend (A6-01, A8 tareas/calendario, A11 TS) y el de Rust (A5 registro propio D-90, A7-01..04: módulos divididos y `services/`). Tests de tiempos: causa real = sobresuscripción de hilos; `RUST_TEST_THREADS=8` (suite Rust 10,5 s → 4,7 s, estable). Rust 498 tests, TS 699.
