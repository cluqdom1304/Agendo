-- D-93: prioridad de la tarea y progreso de su lista de casillas. El progreso se guarda para que
-- las listas no tengan que leer `content_json` de cada fila; se recalcula al guardar el contenido.
-- Solo columnas nuevas con valor por defecto: las filas existentes quedan sin prioridad y con 0/0.
ALTER TABLE tasks ADD COLUMN priority INTEGER NOT NULL DEFAULT 0 CHECK (priority BETWEEN 0 AND 3);
ALTER TABLE tasks ADD COLUMN checklist_done INTEGER NOT NULL DEFAULT 0 CHECK (checklist_done >= 0);
ALTER TABLE tasks ADD COLUMN checklist_total INTEGER NOT NULL DEFAULT 0 CHECK (checklist_total >= 0);
