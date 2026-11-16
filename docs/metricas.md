# Métricas de cumplimiento N-01 a N-08

Resultado de la revisión de F6-08 sobre el commit que la contiene. Fuente de los requisitos:
`docs/SPEC.md` §2. Las filas `[H]` necesitan un equipo Windows 11 real y una persona que
mida: el agente no las rellena (CLAUDE.md §5).

| ID | Requisito | Objetivo | Resultado | Estado | Cómo se midió |
|----|-----------|----------|-----------|--------|---------------|
| N-01 | Arranque (proceso → primera pantalla) | < 200 ms | _pendiente de medición_ | `[H]` | Marcas `startup_mark` en el log de depuración (`first_paint`), ver `docs/manual-pruebas.md` |
| N-02 | Memoria en reposo | < 40 MB (proceso principal) | _pendiente de medición_ | `[H]` | Administrador de tareas, proceso principal y suma con WebView2 |
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
