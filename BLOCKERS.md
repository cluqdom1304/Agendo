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
- **Estado:** ABIERTO
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
- **Estado:** ABIERTO
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

---

## Resueltos

(ninguno)
