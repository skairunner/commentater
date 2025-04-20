-- Store worldanvil id for articles
ALTER TABLE article ADD COLUMN worldanvil_id TEXT;
CREATE INDEX article_worldanvil_id ON article(worldanvil_id);
