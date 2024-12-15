DROP TRIGGER user_aggregates_users ON users;
DROP TRIGGER user_aggregates_posts ON posts;
DROP TRIGGER user_aggregates_following ON followers;
DROP TRIGGER user_aggregates_followers ON followers;

DROP TRIGGER site_aggregates_posts ON posts;
DROP TRIGGER site_aggregates_users ON users;

DROP TRIGGER instance_aggregates_recalculate ON instance_settings;

DROP TABLE user_aggregates;
DROP TABLE instance_aggregates;

DROP FUNCTION instance_aggregates_recalculate;

DROP FUNCTION user_aggregates_users;
DROP FUNCTION user_aggregates_posts;
DROP FUNCTION user_aggregates_following;
DROP FUNCTION user_aggregates_followers;

DROP FUNCTION site_aggregates_users;
DROP FUNCTION site_aggregates_posts;
