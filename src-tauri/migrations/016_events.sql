-- Eventos (D-94): citas con inicio y fin, distintas de las tareas (no se completan). Solo añade
-- tablas: no toca datos existentes. Las repeticiones no se guardan por ocurrencia: se expanden
-- por consulta a partir de `rrule`.
CREATE TABLE events (
  id               TEXT PRIMARY KEY,
  title            TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 500),
  notes            TEXT NOT NULL DEFAULT '' CHECK (length(notes) <= 20000),
  start_date       TEXT NOT NULL,           -- fecha local AAAA-MM-DD
  start_time       TEXT NULL,               -- HH:MM local; NULL = todo el día
  end_date         TEXT NOT NULL,
  end_time         TEXT NULL,
  rrule            TEXT NULL,
  reminder_offsets TEXT NOT NULL DEFAULT '[]', -- JSON: minutos de antelación configurados
  created_at       TEXT NOT NULL,
  updated_at       TEXT NOT NULL,
  deleted_at       TEXT NULL,
  CHECK (end_date >= start_date),
  CHECK ((start_time IS NULL) = (end_time IS NULL)),
  CHECK (start_time IS NULL OR end_date > start_date OR end_time >= start_time)
);
CREATE INDEX ix_events_start ON events(start_date) WHERE deleted_at IS NULL;
CREATE INDEX ix_events_recurring ON events(start_date) WHERE deleted_at IS NULL AND rrule IS NOT NULL;
CREATE INDEX ix_events_trashed ON events(deleted_at) WHERE deleted_at IS NOT NULL;

CREATE TABLE event_tags (
  event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  tag_id   TEXT NOT NULL REFERENCES tags(id)   ON DELETE CASCADE,
  PRIMARY KEY (event_id, tag_id)
);
CREATE INDEX ix_event_tags_tag ON event_tags(tag_id);

-- Avisos de un evento (mismo patrón que `note_reminders`, D-52): una fila por aviso que está
-- esperando o que ya sonó. En una serie solo espera el de la próxima ocurrencia.
CREATE TABLE event_reminders (
  id             TEXT PRIMARY KEY,
  event_id       TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  offset_minutes INTEGER NOT NULL,
  notify_at      TEXT NOT NULL,             -- UTC
  is_sent        INTEGER NOT NULL DEFAULT 0 CHECK (is_sent IN (0,1)),
  sent_at        TEXT NULL,
  created_at     TEXT NOT NULL
);
CREATE INDEX ix_event_reminders_pending ON event_reminders(is_sent, notify_at);
CREATE INDEX ix_event_reminders_event ON event_reminders(event_id);
