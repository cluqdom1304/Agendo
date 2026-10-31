# PROGRESS.md

**Fase activa:** 0
**Estado:** IN_PROGRESS   <!-- IN_PROGRESS | AWAITING_HUMAN | BLOCKED | DONE -->
**Siguiente tarea:** F0-06
**Última actualización:** 2026-10-31 (F0-05)

> Reglas: marcar `[x]` solo al pasar la batería de verificación (CLAUDE.md §4). Los ítems `[H]` nunca se marcan como verificados por el agente.

---

## Fase 0 — Esqueleto y medición base
- [x] F0-01 Crear proyecto con `npm create tauri-app@latest` (Svelte + TypeScript); registrar versiones en DECISIONS.md
- [x] F0-02 Tailwind CSS v4 mediante plugin de Vite
- [x] F0-03 ESLint, Prettier, svelte-check, rustfmt, clippy configurados
- [x] F0-04 `tauri.conf.json`: `decorations: false`, `transparent: true`, identifier estable, tamaños 1100×720 / mínimo 720×480
- [x] F0-05 `capabilities/default.json` con permisos mínimos
- [ ] F0-06 `TitleBar` con `data-tauri-drag-region` y `WindowControls`
- [ ] F0-07 `window/effects.rs`: Mica → Acrylic → none, comando `window_set_effect`
- [ ] F0-08 Tokens CSS y tema oscuro/claro (SPEC §8.3)
- [ ] F0-09 `scripts/size-check.mjs` con presupuesto N-04
- [ ] F0-10 Log de arranque con marcas de tiempo (solo debug)

**Verificaciones [A]:**
- F0-02: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (0 tests), `npm run check` (0 errores, 0 avisos) y `npm run build` en verde. El CSS generado incluye el banner `tailwindcss v4.3.3`.
- F0-03: `npm run lint` (Prettier + ESLint) en verde; comprobado que ESLint falla ante `TODO` y `any`. `cargo clippy` con `unwrap_used`/`expect_used`/`todo`/`unimplemented` en `deny`.
- F0-04: `src-tauri/tests/tauri_conf.rs` (3 tests) comprueba ventana `main` sin decoraciones, transparente, con sombra, 1100×720 / mínimo 720×480 e identifier. `tauri-build` acepta la configuración (clippy/test compilan). La apariencia real es [H] (manual-pruebas § Fase 0).
- F0-05: `src-tauri/tests/capabilities.rs` (4 tests) fija el conjunto exacto de permisos, sin `*:default`, solo ventana `main` y `opener` limitado a `http`/`https`/`mailto`. Los permisos de notificaciones (F4-05) y diálogo de archivos (F6-04) se añadirán con sus plugins, porque un permiso de un plugin no instalado rompe el build. `core:event` se añadirá en F4-08 y cualquier permiso extra de ventana (p. ej. `allow-is-maximized`) se añadirá en la tarea que lo use, actualizando el test.
- No existen aún (no cuentan como fallo): `npm run test` (Vitest, F1/F2), `npm run size` (F0-09).

**Ítems [H]:** ver `docs/manual-pruebas.md` § Fase 0

---

## Fase 1 — Persistencia y dominio
- [ ] F1-01 `db/`: conexión con pragmas (SPEC §5.4), migraciones por `user_version`, test de migración desde vacío
- [ ] F1-02 Migración `001_init.sql` (SPEC §5.2)
- [ ] F1-03 `repo/tasks.rs`, `repo/tags.rs`, `repo/fts.rs` con sincronización FTS en transacción
- [ ] F1-04 `domain/dates.rs`: validación `YYYY-MM-DD` / `HH:MM`, conversión local/UTC
- [ ] F1-05 `domain/ordering.rs`: funciones puras de orden (D-05) con tests de tabla
- [ ] F1-06 `domain/validation.rs`: longitudes, profundidad de etiquetas (D-06)
- [ ] F1-07 Comandos `tasks_*`, `tag_*`, `task_tags_set`, `trash_list`, `task_restore`
- [ ] F1-08 `error.rs` con prefijos de código (D-08)
- [ ] F1-09 `api.ts` y tipos DTO sincronizados con Rust (test de serialización)

**Verificaciones [A]:** (pendiente)
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
| Tamaño ejecutable (N-03) | < 15 MB | | | |
| Bundle JS gzip (N-04) | < 250 KB | | | |
| Arranque (N-01) [H] | < 200 ms | | | |
| RAM en reposo (N-02) [H] | < 40 MB (proceso principal) | | | |
| Consulta de un mes, 10 000 tareas (N-05) | < 30 ms | | | |
| Búsqueda, 10 000 tareas (F-09) | < 100 ms | | | |

## Discrepancias detectadas
- `src-tauri/Cargo.toml` (plantilla) usa `lto = true`, `panic = "abort"`, `strip = true`; SPEC §10.1 pide `lto = "thin"`, `strip = "symbols"`, `panic = "unwind"` (el scheduler necesita capturar pánicos). Se corregirá en F0-09, antes de medir el tamaño del ejecutable.

## Notas de sesión
- 2026-10-31: el entorno real es Windows 11, no Linux (CLAUDE.md §8). El terminal con sandbox (WSL) no arranca por falta de `libasound.so.2`; los comandos se ejecutan fuera del sandbox con aprobación del usuario. Por tanto `cargo check --target x86_64-pc-windows-msvc` es posible en este equipo cuando toque (F0-07).
- 2026-10-31: hasta F0-06 la ventana no tiene barra de título ni botones (`decorations: false`); para cerrarla en `tauri dev` usar Alt+F4 o detener el proceso.
- 2026-10-31: el identifier heredado de F0-01 es `com.cristian.miagendaap` (una `p` menos que el nombre del proyecto). Se mantiene y el test lo fija; cambiarlo después mueve el directorio de datos de la app y la identidad de notificaciones. Decidir antes de distribuir (pendiente para el humano).
- 2026-10-31: `PROGRESS.md` tenía F0-01 sin marcar aunque estaba commiteada (`c62a00e`); marcada en la sesión de F0-02.
