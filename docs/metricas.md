# Métricas de cumplimiento N-01 a N-08

Resultado de la revisión de F6-08 sobre el commit que la contiene. Fuente de los requisitos:
`docs/SPEC.md` §2. Las filas `[H]` necesitan un equipo Windows 11 real y una persona que
mida: el agente no las rellena (CLAUDE.md §5).

| ID | Requisito | Objetivo | Resultado | Estado | Cómo se midió |
|----|-----------|----------|-----------|--------|---------------|
| N-01 | Arranque (proceso → primera pantalla) | < 200 ms | **103 ms** | cumple `[H]` (2026-11-18, medido por el usuario) | Release, `npm run measure` (ventana visible); ver `docs/manual-pruebas.md` |
| N-02 | Memoria en reposo | < 40 MB (proceso principal) | **32,9 MB** (total con WebView2 369,1 MB, informativo) | cumple `[H]` (2026-11-18, medido por el usuario) | Administrador de tareas, proceso principal y suma con WebView2 |
| N-03 | Tamaño del ejecutable (release, sin instalador) | < 15 MB | **12 641 792 B (12,06 MiB)** | cumple `[A]` | `npm run tauri -- build`, `src-tauri/target/release/miagendaapp.exe` |
| N-04 | JS de la aplicación (gzip) | < 250 KB | **173,1 KB** (CSS 8,5 KB) | cumple `[A]` | `npm run build && npm run size` |
| N-05 | Consulta de agenda de un mes | < 30 ms con 10 000 tareas | **2,98 ms** (434 filas, mejor de 5, build de depuración) | cumple `[A]` | `cargo test month_query_with_ten_thousand -- --nocapture` (el test falla por encima de 30 ms) |
| N-06 | Integridad ante cierre forzado | Sin pérdida de datos | **Sin pérdida**: el proceso muerto en mitad de una escritura deja la base íntegra y sin la escritura incompleta | cumple `[A]` | `cargo test --test crash_recovery` (`PRAGMA integrity_check`) |
| N-07 | Precisión de recordatorios | ≤ 5 s tras la hora | Tests del scheduler con reloj inyectado (Fase 4). Prueba real pendiente | `[A]` cumple · `[H]` pendiente | `cargo test scheduler` · prueba manual de Fase 4 |
| N-08 | Accesibilidad | Contraste AA, teclado completo, `prefers-reduced-motion` | Contraste: 6 tokens de texto + etiquetas sobre 4 superficies en 2 temas, todos ≥ 4,5:1 (D-27). Nombres accesibles verificados en lista, Día, Semana, Mes, buscador, ajustes y controles de ventana. Calendario operable con teclado. `prefers-reduced-motion` desactiva animaciones (`base.css`) | `[A]` cumple · `[H]` pendiente | `npm run test` (`src/lib/a11y/`); revisión con Narrador y contraste real sobre Mica pendientes |

## Instaladores (F6-07)

| Artefacto | Tamaño | Notas |
|-----------|--------|-------|
| `MiAgendaApp_0.1.0_x64-setup.exe` (NSIS, instalación por usuario, español) | 3,17 MiB | Sin firma |
| `MiAgendaApp_0.1.0_x64_es-ES.msi` (WiX) | 4,48 MiB | Sin firma |

Sin firma digital, Windows SmartScreen mostrará «Windows protegió su PC» al ejecutar el
instalador (comportamiento esperado, ítem `[H]` en `docs/manual-pruebas.md`). La firma está
pendiente de un certificado de firma de código.

## Cobertura de dominio

`cargo llvm-cov --lib` (instalado en F6-08): `domain/` cubre el **98,8 % de las líneas**
(`content` 98,9 %, `dates` 96,4 %, `ordering` 100 %, `reminders` 100 %, `validation` 100 %);
el total del crate, 92,7 %. Objetivo del DoD de Fase 6: ≥ 90 % en dominio.

## Qué falta para cerrar la tabla

1. N-01, N-02: medir en el equipo de referencia (`[H]`).
2. N-07: prueba real con el equipo despierto (`[H]`).
3. N-08: Narrador de Windows y contraste sobre Mica real (`[H]`).

