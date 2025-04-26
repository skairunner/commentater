-- Add indices to article_queue to help in selecting tasks
CREATE INDEX article_queue_user_id ON article_queue(user_id);
CREATE INDEX article_queue_done ON article_queue(done);
CREATE UNIQUE INDEX user_queue_user_id ON user_queue(user_id);
CREATE INDEX user_queue_last_updated ON user_queue(last_updated)
