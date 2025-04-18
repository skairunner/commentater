-- We want to make the apikey unique and not null.
ALTER TABLE commentater_user ADD CONSTRAINT api_key_unique UNIQUE (api_key);
CREATE INDEX commentater_user_user_id_api_key ON commentater_user(id, api_key);
ALTER TABLE commentater_user ALTER COLUMN api_key SET NOT NULL;
-- We also want to store the nice username for a user
ALTER TABLE commentater_user ADD COLUMN display_name TEXT;
