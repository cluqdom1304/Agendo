# MiAgendaApp — Especificación, decisiones, hoja de ruta y guía de trabajo autónomo

> **Versión:** 1.0 · **Idioma del proyecto:** español (UI, comentarios de negocio y documentación). Identificadores de código en inglés.
> **Ubicación recomendada:** `docs/SPEC.md` dentro del repositorio. La raíz contiene `CLAUDE.md` (plantilla en el Apéndice A) y `PROGRESS.md` (plantilla en el Apéndice B).

---

## 0. Cómo debe leer este documento un agente (Claude)

Orden obligatorio al iniciar cualquier sesión de trabajo:

1. `CLAUDE.md` (reglas de ejecución, sección 14 de este documento resumida).
2. `PROGRESS.md` (estado actual, siguiente tarea, bloqueos abiertos).
3. Esta especificación, **solo** las secciones relevantes a la tarea actual.
4. `DECISIONS.md` (registro de decisiones; prevalece sobre cualquier afirmación anterior de este documento si hay contradicción, y debe reportarse la contradicción).

Si una instrucción de esta especificación y una decisión registrada en `DECISIONS.md` chocan, se sigue la decisión registrada y se anota la discrepancia en `PROGRESS.md`.

---

## 1. Visión y alcance

**Visión:** un editor plano estilo Notepad++ con estructura de agenda (secciones "POR HACER" y "AGENDA" con fechas descendentes), con la potencia de una agenda moderna (checkboxes, etiquetas por asignatura, vistas de calendario, recordatorios nativos) y una estética inspirada en Apple HIG integrada en Windows 11.

**Plataforma objetivo:** Windows 11 (23H2 o superior). Windows 10 es "mejor esfuerzo" (sin Mica; se usa Acrylic o fondo sólido).

**Principios:**
- **100 % local.** Sin cuentas, sin telemetría, sin llamadas de red en tiempo de ejecución.
- **Ligera.** Arranque rápido, poca memoria, binario pequeño.
- **Robusta antes que cómoda.** Los datos del usuario nunca se pierden ni se corrompen; las notificaciones no se pierden aunque la app se cierre o el equipo esté suspendido.
- **Simple.** Cada función debe justificarse frente a la visión de "editor plano".

### 1.1 Alcance v1 (incluido)
- Lista/documento con secciones "POR HACER", "AGENDA" y subsección "Completadas" por día.
- Tareas con título, checkbox, fecha opcional, hora opcional, etiquetas (asignaturas) y notas enriquecidas.
- Notas con negrita, cursiva, código en línea, enlaces y listas (sub-puntos).
- Vistas: Lista, Día, Semana, Mes.
- Creación rápida de tareas con atajos de teclado.
- Búsqueda full-text (Ctrl+K).
- Recordatorios con notificaciones nativas de Windows 11.
- Tema oscuro (por defecto) y claro, que sigue al sistema.
- Exportación/copia de seguridad local de la base de datos (archivo `.sqlite`).

### 1.2 Fuera de alcance v1 (no implementar sin ADR nuevo)
- Sincronización en la nube, multiusuario, colaboración.
- Adjuntos binarios (imágenes, PDF).
- Recurrencia completa (solo se reserva la columna, ver D-07).
- Importación de Notion, Google Calendar, ICS (posible v2).
- Aplicaciones móviles o macOS/Linux (la arquitectura no debe impedirlo, pero no se prueba).

---

## 2. Requisitos

### 2.1 Funcionales

| ID | Requisito | Prioridad |
|----|-----------|-----------|
| F-01 | Crear, editar (título inline y notas), completar, reabrir y eliminar tareas. | Alta |
| F-02 | Eliminación con papelera temporal (soft delete, 30 días) y opción de deshacer 5 s. | Alta |
| F-03 | Sección "POR HACER": tareas sin fecha, orden manual (`position`). | Alta |
| F-04 | Sección "AGENDA": bloques por día, fechas descendentes `DD/MM/YYYY`. Tareas sin hora dentro del día ordenadas por `position`; con hora, por hora ascendente antes que las sin hora. | Alta |
| F-05 | Al completar una tarea, animación de salida y reubicación en "Completadas" al final del bloque de su día. | Alta |
| F-06 | Etiquetas (asignaturas): crear, renombrar, color, eliminar; asignar N por tarea; filtrar por etiqueta en sidebar con contador de pendientes. | Alta |
| F-07 | Sidebar retráctil con etiquetas y contadores. Atajo para ocultarla. | Media |
| F-08 | Editor de notas: negrita, cursiva, código, enlace, listas. Guardado automático con *debounce* de 400 ms. | Alta |
| F-09 | Búsqueda full-text sobre título y notas, con resultados en < 100 ms para 10 000 tareas. | Alta |
| F-10 | Vista Calendario con selector Lista / Día / Semana / Mes. | Media |
| F-11 | Creación rápida de eventos arrastrando un tramo horario en Día/Semana. | Media |
| F-12 | Recordatorios: "En el momento", "15 min antes", "1 hora antes", "1 día antes" o hora personalizada. Varios por tarea. | Alta |
| F-13 | Notificaciones nativas de Windows 11 al disparar un recordatorio, con acción "Marcar como hecha" si la plataforma lo permite. | Alta |
| F-14 | Recordatorios perdidos (equipo apagado o suspendido) se disparan al arrancar si la pérdida fue < 24 h; si fue mayor, se marcan como enviados sin notificar y se registran en log. | Alta |
| F-15 | Atajos de teclado completos (ver sección 7.4). | Media |
| F-16 | Copia de seguridad manual y restauración desde `.sqlite`, con validación de integridad antes de sustituir. | Media |

### 2.2 No funcionales (objetivos medibles)

| ID | Atributo | Objetivo | Cómo se verifica |
|----|----------|----------|------------------|
| N-01 | Arranque (proceso → primera pantalla pintada) | < 200 ms en equipo de referencia, con WebView2 ya instalado | Medición manual con marcas de tiempo en log de debug **[H]** |
| N-02 | Memoria en reposo | Objetivo < 40 MB para el proceso principal; el total incluye procesos WebView2 y puede superar ese valor | Administrador de tareas, suma de procesos **[H]** |
| N-03 | Tamaño del ejecutable (release, sin instalador) | < 15 MB | `ls -l` tras `cargo tauri build` **[A]** |
| N-04 | Presupuesto de JS de la app (gzip, sin WebView) | < 250 KB | Script `npm run size` **[A]** |
| N-05 | Consultas de agenda (un mes) | < 30 ms con 10 000 tareas | Test de integración con datos generados **[A]** |
| N-06 | Integridad de datos | Ninguna pérdida con cierre forzado durante escritura | Test de corte (kill) + `PRAGMA integrity_check` **[A]** |
| N-07 | Precisión de recordatorios | Disparo ≤ 5 s tras la hora, con equipo despierto | Test con reloj inyectado **[A]** y prueba real **[H]** |
| N-08 | Accesibilidad | Contraste WCAG AA en texto; navegación completa por teclado; `prefers-reduced-motion` respetado | Revisión y test de componentes **[A]** / manual **[H]** |

