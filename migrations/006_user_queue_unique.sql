-- User queue table needs to have one entry per user
ALTER TABLE user_queue ADD CONSTRAINT user_queue_user_id_unique UNIQUE (user_id);