## Versión 2.0.0 (F15)

| Medida | Objetivo | Resultado | Cómo se midió |
|--------|----------|-----------|---------------|
| Lista de 10 000 notas (todas) | < 250 ms (la interfaz la virtualiza) | 31,3 ms | `cargo test scale_tests -- --nocapture`, build de depuración, mejor de 5 |
| Lista de una carpeta (200 notas) | < 50 ms | 1,7 ms | ídem |
| Búsqueda con 10 000 notas (peor de 4 consultas) | < 50 ms | 19,9 ms | ídem, `repo::find::find` |
| Abrir una base con 10 000 notas (arranque del almacén) | < 200 ms | 1,8 ms | ídem |
| Migración desde una base de la 1.0.7 (esquema 3) | datos intactos, `integrity_check` y claves foráneas limpios | cumple | `a_1_0_7_database_migrates_to_the_current_schema_keeping_its_data` (base construida con las migraciones 1 a 3 y rellenada como un usuario) |
| N-03 ejecutable release | < 15 MB | 15 363 072 B (**14,65 MiB**; con la medida en MiB de la tabla anterior cumple, en MB decimales 15,36 queda 0,36 por encima) | `npm run tauri -- build` |
| N-04 JS gzip | < 250 KB | 246,4 KB | `npm run size` |
| Instaladores | — | `Agendo_2.0.0_x64-setup.exe` 3,80 MiB · `Agendo_2.0.0_x64_es-ES.msi` 5,38 MiB (sin firma) | `npm run tauri -- build` |

Las mediciones de arranque y memoria (N-01, N-02) siguen siendo `[H]`. El rendimiento medido es el
de la capa de datos con una base **sintética**; el de la interfaz con 10 000 notas reales es `[H]`.

## Versión 2.1.0 — límites vigentes y seguimiento (D-56)

Los límites de N-03 y N-04 cambian por D-56 (`ACCEPTED`, 2026-11-18); `docs/SPEC.md` §2 conserva los
valores originales. Las filas de resultado se rellenan al cerrar cada fase; `npm run size` comprueba ya
300 KB (F16-08).

**F16-01/02 (2026-11-18):** `npm run audit` (`scripts/bundle-audit.mjs`) separa carga inicial y chunks
diferidos. Carga inicial gzip: 106,8 KB → **83,2 KB** (−23,6 KB) al cargar bajo demanda notas, carpetas, «Hoy»,
menú contextual, paleta y diálogos de tarea/ajustes. El JS total sube a 263,4 KB por la división en chunks
(el editor, 136 KB, ya era diferido).

| ID | Requisito | Límite SPEC | Límite vigente (D-56) | Base 2.0.0 | Objetivo de cierre 2.1.0 | Resultado 2.1.0 |
|----|-----------|-------------|-----------------------|------------|--------------------------|-----------------|
| N-01 | Arranque (proceso → primera pantalla) | < 200 ms | < 200 ms | sin medir `[H]` | medido y dentro del límite | _pendiente_ |
| N-02 | Memoria en reposo | < 40 MB | < 40 MB | sin medir `[H]` | medido y dentro del límite | _pendiente_ |
| N-03 | Ejecutable release | < 15 MB | **< 18 MB** | 15 363 072 B (≈ 15,36 MB) | ≤ 15,4 MB (no crecer; F16-03 busca bajar) | _pendiente_ |
| N-04 | JS gzip | < 250 KB | **< 300 KB** | 246,4 KB | **≤ 285 KB** | _pendiente_ |
| — | 10 000 notas y 10 000 tareas en build **release** | — | lista < 250 ms, búsqueda < 50 ms, mes < 30 ms | solo build de depuración | medido en release (F16-05) | _pendiente_ |

**F16-03 (2026-11-18):** perfil release `opt-level = "s"` + `lto = "fat"` (antes `3` + `thin`):
`agendo.exe` 15 419 392 B → **10 516 992 B** (−4,9 MB). `panic = "abort"` descartado: `state.rs` depende de que el
pánico desenrolle y revierta la transacción abierta (D-24 y su test). El efecto en velocidad se mide en F16-05.