Leyenda: **[A]** verificable por el agente en su entorno; **[H]** requiere una máquina Windows 11 real y validación humana.

---

## 3. Stack tecnológico

| Capa | Tecnología | Notas |
|------|------------|-------|
| Shell de escritorio | Tauri v2 (Rust) | Usar la versión 2.x estable más reciente en el momento de crear el proyecto; registrar la versión exacta en `DECISIONS.md`. |
| Frontend | Svelte 5 + Vite + TypeScript | Solo runes (`$state`, `$derived`, `$effect`, `$props`). Prohibido el API legacy `$:` y `export let` en código nuevo. |
| Estilos | Tailwind CSS v4 (plugin Vite) + tokens CSS propios | Tokens en `:root`; Tailwind solo para utilidades de layout. |
| Editor | TipTap Core + extensiones mínimas (Document, Paragraph, Text, Bold, Italic, Code, Link, BulletList, OrderedList, ListItem, History, Placeholder) | Carga diferida (`import()`) al abrir una nota. |
| Persistencia | SQLite vía `rusqlite` con feature `bundled` | Modo WAL, `foreign_keys=ON`, `synchronous=NORMAL`. |
| Búsqueda | Tabla virtual FTS5 propia (ver 5.3) | Sincronizada desde la capa repositorio en la misma transacción. |
| Recordatorios | `tokio` (hilo de scheduler) + `tauri-plugin-notification` | Ver 6. |
| Efectos de ventana | API nativa de Tauri (`set_effects`) con Mica / Acrylic | Ver D-01. |
| Tests Rust | `cargo test`, base de datos en memoria | Reloj inyectable (trait `Clock`). |
| Tests frontend | Vitest + `@testing-library/svelte` | Componentes y stores/runes puros. |
| E2E (fase 6) | WebdriverIO + `tauri-driver` | Solo en Windows **[H]**. |
| Calidad | `rustfmt`, `clippy -D warnings`, `svelte-check`, ESLint, Prettier | Todos obligatorios en CI local. |

**Regla de dependencias:** cualquier dependencia nueva requiere una entrada en `DECISIONS.md` con motivo, versión y alternativa descartada. Preferir la biblioteca estándar y las ya listadas.

---

## 4. Arquitectura

### 4.1 Vista general

```
┌────────────────────────── Ventana Tauri (frameless, transparente) ──────────────────────────┐
│  Frontend (Svelte 5)                                                                       │
│  ├─ shell/        Titlebar, Sidebar, SegmentedControl, Spotlight (Ctrl+K)                  │
│  ├─ features/     todo/ · agenda/ · calendar/ · editor/ · reminders/ · tags/ · settings/   │
│  ├─ lib/          api.ts (único punto IPC) · stores/*.svelte.ts · dates.ts · keymap.ts     │
│  └─ styles/       tokens.css · base.css                                                    │
│                              │ invoke() / listen()                                         │
│  Backend (Rust)              ▼                                                             │
│  ├─ commands/     Capa IPC: validación y mapeo a DTO (sin lógica de negocio)              │
│  ├─ domain/       Reglas puras: ordenación, validación de fechas, cálculo de recordatorios │
│  ├─ repo/         Acceso SQLite (una transacción por operación lógica)                     │
│  ├─ db/           Conexión, migraciones (user_version), backups                           │
│  ├─ scheduler/    Hilo Tokio de recordatorios + trait Clock                                │
│  ├─ notify/       Adaptador de notificaciones nativas                                      │
│  ├─ window/       Efectos Mica/Acrylic y configuración de ventana                          │
│  └─ error.rs      Tipo de error y conversión a String                                      │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

**Regla de capas:** `commands` → `domain` / `repo` → `db`. El frontend nunca habla con SQLite. Ningún módulo de `domain` depende de `tauri`, lo que permite testearlo sin ventana.

### 4.2 Frontend: flujo de datos
- **Fuente de verdad:** la base de datos. El frontend mantiene una copia en stores `$state` que se refresca con `invoke` tras cada mutación y ante el evento `tasks:changed`.
- **Actualizaciones optimistas** solo en checkbox y título inline; si el backend falla, se revierte y se muestra un toast.
- **Un único módulo IPC** (`lib/api.ts`) tipado con los DTO de la sección 7.
- **Derivados** (agrupación por día, contadores de sidebar, filas visibles) con `$derived`, nunca calculados en el template.
- **Listas grandes:** virtualización manual por ventana de filas si el número de bloques visibles supera 200.

### 4.3 Backend: concurrencia
- Una única conexión de escritura protegida por `Mutex` (SQLite admite un escritor). Lecturas con conexión de solo lectura separada en modo WAL.
- Los comandos son `async` y delegan el trabajo bloqueante a `tauri::async_runtime::spawn_blocking`.
- El scheduler posee su propia conexión de escritura, serializada con la misma `Mutex` mediante un `Arc<Db>`.

---

## 5. Modelo de datos

### 5.1 Principios
- Fechas de negocio (`due_date`) en **local, sin zona**: `YYYY-MM-DD`. Hora de negocio (`due_time`) en `HH:MM` local.
- Instantes técnicos (`created_at`, `notify_at`, `completed_at`) en **UTC** ISO 8601 `YYYY-MM-DDTHH:MM:SSZ`.
- Ids: UUID v7 en texto (ordenables por tiempo).
- Versión de esquema en `PRAGMA user_version`.

### 5.2 Migración 001 (`src-tauri/migrations/001_init.sql`)

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE tags (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 60),
  color_hex   TEXT NOT NULL CHECK (color_hex GLOB '#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]'),
  parent_id   TEXT NULL REFERENCES tags(id) ON DELETE SET NULL,
  position    INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT NOT NULL
);
CREATE UNIQUE INDEX ux_tags_name ON tags(name COLLATE NOCASE);

CREATE TABLE tasks (
  id            TEXT PRIMARY KEY,
  title         TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 500),
  content_json  TEXT NOT NULL DEFAULT '{"type":"doc","content":[]}',
  content_text  TEXT NOT NULL DEFAULT '',
  due_date      TEXT NULL CHECK (due_date IS NULL OR due_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
  due_time      TEXT NULL CHECK (due_time IS NULL OR due_time GLOB '[0-2][0-9]:[0-5][0-9]'),
  is_completed  INTEGER NOT NULL DEFAULT 0 CHECK (is_completed IN (0,1)),
  completed_at  TEXT NULL,
  position      INTEGER NOT NULL DEFAULT 0,
  rrule         TEXT NULL,            -- reservado para recurrencia (D-07); no se usa en v1
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT NULL
);
CREATE INDEX ix_tasks_due      ON tasks(due_date)     WHERE deleted_at IS NULL;
CREATE INDEX ix_tasks_todo     ON tasks(position)     WHERE deleted_at IS NULL AND due_date IS NULL;
CREATE INDEX ix_tasks_deleted  ON tasks(deleted_at);

CREATE TABLE task_tags (
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
  PRIMARY KEY (task_id, tag_id)
);
CREATE INDEX ix_task_tags_tag ON task_tags(tag_id);

CREATE TABLE reminders (
  id              TEXT PRIMARY KEY,
  task_id         TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  notify_at       TEXT NOT NULL,       -- UTC
  offset_minutes  INTEGER NULL,        -- NULL = hora personalizada
  is_sent         INTEGER NOT NULL DEFAULT 0 CHECK (is_sent IN (0,1)),
  sent_at         TEXT NULL,
  created_at      TEXT NOT NULL
);
CREATE INDEX ix_reminders_pending ON reminders(is_sent, notify_at);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- FTS5 independiente (no external-content) para evitar problemas de rowid con ids TEXT.
CREATE VIRTUAL TABLE tasks_fts USING fts5(
  task_id UNINDEXED,
  title,
  content_text,
  tokenize = 'unicode61 remove_diacritics 2'
);
```

