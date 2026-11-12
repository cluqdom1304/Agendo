# CLAUDE.md — Agendo

Eres el agente de desarrollo de **Agendo**, una agenda y editor de notas de escritorio para Windows 11 construida con **Tauri v2 (Rust)** y **Svelte 5 (TypeScript)**. El proyecto es 100 % local: sin cuentas, sin telemetría y sin red en tiempo de ejecución.

## 1. Lectura obligatoria al inicio de cada sesión

En este orden:

1. `PROGRESS.md` — fase activa, siguiente tarea, estado.
2. `BLOCKERS.md` — bloqueos y decisiones pendientes de humano.
3. `DECISIONS.md` — decisiones aceptadas. Prevalecen sobre cualquier instrucción anterior de la especificación.
4. `docs/SPEC.md` — solo las secciones relevantes a la tarea (ver índice abajo).

Si una tarea contradice `DECISIONS.md`, sigue la decisión registrada y anota la discrepancia en `PROGRESS.md`.

### Índice de `docs/SPEC.md`
| Sección | Contenido |
|---------|-----------|
| 1–2 | Visión, alcance, requisitos F-xx y N-xx |
| 3 | Stack tecnológico |
| 4 | Arquitectura y capas |
| 5 | Modelo de datos y migraciones |
| 6 | Sistema de recordatorios |
| 7 | Contrato IPC (errores, DTOs, comandos, eventos) |
| 8 | Sistema de diseño |
| 9 | Atajos de teclado |
| 10 | Estructura del repositorio |
| 11 | Decisiones de diseño (resumen de D-01 a D-10) |
| 12 | Hoja de ruta por fases con tareas y DoD |
| 13 | Estrategia de pruebas |
| 14 | Reglas de trabajo autónomo (ampliadas en este archivo) |

## 2. Ciclo de trabajo por tarea

1. Localiza la **primera tarea no marcada** de la fase activa en `PROGRESS.md`.
2. Lee la sección de `docs/SPEC.md` que la afecta y las decisiones relacionadas.
3. Escribe un plan de 3–6 líneas: archivos, tests y riesgos.
4. Implementa en un cambio coherente.
5. Ejecuta la batería de verificación (sección 4). Si algo falla, corrige antes de seguir.
6. Marca la tarea en `PROGRESS.md` y anota resultados y métricas.
7. Haz un commit con **Conventional Commits** (`feat:`, `fix:`, `test:`, `refactor:`, `docs:`, `chore:`). Una tarea lógica = un commit.
8. Continúa con la siguiente tarea.

## 3. Cuándo parar y cuándo seguir

- **Bloqueo puntual** (fallo de causa desconocida, dependencia no disponible): registra en `BLOCKERS.md` la causa, los intentos y las opciones. Continúa con la siguiente tarea **que no dependa** de ella.
- **Ambigüedad de producto**: no decidas en silencio. Regístrala en `BLOCKERS.md` con dos opciones y una recomendación. Si la tarea depende de ella, cambia de tarea.
- **Contradicción entre especificación y necesidad técnica**: no edites `docs/SPEC.md`. Crea un ADR con estado `PROPOSED` en `DECISIONS.md` y espera confirmación si afecta a más de una tarea.
- **Fin de fase**: cuando todas las tareas `[A]` estén en verde, pon `Estado: AWAITING_HUMAN` en `PROGRESS.md`, lista los ítems `[H]` de `docs/manual-pruebas.md` correspondientes a la fase y **detente**. No inicies la fase siguiente sin confirmación humana.

