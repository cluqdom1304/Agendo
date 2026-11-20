-- Pausas del enfoque (D-99): solo añade dos columnas a `time_entries`, sin tocar los datos
-- existentes. `paused_seconds` acumula las pausas ya cerradas y `paused_at` marca la pausa en curso;
-- el tiempo dedicado de una sesión es fin − inicio − pausas.
ALTER TABLE time_entries ADD COLUMN paused_seconds INTEGER NOT NULL DEFAULT 0 CHECK (paused_seconds >= 0);
ALTER TABLE time_entries ADD COLUMN paused_at TEXT NULL;