### 5.3 Reglas de integridad
- **Sincronización FTS:** cada escritura de `tasks` (insert/update/soft-delete) actualiza `tasks_fts` en la **misma transacción**: `DELETE FROM tasks_fts WHERE task_id = ?` y `INSERT` si `deleted_at IS NULL`.
- **Profundidad de etiquetas:** máximo 1 nivel (`parent_id` de una etiqueta no puede tener `parent_id`). Validado en `domain`.
- **Completar** pone `is_completed=1` y `completed_at=now()`; reabrir pone ambos a `0/NULL`.
- **Eliminación:** soft delete (`deleted_at`). Purga física de tareas con `deleted_at` > 30 días al arrancar.
- **Recordatorios:** al cambiar `due_date`/`due_time` de una tarea, los recordatorios con `offset_minutes` se recalculan; los `is_sent=1` no se tocan.
- **Migraciones:** en orden, cada una en transacción; si falla, la base no cambia y la app muestra error bloqueante con ruta del backup.

### 5.4 Pragmas de conexión
`journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `busy_timeout=5000`, `temp_store=MEMORY`.

---

## 6. Sistema de recordatorios

### 6.1 Diseño
Un hilo Tokio dedicado calcula el próximo `notify_at` pendiente (`SELECT MIN(notify_at) FROM reminders WHERE is_sent=0`) y espera con `tokio::select!` entre:
- `sleep_until(min(próximo_notify_at, ahora + 60 s))`. El tope de 60 s es deliberado: detecta saltos de reloj, suspensión y cambios de zona horaria sin depender de eventos del sistema.
- `Notify` de reprogramación, disparado tras cualquier mutación de recordatorios o de fechas de tareas.

### 6.2 Trait de reloj
```rust
pub trait Clock: Send + Sync {
    fn now_utc(&self) -> chrono::DateTime<chrono::Utc>;
}
```
Implementación real con `Utc::now()`; implementación de test que avanza manualmente. **Toda lógica temporal del scheduler usa `Clock`, nunca `Utc::now()` directamente.**

### 6.3 Disparo
1. Seleccionar recordatorios con `is_sent=0 AND notify_at <= now`.
2. Para cada uno: en una transacción, marcar `is_sent=1, sent_at=now`, **después** llamar a la notificación nativa. Si la notificación falla, revertir a `is_sent=0` y reintentar con backoff exponencial (30 s, 60 s, 120 s, máximo 5 intentos; luego registrar en log y marcar como enviado).
   - Justificación: se prefiere un reintento a una notificación perdida. Un duplicado tras fallo del sistema es aceptable.
3. Emitir `reminder:fired` al frontend para la vista in-app.

### 6.4 Arranque y recuperación (F-14)
- Recordatorios con `notify_at` entre `now-24h` y `now` y `is_sent=0`: se disparan al arrancar.
- Anteriores a `now-24h`: `is_sent=1`, `sent_at=now`, sin notificación; se registran en log.

### 6.5 Notificaciones
- Título: nombre de la tarea. Cuerpo: etiqueta(s) y hora si existe.
- Acción "Marcar como hecha" cuando la plataforma la soporte; si no, solo abrir la app en la tarea.
- Requiere `identifier` estable en `tauri.conf.json` para que Windows agrupe y gestione las notificaciones **[H]**.

---

## 7. Contrato IPC (frontend ↔ Rust)

### 7.1 Errores
Todos los comandos devuelven `Result<T, String>`. El `String` tiene formato estable `CODIGO: mensaje legible`, p. ej. `NOT_FOUND: tarea 0193...`, `VALIDATION: título demasiado largo`, `DB: ...`. El frontend parsea el prefijo para decidir el mensaje mostrado; **el código nunca se muestra crudo al usuario**.

Códigos válidos: `NOT_FOUND`, `VALIDATION`, `CONFLICT`, `DB`, `IO`, `NOTIFY`, `INTERNAL`.

### 7.2 DTO (TypeScript y Rust con `serde` `rename_all = "camelCase"`)

```ts
type Task = {
  id: string;
  title: string;
  contentJson: string;      // ProseMirror JSON serializado
  dueDate: string | null;   // YYYY-MM-DD
  dueTime: string | null;   // HH:MM
  isCompleted: boolean;
  completedAt: string | null;
  position: number;
  tagIds: string[];
  reminders: Reminder[];
  createdAt: string;
  updatedAt: string;
};

