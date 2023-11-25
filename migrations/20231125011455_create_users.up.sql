CREATE TABLE "users" (
    id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created_at timestamp NOT NULL DEFAULT(now() AT TIME ZONE 'utc'),
    name varchar(20) UNIQUE NOT NULL,
    email varchar(255) UNIQUE,
    display_name varchar(25),
    password_hash text NOT NULL,
    updated_at timestamp
);
