-- TODO: signature verification and end-to-end encryption
CREATE TABLE users (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    name VARCHAR(20) NOT NULL UNIQUE CHECK(length(name) > 2),

    admin BOOLEAN NOT NULL DEFAULT false,
    display_name VARCHAR(30),

    email VARCHAR(254) CHECK(length(email) > 0),
    email_verified BOOLEAN NOT NULL DEFAULT false,

    -- This is a secure alternative than storing password hashes in the
    -- database since we only have to calculate our access key by deriving
    -- it through our own passphrase (regardless of its length).
    access_key_hash TEXT NOT NULL UNIQUE,

    -- User's permanent AEAD symmetric key to decrypt their subkeys
    --
    -- They may have to enroll another symmetric key but all of their
    -- subkeys may be lost.
    encrypted_symmetric_key TEXT NOT NULL UNIQUE,

    -- This is differentiate from one user to another without resulting
    -- into the same ciphertext when encrypting/decrypting data.
    salt TEXT NOT NULL UNIQUE,
    updated TIMESTAMP
);

CREATE TYPE registration_mode AS ENUM (
    'open', 'require-approval', 'closed'
);

-- Instance settings, it should contain only one instance settings.
CREATE TABLE instance_settings (
    id SERIAL PRIMARY KEY,
    created TIMESTAMP NOT NULL DEFAULT now(),

    -- Post related options --
    post_max_characters INT NOT NULL DEFAULT 200 CHECK (post_max_characters > 0),

    -- Registration options --
    registration_mode REGISTRATION_MODE NOT NULL DEFAULT 'open',
    require_email_registration BOOLEAN NOT NULL DEFAULT false,
    require_email_verification BOOLEAN NOT NULL DEFAULT false,

    -- Moderation related options --
    require_captcha BOOLEAN NOT NULL DEFAULT false,

    updated TIMESTAMP
);
INSERT INTO instance_settings (id) VALUES (0);

CREATE TABLE followers (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    source_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    UNIQUE (source_id, target_id),
    CHECK (source_id != target_id)
);

CREATE TABLE posts (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    author_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    content TEXT,
    updated TIMESTAMP
);
