-- Plantillas de nota (F13-02): un documento con nombre, sin carpeta ni etiquetas.
CREATE TABLE note_templates (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
  content_json TEXT NOT NULL,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
CREATE UNIQUE INDEX ux_note_templates_name ON note_templates(name COLLATE NOCASE);
