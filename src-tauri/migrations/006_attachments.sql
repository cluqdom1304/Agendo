-- Adjuntos de las notas (D-47). El fichero vive en `attachments/<id>`; el id es el SHA-256 de su
-- contenido, así que dos copias del mismo fichero son una sola. `note_attachments` guarda qué
-- notas usan cada adjunto: lo que nadie usa se borra al arrancar, pasado un margen.
CREATE TABLE attachments (
    id          TEXT PRIMARY KEY,
    mime        TEXT NOT NULL,
    size_bytes  INTEGER NOT NULL CHECK (size_bytes > 0),
    created_at  TEXT NOT NULL
);

CREATE TABLE note_attachments (
    note_id        TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    attachment_id  TEXT NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
    PRIMARY KEY (note_id, attachment_id)
) WITHOUT ROWID;
CREATE INDEX ix_note_attachments_attachment ON note_attachments(attachment_id);
