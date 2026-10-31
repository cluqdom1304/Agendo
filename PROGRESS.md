# PROGRESS.md

**Fase activa:** 1
**Estado:** IN_PROGRESS   <!-- IN_PROGRESS | AWAITING_HUMAN | BLOCKED | DONE -->
**Siguiente tarea:** F1-06
**Última actualización:** 2026-10-31 (F1-05)

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
- [ ] F1-06 `domain/validation.rs`: longitudes, profundidad de etiquetas (D-06)
- [ ] F1-07 Comandos `tasks_*`, `tag_*`, `task_tags_set`, `trash_list`, `task_restore`
- [ ] F1-08 `error.rs` con prefijos de código (D-08)
- [ ] F1-09 `api.ts` y tipos DTO sincronizados con Rust (test de serialización)

**Verificaciones [A]:**
- F1-01: `src-tauri/src/db/` (`mod.rs`: `open`, `open_in_memory`, `DbError`; `pragmas.rs`: SPEC §5.4; `migrate.rs`: `Migration`, `migrate` por `user_version`). Cada migración corre en su propia transacción `IMMEDIATE` junto con el `user_version`; si falla se revierte, y una base más nueva que la app se rechaza (`SchemaTooNew`). 16 tests: migración desde vacío, idempotencia, solo pendientes, rollback de una migración fallida (memoria y disco), lista no consecutiva, pragmas reales en fichero (WAL, `foreign_keys`, `synchronous=NORMAL`, `busy_timeout=5000`, `temp_store=MEMORY`), reapertura sin reaplicar. Los tests usan migraciones de prueba; la lista real (`001_init.sql`) llega en F1-02. Dependencia `rusqlite` registrada como D-17. `cargo fmt --check`, `clippy --all-targets -D warnings`, `cargo test` (30 de lib + 4 + 3), `npm run lint`, `check`, `build` y `size` (32,8 KB) en verde; `npm run test` aún no existe.
- F1-02: `src-tauri/migrations/001_init.sql` (copia literal de SPEC §5.2) incrustado con `include_str!` en `db::MIGRATIONS`. `db/schema_tests.rs` (12 tests) comprueba migración desde vacío a `user_version = 1`, tablas e índices, valores por defecto, `CHECK` de título/fechas/hora/color/nombre (incluidos límites 500 y 60), unicidad de etiquetas sin distinguir mayúsculas, claves foráneas, cascadas (`ON DELETE CASCADE` y `SET NULL`) y FTS5 con y sin acentos, mayúsculas y prefijo. Batería completa en verde (42 tests de lib + 4 + 3; bundle JS 32,9 KB).
- F1-03: `repo/{model,tasks,tags,fts}.rs`. Todas las escrituras reciben `&Transaction` (el FTS se actualiza en la misma transacción por construcción) y `repo::with_tx` ejecuta una operación lógica en una transacción `IMMEDIATE`. El repositorio no lee el reloj ni genera ids: el llamador pasa `id` y `now` (UTC). Devuelve `rusqlite::Result`; "no encontrado" es `None`/`false` y los errores de restricción los traducirá `error.rs` (F1-08). `Task` y `Tag` ya serializan en camelCase según SPEC §7.2 (tags y recordatorios se leen con subconsultas JSON, sin N+1). 40 tests nuevos: CRUD, posición al final de la sección, parches (incluido borrar fecha/hora), completar/reabrir, soft delete, restaurar, papelera, purga a 30 días (límite exacto, cascada), listados todo/agenda/rango con filtro por etiqueta, etiquetas (upsert, contadores de pendientes, borrado, `set_task_tags`), FTS (insert/sync/rebuild/rollback) y **10 000 operaciones aleatorias** tras las que `integrity_check = ok`, `foreign_key_check` vacío y `tasks_fts` idéntica a las tareas vivas. Batería completa en verde (82 tests de lib + 4 + 3).
- F1-04: `domain/dates.rs` (dependencia `chrono`, D-18). `parse_date`/`parse_time`/`parse_instant` son estrictos (`YYYY-MM-DD` con fecha de calendario real, `HH:MM` 00:00–23:59, `YYYY-MM-DDTHH:MM:SSZ`; rechazan espacios, signos, dígitos no ASCII y formatos laxos) y sus errores `DateError` tienen mensaje en español. `local_to_utc` / `utc_to_local` reciben la zona como parámetro (`chrono::Local` en producción); no se lee el reloj. Reglas de DST fijadas y probadas con una zona simulada tipo Madrid: hora inexistente de primavera pasa a la hora siguiente (02:30 → 03:30), hora repetida de otoño = primera aparición; fechas extremas devuelven `OutOfRange` en lugar de provocar el pánico de `chrono`. 17 tests (99 de lib en total + 4 + 3). La zona horaria real del equipo solo se ejercita con un test de ida y vuelta fuera de transiciones.
- F1-05: `domain/ordering.rs` con el trait `Sortable` (implementado para `repo::model::Task`, así `domain` no depende de `repo`), `sort_todo`, `sort_day` y `group_by_day` (`DayGroup<T>`, serializa `{ date, tasks }`; deja fuera las tareas sin fecha). Implementa D-05: día más reciente primero; dentro del día, con hora (ascendente) → sin hora (`position`) → completadas por `completed_at`; `id` desempata, de modo que el resultado no depende del orden de entrada (comprobado con todas las permutaciones). Tests de tabla (11 casos de día, 5 de POR HACER), agrupación, serialización y uno con datos reales de SQLite: 7 tests nuevos (106 de lib + 4 + 3). D-05 no dice qué pasa con las completadas sin fecha: se aplicó la misma regla de subsección y se registró la decisión pendiente como B-03.
**Ítems [H]:** ninguno en esta fase