**F16-04 (2026-11-18):** la agenda no estaba virtualizada. Las listas de más de 200 filas se montan por trozos de 50
(`WindowedChunk`, `IntersectionObserver` con margen de 1500 px; altura medida al salir de pantalla, fila seleccionada siempre montada).
Prueba con 10 000 tareas: 150 filas en el DOM. La fluidez real al desplazar es `[H]` (F16-09).

**F16-05 (2026-11-18, build release con `opt-level = "s"` + LTO fat, `cargo test --release scale_tests -- --nocapture`):**

| Medida (10 000 elementos) | Límite | Resultado |
|---|---|---|
| Lista de todas las notas | < 250 ms | 19,7 ms |
| Lista de una carpeta | < 50 ms | 1,2 ms |
| Búsqueda (peor de 4) | < 50 ms | 12,1 ms |
| Abrir la base con 10 000 notas | < 200 ms | 2,1 ms |
| Tareas: POR HACER / AGENDA completa | < 250 ms | 12,6 ms / 11,7 ms |
| Tareas: un mes | < 30 ms | 1,5 ms |

En depuración la prueba de tareas usa 1 000 y no comprueba tiempos (comparte CPU con otras pruebas de tiempo).

**F16-06:** `npm run measure` (`scripts/measure-startup.ps1`) arranca la versión release varias veces con una carpeta de datos
temporal (`MIAGENDA_DATA_DIR`, sin tocar los datos reales) y da el tiempo hasta la ventana visible (N-01) y la memoria
en reposo (proceso principal y con WebView2, N-02). Exige cerrar antes cualquier instancia de Agendo (instancia única). La ejecución
real y su valor siguen siendo `[H]` (F16-09); la ventana visible aproxima, no sustituye, el primer pintado.

## Auditoría A0 — línea base (2026-11-19, commit 7486d25)

| Medida | Valor |
|---|---|
| JS total (gzip) | 268,3 KB (presupuesto de `npm run size`: 300 KB; el objetivo N-04 de SPEC es 250 KB) |
| JS carga inicial / chunks diferidos (gzip) | 86,4 KB / 181,9 KB |
| CSS total (gzip) | 26,0 KB |
| `agendo.exe` release | 10 823 680 B (10,32 MiB) |
| Tests Rust (`cargo test --lib`, depuración) | 457 correctos, 9,4 s |
| Tests TS (`vitest`) | 604 correctos en 55 archivos, 12,5 s |
| Escala 10 000 notas/tareas (release, F16-05) | todas las consultas muy por debajo del límite (≤ 20 ms) |

Tiempo por test de Rust: `--report-time` exige nightly; se medirá en A11-02 con otro método.
Arranque y memoria (`npm run measure`) siguen siendo `[H]`.

## Versión 3.0.0 — seguimiento (D-98, `PROPOSED`)

La 2.1.0 no se publica; los límites de D-56 siguen vigentes para la 3.0.0. La línea base real se anota en F18-00 y cada fase registra lo que consume.

| Métrica | Límite vigente | Línea base (F18-00) | Objetivo de cierre 3.0.0 | Resultado |
|---------|----------------|---------------------|--------------------------|-----------|
| JS gzip, carga inicial (N-04) | < 300 KB | _pendiente_ | dentro del límite, vistas nuevas con `import()` | _pendiente_ |
| Ejecutable release (N-03) | < 18 MB | _pendiente_ | dentro del límite | _pendiente_ |
| Arranque (N-01) `[H]` | < 200 ms | _pendiente_ | sin empeorar | _pendiente_ |
| Memoria en reposo (N-02) `[H]` | < 40 MB | _pendiente_ | sin empeorar | _pendiente_ |
| 10 000 tareas, 10 000 notas y eventos/hábitos en release | lista < 250 ms, búsqueda < 50 ms, mes < 30 ms | _pendiente_ | dentro del límite | _pendiente_ |

