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
- **Resolución humana:**

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
- **Resolución humana:**

---

## Resueltos

(ninguno)
