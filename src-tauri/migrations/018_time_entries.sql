-- Registro de tiempo de enfoque (D-97): cada sesión sobre una tarea. Solo añade una tabla: no toca
-- datos existentes. Inicio y fin los marca el backend con su reloj; una sesión sin `ended_at`
-- está en marcha, y como mucho hay una a la vez.
CREATE TABLE time_entries (
  id              TEXT PRIMARY KEY,
  task_id         TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  started_at      TEXT NOT NULL,           -- UTC
  ended_at        TEXT NULL,               -- UTC
  planned_minutes INTEGER NOT NULL CHECK (planned_minutes BETWEEN 1 AND 600),
  CHECK (ended_at IS NULL OR ended_at >= started_at)
);
CREATE INDEX ix_time_entries_task ON time_entries(task_id, started_at);
CREATE INDEX ix_time_entries_started ON time_entries(started_at);
CREATE UNIQUE INDEX ux_time_entries_one_running ON time_entries((1)) WHERE ended_at IS NULL;
