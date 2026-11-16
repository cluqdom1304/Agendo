-- Nota diaria (F13-01): una nota por día, identificada por su fecha local.
ALTER TABLE notes ADD COLUMN daily_date TEXT NULL;
CREATE UNIQUE INDEX ux_notes_daily_date ON notes(daily_date) WHERE daily_date IS NOT NULL;