type Tag = { id: string; name: string; colorHex: string; parentId: string | null; position: number };
type Reminder = { id: string; notifyAt: string; offsetMinutes: number | null; isSent: boolean };
```

### 7.3 Comandos

| Comando | Entrada | Salida | Notas |
|---------|---------|--------|-------|
| `tasks_list` | `{ view: "todo" \| "agenda" \| "range", from?, to?, tagId? }` | `{ todo: Task[], agenda: DayGroup[] }` o `Task[]` | Agrupación por día en backend para la vista agenda. |
| `task_create` | `{ title?, dueDate?, dueTime?, tagIds?, contentJson? }` | `Task` | Posición al final de su sección. |
| `task_update` | `{ id, patch: Partial<...> }` | `Task` | Patch parcial; valida fechas. |
| `task_set_completed` | `{ id, completed: boolean }` | `Task` | Actualiza `completed_at`. |
| `task_move` | `{ id, toSection: "todo" \| "agenda", date?, beforeId? }` | `Task` | Reordenación por arrastre. |
| `task_delete` | `{ id }` | `()` | Soft delete. |
| `task_restore` | `{ id }` | `Task` | Desde papelera. |
| `trash_list` | `()` | `Task[]` | |
| `tag_list` | `()` | `TagWithCount[]` | Incluye pendientes por etiqueta. |
| `tag_upsert` | `{ id?, name, colorHex, parentId? }` | `Tag` | |
| `tag_delete` | `{ id }` | `()` | Desvincula tareas. |
| `task_tags_set` | `{ taskId, tagIds }` | `Task` | |
| `reminders_set` | `{ taskId, reminders: { offsetMinutes?, notifyAt? }[] }` | `Task` | Reemplaza el conjunto. |
| `search` | `{ query, limit? }` | `SearchHit[]` | `limit` por defecto 50, máximo 200. |
| `settings_get` / `settings_set` | `{ key }` / `{ key, value }` | `string?` / `()` | Claves whitelisted. |
| `backup_create` | `{ path }` | `()` | `VACUUM INTO`. |
| `backup_restore` | `{ path }` | `()` | Valida con `integrity_check` antes de sustituir. |
| `window_set_effect` | `{ mode: "mica" \| "acrylic" \| "none" }` | `EffectApplied` | Devuelve el efecto realmente aplicado (fallback). |

### 7.4 Eventos (backend → frontend)
- `tasks:changed` `{ ids: string[] }`
- `reminder:fired` `{ taskId, reminderId }`
- `db:error` `{ message }`

---

## 8. Sistema de diseño

### 8.1 Ventana
- `decorations: false`, `transparent: true`, `shadow: true`, tamaño por defecto 1100×720, mínimo 720×480.
- Radio de esquina 10 px en el contenedor raíz; borde superior `1px solid rgba(255,255,255,0.08)`.
- Zona de arrastre: `data-tauri-drag-region` solo en titlebar, nunca en inputs ni botones.
- Permisos mínimos en `src-tauri/capabilities/default.json` (`core:window:allow-start-dragging`, `allow-minimize`, `allow-toggle-maximize`, `allow-close`, `allow-set-effects` y los necesarios para notificaciones y diálogo de archivos). **Nada más.**

### 8.2 Efectos (D-01)
- Efecto principal: **Mica** (Windows 11). Si no está disponible, **Acrylic**. Si ninguno, fondo sólido.
- Cuando hay efecto nativo activo, el CSS aplica `background: rgba(22,22,26,0.35)` (tinte) **sin** `backdrop-filter`. Cuando el efecto es `none`, se usa `background: rgba(22,22,26,0.92)`.
- Clase raíz `.native-effect` controla la variante.

### 8.3 Tokens (`styles/tokens.css`)

```css
:root {
  --font-ui: "SF Pro Display", "SF Pro Text", "Segoe UI Variable Display", "Segoe UI Variable Text", Inter, "Segoe UI", system-ui, sans-serif;
  --font-mono: "SF Mono", "Cascadia Code", Consolas, monospace;

  --radius-window: 10px;
  --radius-control: 8px;
  --radius-chip: 999px;

  --motion-fast: 150ms;
  --ease-apple: cubic-bezier(0.16, 1, 0.3, 1);
  --ease-elastic: cubic-bezier(0.34, 1.56, 0.64, 1);

  --space-1: 4px; --space-2: 8px; --space-3: 12px; --space-4: 16px; --space-6: 24px; --space-8: 32px;

  --scrollbar-width: 4px;
}
:root[data-theme="dark"], :root:not([data-theme="light"]) {
  --bg-window: rgba(22, 22, 26, 0.70);
  --bg-elevated: rgba(44, 44, 50, 0.72);
  --bg-hover: rgba(255, 255, 255, 0.06);
  --text-primary: #F5F5F7;
  --text-secondary: #A1A1A6;   /* contraste ≥ 4.5:1 sobre --bg-window opaco equivalente */
  --accent: #0A84FF;
  --border-subtle: rgba(255, 255, 255, 0.08);
  --danger: #FF453A;
  --success: #30D158;
}
@media (prefers-color-scheme: light) {
  :root:not([data-theme="dark"]) {
    --bg-window: rgba(246, 246, 248, 0.72);
    --bg-elevated: rgba(255, 255, 255, 0.80);
    --bg-hover: rgba(0, 0, 0, 0.05);
    --text-primary: #1D1D1F;
    --text-secondary: #6E6E73;
    --accent: #0071E3;
    --border-subtle: rgba(0, 0, 0, 0.08);
    --danger: #D70015;
    --success: #248A3D;
  }
}
```

**Paleta de asignaturas:** 8 colores tenues definidos en tokens (`--tag-1` … `--tag-8`). El badge usa el color con 18 % de opacidad de fondo y 100 % en el texto. Se verifica contraste al crear la etiqueta.

### 8.4 Movimiento
- Duración base 150 ms, curva `--ease-apple` para hover, apertura de modales y aparición de elementos.
- Checkbox: `--ease-elastic` en el marcado (escala 0.85 → 1.1 → 1).
- Completar tarea: 220 ms de opacidad + desplazamiento vertical hacia "Completadas"; tras la animación se reordena el DOM.
- `@media (prefers-reduced-motion: reduce)`: duraciones a 0 ms, sin escalado elástico.

### 8.5 Scrollbars
4 px de ancho, `opacity: 0` en reposo, `opacity: 1` al hacer scroll o hover de la zona, `scrollbar-color` coherente con el tema.

### 8.6 Componentes base (inventario cerrado v1)
`TitleBar`, `WindowControls`, `Sidebar`, `TagChip`, `SectionHeader`, `DayGroup`, `TaskRow`, `Checkbox`, `InlineTitle`, `NoteEditor` (TipTap), `NoteToolbar`, `SegmentedControl`, `CalendarDay`, `CalendarWeek`, `CalendarMonth`, `ReminderPopover`, `Spotlight`, `Toast`, `Modal`, `EmptyState`.

Cualquier componente nuevo requiere justificación en `DECISIONS.md`.

### 8.7 Estructura de la fila de tarea
```
[○] Título editable inline                 [● DORC]   ⏰ 10:00
      Hecho hasta el punto 3.2              ← nota (sub-punto indentado, 24 px)
