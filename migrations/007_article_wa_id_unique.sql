-- Make article WA id unique
ALTER TABLE article ADD CONSTRAINT article_wa_id_unique UNIQUE (worldanvil_id);
CREATE INDEX article_wa_id ON article(worldanvil_id);