## 4. Batería de verificación (obligatoria antes de marcar cualquier tarea)

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run lint
npm run check
npm run test
npm run build
npm run size
```

Una tarea **no** se marca como hecha si cualquiera de estos comandos falla. Si un comando aún no existe en esa fase (p. ej. `size` antes de F0-09), anótalo en `PROGRESS.md` y no lo consideres fallo.

## 5. Etiquetas de verificación

- `[A]` — verificable por el agente en su entorno (Linux). Se verifica ejecutando tests y comandos.
- `[H]` — requiere Windows 11 real y validación humana. **Nunca** la marques como verificada. Anótala en `docs/manual-pruebas.md`.

## 6. Reglas de código (inquebrantables)

- **Cero placeholders:** prohibido `TODO`, `FIXME`, `todo!()`, `unimplemented!()`, funciones vacías o comentarios tipo "implementar luego". Si algo no puede hacerse, no se crea: se registra en `BLOCKERS.md`.
- **Errores Rust:** ningún `unwrap()` ni `expect()` fuera de tests o de inicialización irrecuperable (con mensaje explicativo). Los comandos IPC devuelven `Result<T, String>` con prefijo de código estable (`NOT_FOUND:`, `VALIDATION:`, `DB:`, `IO:`, `NOTIFY:`, `CONFLICT:`, `INTERNAL:`), según D-08.
- **Svelte 5:** solo runes (`$state`, `$derived`, `$effect`, `$props`). Prohibido `$:` y `export let` en código nuevo. El estado compartido vive en módulos `.svelte.ts`.
- **Rendimiento de UI:** lo calculable debe ser `$derived`, nunca lógica en el template. Las filas reciben solo los campos que necesitan.
- **SQL:** consultas parametrizadas siempre. Prohibido `format!` con datos del usuario.
- **Tiempo:** nunca llamar a `Utc::now()` fuera de la implementación real de `Clock`. Toda conversión local/UTC pasa por `domain/dates.rs`.
- **Seguridad:** enlaces validados (solo `http`, `https`, `mailto`). Permisos de Tauri mínimos en `capabilities/default.json`.
- **Dependencias:** ninguna nueva sin entrada en `DECISIONS.md` (motivo, versión, alternativa descartada).
- **Idioma:** código e identificadores en inglés; textos de interfaz y mensajes al usuario en español, centralizados en `src/lib/i18n.ts`.
- **Comentarios:** solo para explicar el *porqué*.

## 7. Lo que nunca debes hacer

- Marcar como verificado algo `[H]` sin validación humana.
- Modificar `docs/SPEC.md` sin ADR aprobado.
- Desactivar, borrar o marcar con `#[ignore]` / `it.skip` tests para que la batería pase (salvo entrada en `BLOCKERS.md`).
- Escribir migraciones destructivas (`DROP`, `DELETE` masivo) sobre datos de usuario. Cada cambio de esquema es una migración nueva numerada.
- Añadir telemetría, analítica o llamadas de red desde la aplicación.
- Saltarse hooks de git (`--no-verify`) o hacer `git push --force` sobre `main`.
- Dar por hecho que la UI funciona en Windows: esa verificación es `[H]`.

## 8. Entorno del agente

- Linux. No puede ejecutar ventanas, Mica, WebView2 ni notificaciones de Windows.
- Red: solo registros de paquetes (crates.io, npm). Ninguna llamada de red de la aplicación.
- Si el toolchain de Windows está disponible, `cargo check --target x86_64-pc-windows-msvc` debe ejecutarse en Fase 0 y en cada fase que toque código `window/` o `notify/`. Si no está disponible, anota en `PROGRESS.md` que queda pendiente.

## 9. Formato del informe final de cada sesión

Al terminar, deja en la respuesta, como máximo:

- Tareas completadas (IDs).
- Verificaciones `[A]` ejecutadas y resultado.
- Nuevos ítems `[H]` que requieren validación humana.
- Bloqueos nuevos (referencia a `BLOCKERS.md`).
- Siguiente tarea propuesta.

## Ahorro de contexto
- Antes de leer un archivo entero, busca con grep y lee solo lo que necesites.
- Para explorar el proyecto, usa un subagente y tráeme solo el resumen.
- Muéstrame el diff, no el archivo completo.
- No repitas lo que ya hemos decidido en esta sesión.
- Si cambiamos de tarea, propón /compact o una sesión nueva.
- Respuestas cortas por defecto.

## 10. Publicar una versión

Cada versión nueva (`X.Y.Z`) cierra con estos pasos, en este orden:

1. Versión idéntica en `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` y los dos lockfiles; entrada en `PROGRESS.md`.
2. Batería de verificación (sección 4) en verde y commit de versión (`chore:` o `feat:` con `vX.Y.Z` en el mensaje).
3. Etiqueta anotada sobre ese commit: `git tag -a vX.Y.Z -m "Versión X.Y.Z"`. Una etiqueta por versión, nunca se mueve ni se reescribe.
4. Subir commit y etiqueta: `git push origin main vX.Y.Z`.
5. Release de GitHub con `gh release create vX.Y.Z --title "Agendo X.Y.Z" --notes "<cambios>"`, con las notas tomadas de la entrada de `PROGRESS.md` y adjuntando `Agendo_X.Y.Z_x64-setup.exe` y `Agendo_X.Y.Z_x64_es-ES.msi` si están generados (sin firma).

Si `gh` no está disponible, haz los pasos 1–4 y deja la release como pendiente en `PROGRESS.md`.
