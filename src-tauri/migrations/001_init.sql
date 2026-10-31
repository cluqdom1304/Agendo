PRAGMA foreign_keys = ON;

CREATE TABLE tags (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 60),
  color_hex   TEXT NOT NULL CHECK (color_hex GLOB '#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]'),
  parent_id   TEXT NULL REFERENCES tags(id) ON DELETE SET NULL,
  position    INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT NOT NULL
);
CREATE UNIQUE INDEX ux_tags_name ON tags(name COLLATE NOCASE);

CREATE TABLE tasks (
  id            TEXT PRIMARY KEY,
  title         TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 500),
  content_json  TEXT NOT NULL DEFAULT '{"type":"doc","content":[]}',
  content_text  TEXT NOT NULL DEFAULT '',
  due_date      TEXT NULL CHECK (due_date IS NULL OR due_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
  due_time      TEXT NULL CHECK (due_time IS NULL OR due_time GLOB '[0-2][0-9]:[0-5][0-9]'),
  is_completed  INTEGER NOT NULL DEFAULT 0 CHECK (is_completed IN (0,1)),
  completed_at  TEXT NULL,
  position      INTEGER NOT NULL DEFAULT 0,
  rrule         TEXT NULL,            -- reservado para recurrencia (D-07); no se usa en v1
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT NULL
);
CREATE INDEX ix_tasks_due      ON tasks(due_date)     WHERE deleted_at IS NULL;
CREATE INDEX ix_tasks_todo     ON tasks(position)     WHERE deleted_at IS NULL AND due_date IS NULL;
CREATE INDEX ix_tasks_deleted  ON tasks(deleted_at);

CREATE TABLE task_tags (
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
  PRIMARY KEY (task_id, tag_id)
);
CREATE INDEX ix_task_tags_tag ON task_tags(tag_id);

CREATE TABLE reminders (
  id              TEXT PRIMARY KEY,
  task_id         TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  notify_at       TEXT NOT NULL,       -- UTC
  offset_minutes  INTEGER NULL,        -- NULL = hora personalizada
  is_sent         INTEGER NOT NULL DEFAULT 0 CHECK (is_sent IN (0,1)),
  sent_at         TEXT NULL,
  created_at      TEXT NOT NULL
);
CREATE INDEX ix_reminders_pending ON reminders(is_sent, notify_at);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- FTS5 independiente (no external-content) para evitar problemas de rowid con ids TEXT.
CREATE VIRTUAL TABLE tasks_fts USING fts5(
  task_id UNINDEXED,
  title,
  content_text,
  tokenize = 'unicode61 remove_diacritics 2'
);
