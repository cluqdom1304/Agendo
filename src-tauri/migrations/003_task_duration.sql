-- Duración de una tarea con hora (D-33): la hora de fin es `due_time` + `duration_minutes`,
-- siempre dentro del mismo día. Sin hora no hay duración.
ALTER TABLE tasks ADD COLUMN duration_minutes INTEGER NULL
  CHECK (duration_minutes IS NULL OR duration_minutes BETWEEN 1 AND 1440);