---

## Fase 2 — Vista principal tipo documento
- [ ] F2-01 Componentes `SectionHeader`, `DayGroup`, `TaskRow`, `Checkbox`, `InlineTitle`
- [ ] F2-02 Stores con runes (`tasks.svelte.ts`) y agrupación con `$derived`
- [ ] F2-03 Secciones "POR HACER" y "AGENDA" con fechas `DD/MM/YYYY` descendentes
- [ ] F2-04 Creación rápida (Ctrl+N, Enter en título)
- [ ] F2-05 Completar con animación y reubicación en "Completadas"
- [ ] F2-06 Edición inline de título (guardado en Enter o pérdida de foco)
- [ ] F2-07 Eliminación con deshacer de 5 s (F-02)
- [ ] F2-08 Atajos de SPEC §9 para estas acciones (`lib/keymap.ts`)
- [ ] F2-09 Estados vacíos (`EmptyState`)

**Verificaciones [A]:** (pendiente)
**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 2

---

## Fase 3 — Editor de notas y búsqueda
- [ ] F3-01 `NoteEditor` con TipTap (carga diferida, D-10) y `NoteToolbar`
- [ ] F3-02 Extensiones: negrita, cursiva, código, enlace, listas, historial
- [ ] F3-03 Guardado con debounce de 400 ms y `content_text` derivado
- [ ] F3-04 Sub-puntos indentados bajo la tarea (24 px)
- [ ] F3-05 Spotlight (`Ctrl+K`) con búsqueda FTS y navegación por teclado
- [ ] F3-06 Saneado de enlaces (solo `http`, `https`, `mailto`)

**Verificaciones [A]:** (pendiente)
**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 3

---

## Fase 4 — Recordatorios y notificaciones
- [ ] F4-01 `scheduler/clock.rs`: trait `Clock` y dos implementaciones
- [ ] F4-02 `scheduler/mod.rs`: bucle `select!` con tope de 60 s y reprogramación por `Notify`
- [ ] F4-03 Disparo transaccional con reintento y backoff (SPEC §6.3)
- [ ] F4-04 Recuperación al arrancar (ventana 24 h, SPEC §6.4)
- [ ] F4-05 `notify/mod.rs` con `tauri-plugin-notification`
- [ ] F4-06 `ReminderPopover` con opciones de F-12
- [ ] F4-07 Recalcular recordatorios al cambiar fecha/hora de la tarea
- [ ] F4-08 Evento `reminder:fired` y toast in-app

**Verificaciones [A]:** (pendiente)
**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 4

---

## Fase 5 — Vistas de calendario
- [ ] F5-01 `SegmentedControl` (Lista / Día / Semana / Mes), persistido en `settings`
- [ ] F5-02 `CalendarMonth`, `CalendarWeek`, `CalendarDay`
- [ ] F5-03 Creación rápida arrastrando un tramo horario (F-11)
- [ ] F5-04 Carga diferida del módulo de calendario (`import()`)
- [ ] F5-05 Primer día de semana según sistema (`Intl.Locale`)

**Verificaciones [A]:** (pendiente)
**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 5

---

