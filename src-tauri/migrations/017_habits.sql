-- Hábitos (D-95): algo que se repite en ciertos días y se marca como hecho cada día. Solo añade
-- tablas: no toca datos existentes. La racha no se guarda: se deriva de `habit_logs`, así que
-- editar un día pasado nunca la desincroniza.
CREATE TABLE habits (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
  -- Días programados como máscara de bits, bit 0 = lunes ... bit 6 = domingo (127 = a diario).
  days        INTEGER NOT NULL DEFAULT 127 CHECK (days BETWEEN 1 AND 127),
  time        TEXT NULL,
  tag_id      TEXT NULL REFERENCES tags(id) ON DELETE SET NULL,
  position    INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  archived_at TEXT NULL,
  deleted_at  TEXT NULL
);
CREATE INDEX ix_habits_live ON habits(position, id) WHERE deleted_at IS NULL;
CREATE INDEX ix_habits_trashed ON habits(deleted_at) WHERE deleted_at IS NOT NULL;

-- Marcar un día es insertar su fila y desmarcarlo es borrarla (solo la del propio hábito).
CREATE TABLE habit_logs (
  habit_id TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
  date     TEXT NOT NULL,                  -- fecha local AAAA-MM-DD
  PRIMARY KEY (habit_id, date)
) WITHOUT ROWID;
