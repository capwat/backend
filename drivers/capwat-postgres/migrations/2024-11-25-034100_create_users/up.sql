CREATE TABLE users (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    updated TIMESTAMP,
    name VARCHAR(20) NOT NULL UNIQUE CHECK(length(name) > 2),
    display_name VARCHAR(30),
    email VARCHAR(254) CHECK(length(email) > 0),
    email_verified BOOLEAN NOT NULL DEFAULT false,
    password_hash TEXT NOT NULL CHECK(length(password_hash) > 0)
);
