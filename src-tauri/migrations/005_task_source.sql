-- Nota de la que nació una tarea (D-45). Si la nota se elimina del todo, la tarea se conserva.
ALTER TABLE tasks ADD COLUMN source_note_id TEXT NULL REFERENCES notes(id) ON DELETE SET NULL;
CREATE INDEX ix_tasks_source_note ON tasks(source_note_id) WHERE source_note_id IS NOT NULL;