```

---

## 9. Atajos de teclado

| Atajo | Acción |
|-------|--------|
| `Ctrl+K` | Spotlight / búsqueda |
| `Ctrl+N` | Nueva tarea en la sección activa |
| `Ctrl+Shift+N` | Nueva tarea con fecha de hoy |
| `Enter` (en título) | Confirmar y crear siguiente tarea |
| `Espacio` (fila seleccionada) | Completar / reabrir |
| `Supr` | Eliminar (con deshacer 5 s) |
| `Ctrl+B / I / E` | Negrita / cursiva / código en editor |
| `Ctrl+L` | Insertar enlace |
| `Ctrl+\` | Mostrar / ocultar sidebar |
| `Ctrl+1 / 2 / 3 / 4` | Lista / Día / Semana / Mes |
| `Ctrl+Z` | Deshacer |
| `Esc` | Cerrar modal o salir de edición |

Los atajos se definen en `lib/keymap.ts`, una sola tabla, y no están codificados en componentes.

---

## 10. Estructura del repositorio

```
miagendaapp/
├─ CLAUDE.md                      # reglas de trabajo del agente (Apéndice A)
├─ PROGRESS.md                    # estado y siguiente tarea (Apéndice B)
├─ DECISIONS.md                   # ADRs (sección 11)
├─ BLOCKERS.md                    # bloqueos abiertos
├─ README.md                      # instrucciones para humanos
├─ docs/
│  ├─ SPEC.md                     # este documento
│  └─ manual-pruebas.md           # checklist [H] por fase
├─ package.json
├─ vite.config.ts
├─ tsconfig.json
├─ eslint.config.js
├─ .prettierrc
├─ src/                           # frontend Svelte
│  ├─ app.html
│  ├─ main.ts
│  ├─ App.svelte
│  ├─ styles/ (tokens.css, base.css)
│  ├─ lib/ (api.ts, dates.ts, keymap.ts, stores/, types.ts, i18n.ts)
│  ├─ shell/ (TitleBar, Sidebar, Spotlight, Toast, Modal)
│  └─ features/ (todo/, agenda/, calendar/, editor/, reminders/, tags/, settings/)
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ build.rs
│  ├─ capabilities/default.json
│  ├─ migrations/ (001_init.sql, ...)
│  └─ src/
│     ├─ main.rs
│     ├─ lib.rs
│     ├─ error.rs
│     ├─ state.rs
│     ├─ commands/ (tasks.rs, tags.rs, reminders.rs, search.rs, settings.rs, backup.rs, window.rs)
│     ├─ domain/ (dates.rs, ordering.rs, validation.rs, reminders.rs)
│     ├─ repo/ (tasks.rs, tags.rs, reminders.rs, fts.rs, settings.rs)
│     ├─ db/ (mod.rs, migrate.rs, pragmas.rs, backup.rs)
│     ├─ scheduler/ (mod.rs, clock.rs)
│     ├─ notify/ (mod.rs)
│     └─ window/ (effects.rs)
├─ tests/ (e2e/ — fase 6)
└─ scripts/ (size-check.mjs, bench-startup.ps1)
```

### 10.1 Perfil de compilación release (`Cargo.toml`)

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "symbols"
panic = "unwind"    # se mantiene para poder capturar pánicos en el scheduler
```

---

## 11. Decisiones de diseño (registro inicial)

Estas decisiones quedan aceptadas. Cambiarlas requiere un nuevo ADR en `DECISIONS.md` que referencie el ID.

| ID | Decisión | Motivo | Alternativa descartada |
|----|----------|--------|------------------------|
| D-01 | Efectos Mica/Acrylic **nativos** vía `set_effects`. Sin `backdrop-filter` cuando el efecto nativo está activo. | Mejor rendimiento, respeta el tema del sistema y evita doble desenfoque. | `backdrop-filter` en CSS sobre ventana transparente (más coste de GPU y comportamiento inconsistente). |
| D-02 | Objetivo de RAM < 40 MB se trata como **objetivo de medición**, no garantía. Se mide proceso a proceso y en total. | WebView2 usa varios procesos hijos; la cifra depende de la versión del runtime. | Comprometer el número sin medición. |
| D-03 | Contenido en `content_json` (ProseMirror) y texto plano derivado en `content_text` solo para FTS. | Evita perder estructura y permite Markdown posterior con conversión. | Guardar solo texto plano (pierde formato) o solo JSON sin índice de búsqueda. |
| D-04 | Scheduler con *sleep-until* del próximo `notify_at`, con tope de 60 s y reprogramación por `Notify`. | Precisión sin polling agresivo; el tope cubre suspensión y saltos de reloj. | Polling fijo cada 20–30 s (más despertares) o solo eventos del sistema (poco fiables). |
| D-05 | Orden: POR HACER por `position`; AGENDA por `due_date` descendente, dentro del día: con hora (ascendente) y luego sin hora por `position`; completadas al final del día. | Coherente con la estructura del documento y con la lectura de agenda. | Orden global por fecha de creación (no refleja la intención del usuario). |
| D-06 | Etiquetas con jerarquía de **un nivel** (`parent_id`). | Cubre "Grado → asignatura" sin complejidad de árbol. | Árbol arbitrario (más validación y UI). |
| D-07 | Recurrencia **no implementada** en v1; columna `rrule` reservada, nullable. | Evita migraciones futuras cuando se implemente. | Omitir la columna (obliga a migrar después). |
| D-08 | Errores IPC como `String` con prefijo de código (sección 7.1). | Cumple la regla de `Result<T, String>` del producto manteniendo códigos estables. | `Result<T, AppError>` serializable (más robusto, pero contradice la especificación inicial; reconsiderar si el prefijo resulta insuficiente). |
| D-09 | Backups con `VACUUM INTO` y validación `integrity_check` antes de restaurar. | Copia consistente aunque haya WAL activo. | Copiar el archivo `.sqlite` en caliente (riesgo de copia inconsistente). |
| D-10 | Notas: TipTap cargado con `import()` diferido. | Acelera el arranque (N-01). | Carga síncrona del editor en el bundle inicial. |

**Decisiones pendientes** (el agente NO debe resolverlas por su cuenta; las propone en `BLOCKERS.md` con dos opciones):
- Si la acción "Marcar como hecha" de la notificación es prioritaria para v1 o se pospone.
- Formato de exportación de la copia de seguridad además de `.sqlite` (p. ej. JSON).

---

## 12. Hoja de ruta

