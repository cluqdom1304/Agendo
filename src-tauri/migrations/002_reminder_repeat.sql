-- Recordatorios periódicos (D-32). Solo los de hora personalizada se repiten.
-- Al sonar una repetición se queda como historial y se crea la siguiente con `occurrence` + 1.
ALTER TABLE reminders ADD COLUMN repeat_every INTEGER NULL
  CHECK (repeat_every IS NULL OR repeat_every BETWEEN 1 AND 365);
ALTER TABLE reminders ADD COLUMN repeat_unit TEXT NULL
  CHECK (repeat_unit IS NULL OR repeat_unit IN ('hour','day','week','month'));
ALTER TABLE reminders ADD COLUMN repeat_count INTEGER NULL
  CHECK (repeat_count IS NULL OR repeat_count BETWEEN 2 AND 1000);
ALTER TABLE reminders ADD COLUMN repeat_until TEXT NULL
  CHECK (repeat_until IS NULL OR repeat_until GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]');
ALTER TABLE reminders ADD COLUMN occurrence INTEGER NOT NULL DEFAULT 1 CHECK (occurrence >= 1);
