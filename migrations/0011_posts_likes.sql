CREATE TABLE posts_likes
(
    id       SERIAL PRIMARY KEY,
    post_id  integer NOT NULL,
    user_id  uuid    NOT NULL,
    liked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX posts_likes_post_id_idx ON posts_likes (post_id);
CREATE INDEX posts_likes_user_id_idx ON posts_likes (user_id);

ALTER TABLE posts
    ADD COLUMN likes INT DEFAULT 0;