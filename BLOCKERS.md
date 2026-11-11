# BLOCKERS.md — Bloqueos y decisiones pendientes

El agente registra aquí cualquier bloqueo, ambigüedad de producto o test que no puede pasar por causa externa. Cada entrada incluye causa, intentos realizados y opciones. Un humano cierra la entrada escribiendo la resolución y la fecha.

**Formato de entrada:**

```markdown
## B-XX — Título corto
- **Fecha:** AAAA-MM-DD
- **Tarea afectada:** F?-??
- **Tipo:** BLOQUEO_TECNICO | DECISION_PRODUCTO | ENTORNO | TEST_EXTERNO
- **Causa:**
- **Intentos:**
- **Opciones:** 1) … 2) …
- **Recomendación del agente:**
- **Estado:** ABIERTO | RESUELTO
- **Resolución humana:**
```

---

## Abiertos

## B-01 — Acción "Marcar como hecha" en notificaciones
- **Fecha:** 2026-10-31
- **Tarea afectada:** F4-05
- **Tipo:** DECISION_PRODUCTO
- **Causa:** la especificación la menciona como deseable, pero depende de la compatibilidad de `tauri-plugin-notification` con acciones en Windows 11, que solo puede confirmarse en una máquina real.
- **Opciones:**
  1. Incluirla en v1 si la versión del plugin la soporta; si no, abrir la app en la tarea.
  2. Posponerla a v2 y en v1 solo abrir la app en la tarea.
- **Recomendación del agente:** opción 2. Reduce riesgo en el worker de notificaciones, que es la parte más crítica de la fase 4.
- **Estado:** RESUELTO
- **Resolución humana:** añadir para la v2, por ahora no se hace en v1.

## B-02 — Formato de exportación además de `.sqlite`
- **Fecha:** 2026-10-31
- **Tarea afectada:** F6-04
- **Tipo:** DECISION_PRODUCTO
- **Causa:** la especificación solo exige copia `.sqlite`. No está claro si el usuario necesita un formato legible.
- **Opciones:**
  1. Solo `.sqlite` (copia completa y restaurable).
  2. `.sqlite` más exportación JSON (legible y portable, sin restauración directa en v1).
- **Recomendación del agente:** opción 1 para v1. Añadir JSON en v2 si se pide.
- **Estado:** RESUELTO
- **Resolución humana:** Por ahora, solo .sqlite, permitir importación en la app de este tipo de archivos para v1.

## B-03 — Tareas completadas dentro de POR HACER
- **Fecha:** 2026-10-31
- **Tarea afectada:** F1-05 (implementada con la opción 1), F2-03, F2-05
- **Tipo:** DECISION_PRODUCTO
- **Causa:** D-05 y F-05 definen la subsección "Completadas" al final del bloque de cada **día** de AGENDA, pero no dicen qué ocurre con una tarea completada que no tiene fecha (sección POR HACER, que solo se ordena por `position`).
- **Opciones:**
  1. POR HACER también termina con una subsección "Completadas", ordenada por `completed_at` ascendente, igual que un día.
  2. Las completadas sin fecha conservan su `position` entre las pendientes.
- **Recomendación del agente:** opción 1 (misma regla en todos los bloques; evita que una tarea hecha siga mezclada con las pendientes). Es la que implementa `domain::ordering::sort_todo`; cambiarla solo toca esa función y su test de tabla. El trabajo continúa con la opción 1 hasta que un humano decida.
- **Estado:** RESUELTO
- **Resolución humana:** Cuando una tarea sin fecha se complete, ajustar la fecha (y hora si procede) al momento que se ha marcado como completada, añadiendola como completada en el día correspondiente.

## B-04 — Hora de fin al crear arrastrando un tramo
- **Fecha:** 2026-11-09
- **Tarea afectada:** F5-03 (implementada con la opción 1)
- **Tipo:** DECISION_PRODUCTO
- **Causa:** F-11 habla de arrastrar un «tramo horario», pero el esquema (SPEC §5.2) solo tiene `due_time`; no hay hora de fin ni duración.
- **Opciones:**
  1. Guardar solo la hora de inicio del tramo; el arrastre sirve para elegirla y los bloques ocupan 30 min visuales.
  2. Migración nueva (`due_end_time` o duración) y bloques con su altura real; afecta a validación, DTO, ordenación y recordatorios.
- **Recomendación del agente:** opción 1 en v1 (no toca el modelo de datos); pasar a la 2 si se necesitan eventos con duración.
- **Estado:** RESUELTO
- **Resolución humana:** 2026-11-09, a petición del usuario: opción 2 (duración, ver D-33).

---

## B-05 — Organización de las notas de la sección «Notas»
- **Fecha:** 2026-11-11
- **Tarea afectada:** V2-F1-01, V2-F1-05
- **Tipo:** DECISION_PRODUCTO
- **Causa:** la sección de notas pedida para la V2 necesita una forma de ordenarlas y el modelo depende de ella.
- **Opciones:**
  1. Las mismas etiquetas de las tareas (una por nota, mismo filtro lateral).
  2. Carpetas propias de las notas, independientes de las etiquetas.
- **Recomendación del agente:** opción 1: un solo concepto que aprender, la barra lateral ya existe y una asignatura agrupa sus tareas y sus apuntes.
- **Estado:** ABIERTO

## B-06 — Imágenes y adjuntos en las notas
- **Fecha:** 2026-11-11
- **Tarea afectada:** V2-F1-04
- **Tipo:** DECISION_PRODUCTO
- **Causa:** un editor de notas completo suele admitir imágenes; guardarlas afecta al tamaño de la base, a las copias de seguridad y a la RAM.
- **Opciones:**
  1. Solo texto con formato en la 2.0; adjuntos en una versión posterior.
  2. Imágenes en la 2.0, guardadas como archivos junto a la base y referenciadas desde la nota.
- **Recomendación del agente:** opción 1: protege los presupuestos de rendimiento y deja la 2.0 centrada en escribir rápido.
- **Estado:** ABIERTO

## B-07 — Actualizaciones de la aplicación
- **Fecha:** 2026-11-11
- **Tarea afectada:** despliegue de la V2
- **Tipo:** DECISION_PRODUCTO
- **Causa:** con varias versiones previstas, instalar cada una a mano es más trabajo; comprobar actualizaciones exige red, que el proyecto prohíbe en ejecución.
- **Opciones:**
  1. Seguir con instalación manual del `.exe` sobre la versión anterior.
  2. Comprobación automática de actualizaciones (plugin updater, servidor de versiones y firma): introduce llamadas de red.
- **Recomendación del agente:** opción 1: mantiene «100 % local, sin red».
- **Estado:** ABIERTO

## B-08 — Semántica de las tareas periódicas
- **Fecha:** 2026-11-11
- **Tarea afectada:** V2-F2-01
- **Tipo:** DECISION_PRODUCTO
- **Causa:** D-07 dejó la recurrencia fuera de la v1; hay dos modelos habituales y cambian el esquema.
- **Opciones:**
  1. Una sola tarea con regla de repetición: al completarla aparece la siguiente.
  2. Generar de antemano todas las repeticiones como tareas independientes (visibles en el calendario futuro).
- **Recomendación del agente:** opción 1, mostrando en el calendario las repeticiones futuras calculadas al vuelo, sin guardarlas.
- **Estado:** ABIERTO

## Resueltos

(ninguno)
