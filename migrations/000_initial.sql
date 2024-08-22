-- Table of worldanvil identities.
CREATE TABLE wa_user (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY(SEQUENCE NAME wa_user_pkey START WITH 38281),
    worldanvil_id TEXT,
    name TEXT NOT NULL,
    avatar_url TEXT
);

CREATE INDEX wa_user_name ON wa_user (name);

-- Table of commentater users.
-- This is also used to delete all info tied to a specific user.
CREATE TABLE commentater_user (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    worldanvil_id BIGINT REFERENCES wa_user(id) ON DELETE CASCADE,
    last_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE world (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY(SEQUENCE NAME world_pkey START WITH 123094),
    user_id BIGINT REFERENCES commentater_user(id) ON DELETE CASCADE,
    worldanvil_id TEXT NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE article (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY(SEQUENCE NAME article_pkey START WITH 88843),
    user_id BIGINT REFERENCES commentater_user(id) ON DELETE CASCADE,
    world_id BIGINT REFERENCES world(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    last_checked TIMESTAMP WITH TIME ZONE
);

CREATE TABLE article_content (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    article_id BIGINT REFERENCES article(id) ON DELETE CASCADE,
    worldanvil_id TEXT NOT NULL,
    title TEXT NOT NULL
);

CREATE TABLE comment (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id BIGINT REFERENCES commentater_user(id) ON DELETE CASCADE,
    author_id BIGINT REFERENCES wa_user(id) ON DELETE SET NULL,
    article_id BIGINT REFERENCES article(id) ON DELETE CASCADE,
    content TEXT NOT NULL DEFAULT '',
    date TIMESTAMP WITH TIME ZONE NOT NULL,
    starred BOOLEAN NOT NULL DEFAULT FALSE,
    deleted BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE comment_replies (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id BIGINT REFERENCES commentater_user(id) ON DELETE CASCADE,
    article_id BIGINT REFERENCES article(id) ON DELETE CASCADE,
    content TEXT NOT NULL DEFAULT '',
    date TIMESTAMP WITH TIME ZONE NOT NULL,
    starred BOOLEAN NOT NULL DEFAULT FALSE,
    parent BIGINT REFERENCES comment(id) ON DELETE SET NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE article_queue (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    article_id BIGINT NOT NULL REFERENCES article(id) ON DELETE CASCADE
);
