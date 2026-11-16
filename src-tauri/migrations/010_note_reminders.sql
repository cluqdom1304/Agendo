-- Recordatorios de nota (F13-04, D-52): una hora a la que avisar de una nota. El motor de avisos
-- es el mismo que el de las tareas; solo cambia dónde se guarda el aviso. Una nota tiene como
-- mucho un aviso pendiente.
CREATE TABLE note_reminders (
  id         TEXT PRIMARY KEY,
  note_id    TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  notify_at  TEXT NOT NULL,       -- UTC
  is_sent    INTEGER NOT NULL DEFAULT 0 CHECK (is_sent IN (0,1)),
  sent_at    TEXT NULL,
  created_at TEXT NOT NULL
);
CREATE INDEX ix_note_reminders_pending ON note_reminders(is_sent, notify_at);
CREATE UNIQUE INDEX ux_note_reminders_one_pending ON note_reminders(note_id) WHERE is_sent = 0;