## Fase 6 — Pulido, accesibilidad, copias y empaquetado
- [ ] F6-01 Modo claro/oscuro completo y verificado
- [ ] F6-02 Revisión automatizada de contraste sobre tokens (N-08)
- [ ] F6-03 Navegación por teclado completa y ARIA en controles
- [ ] F6-04 `backup_create` / `backup_restore` con validación (F-16, D-09)
- [ ] F6-05 Ajustes: tema, inicio con Windows (desactivado por defecto), efecto de ventana
- [ ] F6-06 Tests E2E con WebdriverIO + tauri-driver
- [ ] F6-07 Instalador NSIS/MSI, icono y metadatos (firma pendiente)
- [ ] F6-08 Tabla de métricas N-01 a N-08 en `docs/`

**Verificaciones [A]:** (pendiente)
**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 6

---

## Métricas

| Métrica | Objetivo | Valor | Fecha | Notas |
|---------|----------|-------|-------|-------|
| Tamaño ejecutable (N-03) | < 15 MB | 8 496 640 B (≈ 8,1 MiB) | 2026-10-31 | `tauri build --no-bundle`, commit F0-09, sin instalador; plantilla + TitleBar + window-vibrancy |
| Bundle JS gzip (N-04) | < 250 KB | 32,8 KB (CSS 3,7 KB) | 2026-10-31 | `npm run size` tras F0-09 |
| Arranque (N-01) [H] | < 200 ms | | | |
| RAM en reposo (N-02) [H] | < 40 MB (proceso principal) | | | |
| Consulta de un mes, 10 000 tareas (N-05) | < 30 ms | 3,4 ms (434 filas, mejor de 5) | 2026-10-31 | Test `month_query_with_ten_thousand_tasks_is_fast`, build de depuración; el test falla si supera 30 ms |
| Búsqueda, 10 000 tareas (F-09) | < 100 ms | | | |
| `fts::sync` con 10 000 filas (D-11) | < 100 ms (umbral del test) | 2,3 ms (mejor de 5) | 2026-10-31 | Borrado por `task_id` sobre FTS independiente; no hace falta cambiar el diseño de D-11 |

## Discrepancias detectadas
- D-01 pide `set_effects` de Tauri, pero este no informa de errores y no permite el fallback real; se usa `window-vibrancy` directo. Registrado como D-16 (`PROPOSED`, pendiente de confirmación humana).
- (resuelta en F0-09) El perfil `release` de la plantilla difería de SPEC §10.1; ahora usa `lto = "thin"`, `strip = "symbols"`, `panic = "unwind"`.

## Notas de sesión
- 2026-10-31 (F1-03, para F4): al mover una tarea a la papelera sus `reminders` no se borran (se restauran con ella). El scheduler (F4-02/F4-03) debe ignorar los recordatorios cuyas tareas tengan `deleted_at` y las consultas de recuperación (F4-04) también.
- 2026-10-31 (F1-03): `TagWithCount` se serializa como `Tag` más `pendingCount` (SPEC §7.3 no define el campo); F1-09 debe reflejarlo en `api.ts`.
- 2026-10-31 (F1-01): SPEC §5.3 menciona la "ruta del backup" cuando falla una migración; la copia previa a migrar depende de `db/backup.rs` (F6-04) y aún no existe. Hasta entonces el fallo deja la base intacta (rollback) y el error indica la versión que falló.
- 2026-10-31: el entorno real es Windows 11, no Linux (CLAUDE.md §8). El terminal con sandbox (WSL) no arranca por falta de `libasound.so.2`; los comandos se ejecutan fuera del sandbox con aprobación del usuario. Por tanto `cargo check --target x86_64-pc-windows-msvc` es posible en este equipo cuando toque (F0-07).
- 2026-10-31: SPEC §8.3 no define los tokens claros cuando `data-theme="light"` y el SO está en oscuro (el bloque claro solo vive dentro de `prefers-color-scheme: light`). `tokens.css` añade el bloque `:root[data-theme="light"]` para que el selector manual de tema (F6-05) funcione. También añade `--bg-window-native`, `--bg-window-solid` (valores de SPEC §8.2, ahora por tema), `--scrollbar-thumb` y `--tag-1`…`--tag-8` (colores propuestos por el agente; su contraste se revisa en F6-02).
- 2026-10-31: hasta F0-06 la ventana no tiene barra de título ni botones (`decorations: false`); para cerrarla en `tauri dev` usar Alt+F4 o detener el proceso.
- 2026-10-31: el identifier heredado de F0-01 es `com.cristian.miagendaap` (una `p` menos que el nombre del proyecto). Se mantiene y el test lo fija; cambiarlo después mueve el directorio de datos de la app y la identidad de notificaciones. Decidir antes de distribuir (pendiente para el humano).
- 2026-10-31: `PROGRESS.md` tenía F0-01 sin marcar aunque estaba commiteada (`c62a00e`); marcada en la sesión de F0-02.
