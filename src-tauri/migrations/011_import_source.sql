-- Importación sin duplicados (F17-06): de dónde viene una nota importada y la huella de su
-- contenido. Una nota con el mismo origen y la misma huella no se vuelve a importar.
ALTER TABLE notes ADD COLUMN import_origin TEXT NULL;
ALTER TABLE notes ADD COLUMN import_hash   TEXT NULL;
CREATE INDEX ix_notes_import ON notes(import_origin, import_hash) WHERE import_origin IS NOT NULL;