Cada fase termina con una **Definición de hecho (DoD)**. Las tareas marcadas **[H]** requieren validación humana en Windows 11; el agente las implementa, las documenta en `docs/manual-pruebas.md` y **no** las da por verificadas.

### Fase 0 — Esqueleto y medición base
**Objetivo:** aplicación que arranca, ventana frameless con efecto nativo, medición de rendimiento inicial.

Tareas:
1. Crear proyecto con `npm create tauri-app@latest` (plantilla Svelte + TypeScript) y registrar versiones en `DECISIONS.md`.
2. Configurar Tailwind CSS v4 mediante su plugin de Vite; verificar que compila.
3. Configurar ESLint, Prettier, `svelte-check`, `rustfmt`, `clippy`.
4. `tauri.conf.json`: `decorations: false`, `transparent: true`, identificador estable, tamaños por defecto y mínimos.
5. `capabilities/default.json` con permisos mínimos.
6. `TitleBar` con `data-tauri-drag-region` y `WindowControls`.
7. Módulo `window/effects.rs`: aplicar Mica, con fallback a Acrylic y luego `none`; comando `window_set_effect`.
8. Tokens CSS (sección 8.3) y tema oscuro/claro.
9. Script `scripts/size-check.mjs` que informa tamaño del bundle JS.
10. Log de arranque con marcas de tiempo (solo en debug).

**DoD:**
- [A] `cargo clippy -D warnings`, `cargo test`, `npm run check`, `npm run lint`, `npm run build` sin errores.
- [A] `cargo tauri build` genera ejecutable; se registra su tamaño.
- [A] Tests de `effects.rs` de la lógica de fallback (sin ventana real, con backend simulado).
- [H] Ventana se ve con esquinas redondeadas y efecto Mica en Windows 11; se arrastra desde la titlebar; medición de N-01 y N-02 registrada.

### Fase 1 — Persistencia y dominio
**Objetivo:** base de datos sólida, CRUD de tareas y etiquetas sin UI compleja.

Tareas:
1. `db/`: apertura de conexión con pragmas (5.4), migraciones por `user_version`, test de migración desde estado vacío.
2. Migración `001_init.sql` (sección 5.2).
3. `repo/tasks.rs`, `repo/tags.rs`, `repo/fts.rs` con sincronización FTS en transacción.
4. `domain/dates.rs`: validación `YYYY-MM-DD`, `HH:MM`, conversión local/UTC.
5. `domain/ordering.rs`: funciones puras de ordenación (D-05), con tests de tabla.
6. `domain/validation.rs`: límites de longitud, profundidad de etiquetas (D-06).
7. Comandos `tasks_*`, `tag_*`, `task_tags_set`, `trash_list`, `task_restore`.
8. `error.rs` con prefijos de código (D-08).
9. `api.ts` y tipos DTO sincronizados con Rust (test que compara los campos serializados de un ejemplo).

**DoD:**
- [A] Tests unitarios de dominio ≥ 90 % de líneas en `domain/`.
- [A] Tests de repo con base en memoria: CRUD, soft delete, purga de 30 días, FTS tras cada mutación.
- [A] Test de integridad: `PRAGMA integrity_check` = `ok` tras 10 000 operaciones aleatorias.
- [A] Test de consulta de un mes con 10 000 tareas < 30 ms (N-05).

### Fase 2 — Vista principal tipo documento
**Objetivo:** la interfaz central funciona con datos reales.

Tareas:
1. `SectionHeader`, `DayGroup`, `TaskRow`, `Checkbox`, `InlineTitle`.
2. Stores con runes para tareas (`tasks.svelte.ts`), agrupación con `$derived`.
3. Sección "POR HACER" y "AGENDA" con fechas descendentes `DD/MM/YYYY`.
4. Creación rápida (Ctrl+N, Enter en título).
5. Completar con animación y reubicación en "Completadas" (5.x y 8.4).
6. Edición inline de título con guardado al perder foco o Enter.
7. Eliminación con deshacer de 5 s (F-02).
8. Atajos de la sección 9 para estas acciones.
9. Estados vacíos (`EmptyState`).

**DoD:**
- [A] Tests de componentes: agrupación por día, orden, completar mueve a "Completadas", deshacer.
- [A] Tests del store: actualización optimista y reversión si `invoke` falla.
- [A] Sin re-render de filas no afectadas (test con contador de renders en la fila).
- [H] Animación fluida a 60 fps; sensación de documento tipo Notepad++.

### Fase 3 — Editor de notas y búsqueda
**Objetivo:** notas enriquecidas y localización rápida.

Tareas:
1. `NoteEditor` con TipTap (carga diferida, D-10) y `NoteToolbar`.
2. Extensiones: negrita, cursiva, código, enlace, listas, historial.
3. Guardado con *debounce* de 400 ms y `content_text` derivado.
4. Sub-puntos indentados justo bajo la tarea (listas dentro de la nota, mostradas con 24 px de sangría).
5. Spotlight (`Ctrl+K`) con búsqueda FTS, resultados navegables con teclado.
6. Saneado de enlaces: solo `http`, `https`, `mailto`. Otros esquemas se rechazan.

**DoD:**
- [A] Tests de sanitización de enlaces y de `content_text`.
- [A] Test de búsqueda: resultados correctos con acentos y sin ellos.
- [A] Búsqueda < 100 ms con 10 000 tareas (F-09).
- [H] Experiencia de escritura sin latencia perceptible.

### Fase 4 — Recordatorios y notificaciones
**Objetivo:** avisos fiables incluso tras suspensión o reinicio.

Tareas:
1. `scheduler/clock.rs` (trait `Clock` y dos implementaciones).
2. `scheduler/mod.rs`: bucle con `select!` (sección 6.1), reprogramación por `Notify`.
3. Disparo con transacción y reintento con backoff (6.3).
4. Recuperación al arrancar (6.4).
5. `notify/mod.rs` con `tauri-plugin-notification`.
6. `ReminderPopover` con opciones de la sección F-12.
7. Recalcular recordatorios al cambiar fecha/hora de la tarea.
8. Evento `reminder:fired` y toast in-app.

**DoD:**
- [A] Tests del scheduler con `Clock` simulado: disparo exacto, reintento tras fallo de notificación, no duplicados, recuperación < 24 h y > 24 h, cambio de hora de la tarea reprograma.
- [A] Test de corte: matar el proceso durante escritura y verificar integridad.
- [H] Notificación real aparece en Windows 11 en el momento; recordatorio recuperado tras reinicio; comportamiento tras suspensión (N-07).

### Fase 5 — Vistas de calendario
**Objetivo:** Lista, Día, Semana y Mes.

