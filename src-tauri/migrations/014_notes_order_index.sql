-- A3-01 (D-64): el orden por defecto de la lista de notas (fijadas primero, la última edición
-- arriba) sale de un índice, así que una ventana (`LIMIT`/`OFFSET`) lee solo las filas que
-- devuelve en lugar de ordenar todas las notas.
CREATE INDEX ix_notes_order ON notes(is_pinned DESC, updated_at DESC, id) WHERE deleted_at IS NULL;
