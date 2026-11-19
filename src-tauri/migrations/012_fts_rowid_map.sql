-- A2-01 (D-62): el índice FTS se actualiza por rowid en lugar de borrar por `task_id` /
-- `note_id` (columna UNINDEXED: cada borrado recorría todo el índice).
-- El mapa id -> rowid del índice es una tabla normal con clave primaria.
CREATE TABLE tasks_fts_map (
  task_id   TEXT    PRIMARY KEY,
  fts_rowid INTEGER NOT NULL
) WITHOUT ROWID;
CREATE TABLE notes_fts_map (
  note_id   TEXT    PRIMARY KEY,
  fts_rowid INTEGER NOT NULL
) WITHOUT ROWID;

-- Reindexado desde las tablas base (el índice es un dato derivado): una sola fila por
-- tarea/nota viva, sin duplicados ni filas huérfanas heredadas.
DELETE FROM tasks_fts;
INSERT INTO tasks_fts (task_id, title, content_text)
  SELECT id, title, content_text FROM tasks WHERE deleted_at IS NULL;
INSERT INTO tasks_fts_map (task_id, fts_rowid) SELECT task_id, rowid FROM tasks_fts;

DELETE FROM notes_fts;
INSERT INTO notes_fts (note_id, title, content_text)
  SELECT id, title, content_text FROM notes WHERE deleted_at IS NULL;
INSERT INTO notes_fts_map (note_id, fts_rowid) SELECT note_id, rowid FROM notes_fts;
