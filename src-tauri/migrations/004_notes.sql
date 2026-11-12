-- Notas y carpetas (D-43), etiquetas compartidas con las tareas (D-44), referencias de una nota
-- a tareas (D-45) y enlaces entre notas (D-46). Solo añade tablas: no toca datos existentes.

CREATE TABLE folders (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 100),
  parent_id   TEXT NULL REFERENCES folders(id) ON DELETE SET NULL,
  position    INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);
-- Dos carpetas hermanas no pueden llamarse igual (sin distinguir mayúsculas).
CREATE UNIQUE INDEX ux_folders_sibling_name ON folders(COALESCE(parent_id, ''), name COLLATE NOCASE);
CREATE INDEX ix_folders_parent ON folders(parent_id);

CREATE TABLE notes (
  id            TEXT PRIMARY KEY,
  folder_id     TEXT NULL REFERENCES folders(id) ON DELETE SET NULL,  -- NULL = Bandeja de entrada
  title         TEXT NOT NULL DEFAULT '' CHECK (length(title) <= 500),
  content_json  TEXT NOT NULL DEFAULT '{"type":"doc","content":[]}',
  content_text  TEXT NOT NULL DEFAULT '',
  is_pinned     INTEGER NOT NULL DEFAULT 0 CHECK (is_pinned IN (0,1)),
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  deleted_at    TEXT NULL
);
CREATE INDEX ix_notes_folder  ON notes(folder_id, updated_at) WHERE deleted_at IS NULL;
CREATE INDEX ix_notes_updated ON notes(updated_at)            WHERE deleted_at IS NULL;
CREATE INDEX ix_notes_deleted ON notes(deleted_at);

CREATE TABLE note_tags (
  note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
  PRIMARY KEY (note_id, tag_id)
);
CREATE INDEX ix_note_tags_tag ON note_tags(tag_id);

-- Tareas que una nota referencia con nodos `taskRef`; se recalcula en cada guardado.
CREATE TABLE note_tasks (
  note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  PRIMARY KEY (note_id, task_id)
);
CREATE INDEX ix_note_tasks_task ON note_tasks(task_id);

-- Enlaces `noteLink` de una nota a otra; se recalcula en cada guardado.
CREATE TABLE note_links (
  source_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  target_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  PRIMARY KEY (source_note_id, target_note_id)
);
CREATE INDEX ix_note_links_target ON note_links(target_note_id);

-- Igual que `tasks_fts` (D-11): independiente, se borra por `note_id`.
CREATE VIRTUAL TABLE notes_fts USING fts5(
  note_id UNINDEXED,
  title,
  content_text,
  tokenize = 'unicode61 remove_diacritics 2'
);
