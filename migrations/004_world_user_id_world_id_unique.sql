-- Make combo of user_id, worldanvil_id unique
ALTER TABLE world ADD CONSTRAINT world_user_id_worldanvil_id_unique UNIQUE(user_id, worldanvil_id);
-- Make WA ID required for user
ALTER TABLE commentater_user ALTER COLUMN worldanvil_id SET NOT NULL;
