-- Fixes a mistake on the world table
ALTER TABLE world ALTER COLUMN user_id SET NOT NULL;
