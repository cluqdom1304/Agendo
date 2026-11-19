-- A2-04 (D-63): índices que faltaban o que engañaban al planificador (EXPLAIN QUERY PLAN).
-- Los recordatorios de cada tarea se buscaban recorriendo toda la tabla `reminders` (una vez por
-- tarea en cada listado, y en cada borrado en cascada de una tarea).
CREATE INDEX ix_reminders_task ON reminders(task_id);

-- Los índices sobre `deleted_at` incluían todas las filas vivas (NULL) y las estadísticas de
-- SQLite cuentan cada NULL como valor distinto: el planificador los creía muy selectivos y los
-- prefería a índices parciales mejores. Se sustituyen por índices solo de lo que está en la
-- papelera (los únicos que las consultas de papelera y de purga necesitan). Solo son índices:
-- no se toca ningún dato.
DROP INDEX ix_tasks_deleted;
CREATE INDEX ix_tasks_trashed ON tasks(deleted_at) WHERE deleted_at IS NOT NULL;
DROP INDEX ix_notes_deleted;
CREATE INDEX ix_notes_trashed ON notes(deleted_at) WHERE deleted_at IS NOT NULL;
