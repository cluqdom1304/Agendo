-- Historial de versiones de nota (F13-03, D-48): instantáneas del documento anterior a un
-- guardado, espaciadas en el tiempo y con un máximo por nota. Se borran con la nota.
CREATE TABLE note_versions (
  id           TEXT PRIMARY KEY,
  note_id      TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  title        TEXT NOT NULL,
  content_json TEXT NOT NULL,
  created_at   TEXT NOT NULL
);
CREATE INDEX ix_note_versions_note ON note_versions(note_id, created_at);
