CREATE TABLE users (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    updated TIMESTAMP,
    name VARCHAR(20) NOT NULL UNIQUE CHECK(length(name) > 2),
    display_name VARCHAR(30),
    email VARCHAR(254) CHECK(length(email) > 0),
    email_verified BOOLEAN NOT NULL DEFAULT false,

    access_key_hash TEXT NOT NULL UNIQUE,

    root_classic_pk TEXT NOT NULL,
    root_encrypted_classic_sk TEXT NOT NULL,

    root_pqc_pk TEXT NOT NULL,
    root_encrypted_pqc_sk TEXT NOT NULL
);
