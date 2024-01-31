CREATE TABLE users (
    id BIGINT PRIMARY KEY CHECK(id > 0),
    created_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    name VARCHAR(25) UNIQUE NOT NULL,
    email VARCHAR,
    display_name VARCHAR(30),
    password_hash TEXT NOT NULL,
    updated_at TIMESTAMP
);
SELECT diesel_manage_updated_at('users');
