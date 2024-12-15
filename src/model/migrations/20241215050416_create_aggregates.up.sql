CREATE TABLE instance_aggregates (
    id SERIAL PRIMARY KEY,
    updated TIMESTAMP NOT NULL DEFAULT now(),

    -- Total amount of registered users in this instance excluding
    -- those whose their emails are not verified (if required) or
    -- their emails are not set (if required).
    users BIGINT NOT NULL,
    -- Total amount of submitted posts in this instance
    posts BIGINT NOT NULL
);

CREATE FUNCTION instance_aggregates_recalculate()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
DECLARE
    "total_users" BIGINT = 0;
    "total_posts" BIGINT = 0;
BEGIN
    SELECT coalesce(count(*), 0) INTO "total_users" FROM users u
        JOIN instance_settings "is" ON "is".id = 0
        WHERE (
            CASE WHEN "is".require_email_verification
            THEN u.email_verified
            ELSE true END
        ) AND (
            CASE WHEN "is".require_email_registration
            THEN u.email IS NOT NULL
            ELSE true
        END);

    SELECT coalesce(count(*), 0) INTO "total_posts"
    FROM posts;

    UPDATE instance_aggregates
    SET posts = "total_posts",
        users = "total_users"
    WHERE id = 0;

    RETURN NULL;
END $$;

CREATE TRIGGER instance_aggregates_recalculate
AFTER UPDATE ON instance_settings FOR EACH ROW
EXECUTE PROCEDURE instance_aggregates_recalculate();

INSERT INTO instance_aggregates (users, posts)
SELECT (
    SELECT coalesce(count(*), 0) FROM users u
    JOIN instance_settings "is" ON "is".id = 0
    WHERE (
        CASE WHEN "is".require_email_verification
        THEN u.email_verified
        ELSE true END
    ) AND (
        CASE WHEN "is".require_email_registration
        THEN u.email IS NOT NULL
        ELSE true
    END)
) AS users,
( SELECT coalesce(count(*), 0) FROM posts ) AS posts;

CREATE TABLE user_aggregates (
    id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    updated TIMESTAMP NOT NULL DEFAULT now(),
    
    -- Total user followed someone
    following BIGINT NOT NULL DEFAULT 0,

    -- Total user followers
    followers BIGINT NOT NULL DEFAULT 0,

    -- Total user posts
    posts BIGINT NOT NULL DEFAULT 0
);

INSERT INTO user_aggregates (id, following, followers, posts)
SELECT
    u.id,
    coalesce(fg.following, 0) AS following,
    coalesce(fr.followers, 0) AS followers,
    coalesce(pd.posts, 0) AS posts
FROM users u
    LEFT JOIN (
        SELECT count(p.id) AS posts, p.author_id
        FROM posts p
        GROUP BY p.author_id
    ) pd ON pd.author_id = u.id
    LEFT JOIN (
        SELECT count(f.id) AS followers, f.target_id
        FROM followers f
        GROUP BY f.target_id
    ) fr ON fr.target_id = u.id
    LEFT JOIN (
        SELECT count(f.id) AS following, f.source_id
        FROM followers f
        GROUP BY f.source_id
    ) fg on fg.source_id = u.id;

CREATE FUNCTION site_aggregates_users()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE instance_aggregates
        SET users = users + 1,
            updated = now();
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE instance_aggregates
        SET users = users - 1,
            updated = now();
    END IF;
    RETURN NULL;
END $$;

CREATE FUNCTION site_aggregates_posts()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE instance_aggregates
        SET posts = posts + 1,
            updated = now();
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE instance_aggregates
        SET posts = posts - 1,
            updated = now();
    END IF;
    RETURN NULL;
END $$;

CREATE TRIGGER site_aggregates_users
AFTER INSERT OR DELETE ON users FOR EACH ROW
EXECUTE PROCEDURE site_aggregates_users();

CREATE TRIGGER site_aggregates_posts
AFTER INSERT OR DELETE ON posts FOR EACH ROW
EXECUTE PROCEDURE site_aggregates_posts();

CREATE FUNCTION user_aggregates_users()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO user_aggregates (id) VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM user_aggregates WHERE id = OLD.id;
    END IF;
    RETURN NULL;
END $$;

CREATE FUNCTION user_aggregates_posts()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE user_aggregates
        SET posts = posts + 1,
            updated = now()
        WHERE id = NEW.id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE user_aggregates
        SET posts = posts - 1,
            updated = now()
        WHERE id = OLD.id;
    END IF;
    RETURN NULL;
END $$;

CREATE FUNCTION user_aggregates_following()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE user_aggregates
        SET following = following + 1,
            updated = now()
        WHERE id = NEW.source_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE user_aggregates
        SET following = following - 1,
            updated = now()
        WHERE id = OLD.source_id;
    END IF;
    RETURN NULL;
END $$;

CREATE FUNCTION user_aggregates_followers()
RETURNS TRIGGER LANGUAGE PLPGSQL
AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE user_aggregates
        SET followers = followers + 1,
            updated = now()
        WHERE id = NEW.target_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE user_aggregates
        SET followers = followers - 1,
            updated = now()
        WHERE id = OLD.target_id;
    END IF;
    RETURN NULL;
END $$;

CREATE TRIGGER user_aggregates_users
AFTER INSERT OR DELETE ON users FOR EACH ROW
EXECUTE PROCEDURE user_aggregates_users();

CREATE TRIGGER user_aggregates_posts
AFTER INSERT OR DELETE ON posts FOR EACH ROW
EXECUTE PROCEDURE user_aggregates_posts();

CREATE TRIGGER user_aggregates_following
AFTER INSERT OR DELETE ON followers FOR EACH ROW
EXECUTE PROCEDURE user_aggregates_following();

CREATE TRIGGER user_aggregates_followers
AFTER INSERT OR DELETE ON followers FOR EACH ROW
EXECUTE PROCEDURE user_aggregates_followers();