Tareas:
1. `SegmentedControl` (Lista / Día / Semana / Mes) con persistencia en `settings`.
2. `CalendarMonth`, `CalendarWeek`, `CalendarDay` con cuadrícula tenue y bloques etiquetados.
3. Creación rápida arrastrando un tramo horario (F-11).
4. Carga diferida del módulo de calendario (`import()`) para no afectar a N-01.
5. Respeto a la primera día de semana del sistema (`Intl.Locale`) y formato de fecha `DD/MM/YYYY`.

**DoD:**
- [A] Tests de cálculo de semanas y meses (incluido cambio de hora DST con fechas locales).
- [A] Tests de componentes para creación rápida.
- [A] Bundle inicial no crece por el módulo de calendario (`size-check`).
- [H] Interacciones de arrastre fluidas.

### Fase 6 — Pulido, accesibilidad, copias y empaquetado
**Objetivo:** versión 1.0 instalable.

Tareas:
1. Modo claro/oscuro completo y verificado.
2. Revisión de contraste (N-08) con herramienta automatizada sobre tokens.
3. Navegación por teclado completa y etiquetas ARIA en controles.
4. `backup_create` / `backup_restore` con validación (F-16, D-09).
5. Configuración: tema, inicio con Windows (opcional, desactivado por defecto), efecto de ventana.
6. Tests E2E con WebdriverIO + `tauri-driver` **[H]**.
7. Instalador NSIS/MSI, firma pendiente **[H]**, icono y metadatos.
8. Revisión de cumplimiento de N-01 a N-08 con tabla de resultados en `docs/`.

**DoD:**
- [A] Todos los tests y linters en verde; cobertura de dominio ≥ 90 %.
- [A] Tabla de métricas N-03, N-04, N-05, N-06 con valores reales.
- [H] Pruebas E2E en Windows, medición de N-01, N-02, N-07 y N-08 documentadas.

---

## 13. Estrategia de pruebas

| Nivel | Herramienta | Qué cubre | Dónde se ejecuta |
|-------|-------------|-----------|------------------|
| Unitario Rust | `cargo test` | `domain/`, `repo/` con SQLite en memoria, scheduler con `Clock` | Entorno del agente |
| Integración Rust | `cargo test` con archivo temporal | Migraciones, corte, backups, FTS | Entorno del agente |
| Componentes | Vitest + Testing Library | Vistas, stores, atajos | Entorno del agente |
| Contrato IPC | Test de serialización | DTO Rust ↔ TypeScript | Entorno del agente |
| Rendimiento | Scripts de medición | Tamaño, bundle, consultas | Entorno del agente (parcial) |
| Sistema | Pruebas manuales guiadas | Notificaciones, Mica, arranque, RAM | Windows 11 **[H]** |
| E2E | WebdriverIO + tauri-driver | Flujos completos | Windows **[H]** |

**Reglas:**
- Los tests de reloj nunca usan `sleep` real; avanzan `Clock`.
- Cada bug corregido añade un test de regresión antes del arreglo.
- Prohibido `#[ignore]` o `it.skip` sin entrada en `BLOCKERS.md`.

---

## 14. Reglas de trabajo autónomo para Claude

Estas reglas son vinculantes. Son el contenido base de `CLAUDE.md`.

### 14.1 Entorno y límites
- El agente trabaja en un entorno Linux. **No puede** ejecutar la ventana, Mica, WebView2 ni las notificaciones de Windows. Esas verificaciones se marcan **[H]** y se documentan en `docs/manual-pruebas.md`.
- Compilación cruzada de Windows solo si el toolchain está instalado; si no, anotar en `PROGRESS.md` que la comprobación de código específico de Windows queda pendiente.
- Red: solo registros de paquetes (crates.io, npm). Ninguna llamada de red de la aplicación.

### 14.2 Ciclo de trabajo por tarea
1. Leer `PROGRESS.md` y localizar la **primera tarea no marcada** de la fase activa.
2. Leer la sección de la especificación relevante y `DECISIONS.md`.
3. Plan de 3–6 líneas: archivos a tocar, tests a escribir, riesgos.
4. Implementar en un único cambio coherente.
5. Ejecutar la batería de verificación (14.4). Si falla, corregir antes de continuar.
6. Actualizar `PROGRESS.md` (marcar tarea, anotar resultados y métricas).
7. Commit con Conventional Commits (`feat:`, `fix:`, `test:`, `refactor:`, `docs:`, `chore:`), un commit por tarea lógica.
8. Continuar con la siguiente tarea.

### 14.3 Cuándo parar y cuándo seguir
- **Bloqueo puntual** (un test falla por causa desconocida, dependencia no disponible): registrar en `BLOCKERS.md` con causa, intentos y opciones; continuar con la siguiente tarea **no dependiente**.
- **Ambigüedad de producto**: no decidir en silencio. Registrar en `BLOCKERS.md` con dos opciones y una recomendación. Si la tarea depende de ello, pasar a otra.
- **Fin de fase**: si todas las tareas [A] pasan, marcar la fase como `AWAITING_HUMAN` en `PROGRESS.md`, listar los ítems [H] del `manual-pruebas.md` y **detenerse**. No iniciar la fase siguiente hasta que un humano lo confirme con `PROGRESS.md` actualizado.
- **Contradicción** entre especificación y necesidad técnica: no cambiar la especificación; crear ADR propuesto en `DECISIONS.md` con estado `PROPOSED` y esperar confirmación para aplicarlo si afecta a más de una tarea.

### 14.4 Batería de verificación (antes de marcar cualquier tarea)
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run lint
npm run check          # svelte-check
npm run test           # Vitest
npm run build
npm run size           # presupuesto N-04
```
Una tarea no se marca como hecha si cualquiera de estos comandos falla.

### 14.5 Reglas de código (inquebrantables)
- **Cero placeholders:** prohibido `TODO`, `FIXME`, `unimplemented!()`, `todo!()`, funciones vacías o `// implementar luego`. Si algo no puede implementarse, no se crea: se registra en `BLOCKERS.md`.
- **Errores:** ningún `unwrap()` ni `expect()` fuera de tests y de inicialización irrecuperable (con mensaje explicativo). Todo comando IPC devuelve `Result<T, String>` con código (D-08).
- **Svelte 5:** solo runes. Sin `$:`, sin stores legados para estado nuevo (se usan módulos `.svelte.ts` con `$state`).
- **Rendimiento:** no leer ni calcular en el template lo que puede ser `$derived`. No suscribirse a todo el store en cada fila: pasar solo los campos necesarios.
- **SQL:** siempre consultas parametrizadas. Ningún `format!` con datos del usuario.
- **Fechas:** nunca `Utc::now()` fuera de la implementación real de `Clock`. Nunca mezclar fecha local con UTC sin pasar por `domain/dates.rs`.
- **Seguridad:** no se abre HTML sin sanear; enlaces validados (F-08); capacidades de Tauri mínimas (8.1).
- **Dependencias:** ninguna nueva sin entrada en `DECISIONS.md`.
- **Nombres:** código en inglés; textos de UI y mensajes al usuario en español; claves de i18n centralizadas en `lib/i18n.ts`.
- **Comentarios:** solo para explicar *por qué*, nunca para describir lo que el código ya dice.

