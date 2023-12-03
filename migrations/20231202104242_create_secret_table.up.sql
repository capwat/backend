-- postgresql-contrib package is required to import
-- this required extension below.
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE "secrets" (
    id serial PRIMARY KEY,
    jwt varchar NOT NULL DEFAULT encode(gen_random_bytes(36),'base64')
);

INSERT INTO "secrets" DEFAULT VALUES;