### 14.6 Lo que el agente nunca hace
- Marcar como verificado algo [H] sin validación humana.
- Modificar esta especificación sin ADR aprobado.
- Eliminar o desactivar tests para que la batería pase.
- Ejecutar migraciones destructivas (`DROP`) sobre datos de usuario; cualquier cambio de esquema es una nueva migración numerada.
- Introducir telemetría, analítica o llamadas de red de la aplicación.
- Hacer commits con `--no-verify` o saltarse hooks.

### 14.7 Formato de los informes al final de cada sesión
El agente actualiza `PROGRESS.md` y deja en la respuesta final, como máximo, este resumen:
- Tareas completadas (IDs).
- Verificaciones [A] ejecutadas y resultado.
- Ítems [H] nuevos que requieren validación humana.
- Bloqueos nuevos (enlace a `BLOCKERS.md`).
- Siguiente tarea propuesta.

---

## 15. Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| RAM real superior a 40 MB por WebView2 | Alta | Medio | D-02: medir desde Fase 0; reducir carga de módulos y extensiones de TipTap. |
| Mica no disponible en algunas configuraciones | Media | Bajo | Fallback Acrylic → sólido (fase 0, DoD). |
| Notificaciones no agrupadas o sin identificador | Media | Medio | `identifier` estable; prueba manual temprana en Fase 4. |
| Pérdida de recordatorios por suspensión | Media | Alto | Tope de 60 s en el sleep, recuperación al arranque (6.4). |
| Corrupción por cierre forzado | Baja | Alto | WAL, transacciones, test de corte (N-06). |
| Cambios de versión en Tauri v2 / Svelte 5 | Media | Medio | Versiones fijadas en `package-lock.json` y `Cargo.lock`; actualizar solo en tarea dedicada. |
| Agente sin acceso a Windows para verificar UI | Alta | Medio | Marcado [H] explícito; no se declara nada verificado sin humano. |

---

## Apéndice A — Plantilla de `CLAUDE.md`

```markdown
# CLAUDE.md — MiAgendaApp

Eres el agente de desarrollo de MiAgendaApp (Tauri v2 + Rust + Svelte 5).

## Lectura obligatoria al inicio
1. PROGRESS.md
2. docs/SPEC.md (sección relevante a la tarea)
3. DECISIONS.md
4. BLOCKERS.md

## Reglas
- Trabaja tarea a tarea según PROGRESS.md. Una tarea = un commit Conventional Commits.
- Batería de verificación completa (SPEC §14.4) antes de marcar una tarea.
- Cero placeholders, cero TODO, cero unwrap() fuera de tests.
- Solo runes de Svelte 5. Consultas SQL parametrizadas. Reloj inyectado (trait Clock).
- Las verificaciones [H] no se dan por hechas: van a docs/manual-pruebas.md.
- Ante ambigüedad o contradicción: BLOCKERS.md o ADR PROPOSED. No decidir en silencio.
- Al terminar una fase con todos los [A] en verde: marcar AWAITING_HUMAN y detenerse.
- No añadir dependencias sin entrada en DECISIONS.md.
- Nada de red en tiempo de ejecución ni telemetría.

## Comandos
- Frontend: npm run dev | build | check | lint | test | size
- Backend: cargo test | cargo clippy --all-targets -- -D warnings | cargo fmt --check
- App: npm run tauri dev | npm run tauri build
```

---

## Apéndice B — Plantilla de `PROGRESS.md`

```markdown
# PROGRESS.md

**Fase activa:** 0
**Estado:** IN_PROGRESS   <!-- IN_PROGRESS | AWAITING_HUMAN | BLOCKED | DONE -->
**Siguiente tarea:** F0-01

## Fase 0 — Esqueleto y medición base
- [ ] F0-01 Crear proyecto Tauri v2 + Svelte 5 + TS
- [ ] F0-02 Tailwind CSS v4 vía plugin Vite
- [ ] F0-03 ESLint, Prettier, svelte-check, rustfmt, clippy
- [ ] F0-04 tauri.conf.json (frameless, transparente, identifier)
- [ ] F0-05 capabilities/default.json mínimas
- [ ] F0-06 TitleBar y WindowControls
- [ ] F0-07 window/effects.rs (Mica → Acrylic → none)
- [ ] F0-08 Tokens CSS y tema
- [ ] F0-09 scripts/size-check.mjs
- [ ] F0-10 Log de arranque en debug

### Verificaciones [A]
- (pendiente)

### Ítems [H] para validación humana
- (pendiente)

### Métricas
| Métrica | Valor | Fecha | Notas |
|---------|-------|-------|-------|
| Tamaño exe (N-03) | | | |
| Bundle JS gzip (N-04) | | | |
| Arranque (N-01) [H] | | | |
| RAM reposo (N-02) [H] | | | |

## Discrepancias detectadas
- (ninguna)

## Notas de sesión
- (vacío)
```

---

## Apéndice C — Plantilla de ADR (`DECISIONS.md`)

```markdown
### D-XX — Título corto
- **Estado:** PROPOSED | ACCEPTED | SUPERSEDED por D-YY
- **Contexto:** qué problema existe.
- **Decisión:** qué se hace.
- **Consecuencias:** qué cambia en el código, tests o especificación.
- **Alternativas descartadas:** y por qué.
- **Fecha:** AAAA-MM-DD
```

---

## Apéndice D — Primer prompt de arranque para el agente

> Lee `CLAUDE.md`, `PROGRESS.md`, `DECISIONS.md` y `docs/SPEC.md` (secciones 0, 3, 8, 10, 12 — Fase 0 y 14). Confirma en un párrafo tu entendimiento de la fase activa y propón el plan de F0-01 a F0-03. Después ejecuta las tareas en orden según §14.2, registrando versiones exactas de Tauri, Svelte, Tailwind y TipTap en `DECISIONS.md`. Al terminar la Fase 0, sigue §14.3 (AWAITING_HUMAN) y deja listados los ítems [H].
