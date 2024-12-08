-- TODO: signature verification and end-to-end encryption

-- Our maximum is monthly because why yearly!?
CREATE TYPE key_rotation_frequency AS ENUM (
    'weekly', 'monthly'
);

CREATE TABLE users (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    name VARCHAR(20) NOT NULL UNIQUE CHECK(length(name) > 2),

    admin BOOLEAN NOT NULL DEFAULT false,
    display_name VARCHAR(30),
    key_rotation_frequency KEY_ROTATION_FREQUENCY NOT NULL,

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

    -- Registration options --
    default_key_rotation_frequency KEY_ROTATION_FREQUENCY NOT NULL DEFAULT 'monthly',
    registration_mode registration_mode NOT NULL DEFAULT 'open',
    require_email_registration BOOLEAN NOT NULL DEFAULT false,
    require_email_verification BOOLEAN NOT NULL DEFAULT false,

    -- Moderation related options --
    require_captcha BOOLEAN NOT NULL DEFAULT false,

    updated TIMESTAMP
);


-- A collection of user keys that were created since registration
-- or throughout the running of this instance. Users' keys will be
-- renewed every month (key rotation).
CREATE TABLE user_keys (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    created TIMESTAMP NOT NULL DEFAULT now(),

    -- when these keys will expire before rerolling again
    -- into a new key and move on with our day.
    expires_at TIMESTAMP NOT NULL,

    public_key TEXT NOT NULL, -- classic public key
    encrypted_secret_key TEXT NOT NULL -- classic secret key
);

-- CREATE TABLE post_clusters (
--     id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
--     created TIMESTAMP NOT NULL DEFAULT now(),
--     parent_id BIGINT REFERENCES post_clusters(id),

--     -- TODO: root clusters must have a user id or something related
--     --       otherwise, it is an invalid cluster.
--     user_id BIGINT REFERENCES users(id),

--     CHECK (id != parent_id)
-- );

-- -- each cluster will rotate their keys :)
-- CREATE TABLE post_cluster_keys (
--     id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
--     created TIMESTAMP NOT NULL DEFAULT now(),
--     cluster_id BIGINT NOT NULL REFERENCES post_clusters(id) ON DELETE CASCADE,

--     -- when these keys will expire before rerolling again
--     -- into a new key and move on with our day.
--     expires_at TIMESTAMP NOT NULL,

--     -- each cluster has its own public and private keys
--     public_key TEXT NOT NULL, -- classic public key
--     encrypted_secret_key TEXT NOT NULL -- classic secret key
-- );

-- CREATE TABLE post_cluster_members (
--     id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
--     created TIMESTAMP NOT NULL DEFAULT now(),

--     cluster_id BIGINT NOT NULL REFERENCES post_clusters(id) ON DELETE CASCADE,
--     member_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE
-- );

-- CREATE TABLE post_cluster_member_keys (
--     created TIMESTAMP NOT NULL DEFAULT now(),
--     member_id BIGINT NOT NULL REFERENCES post_cluster_members(id),
--     cluster_keys_id BIGINT NOT NULL REFERENCES post_cluster_keys(id),

--     -- their own agreed symmetric key made from the member's
--     -- public key and stuff :)
--     symmetric_key TEXT NOT NULL,

--     PRIMARY KEY (member_id, cluster_keys_id)
-- );

-- CREATE TABLE posts (
--     -- Since diesel does not support tables without a primary key, we'll
--     -- have to create our own some kind of an index i guess.
--     "_pg_id" BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

--     -- Since we'll have to support end-to-end encryption and some clusters
--     -- may generate different ciphertext, we'll make this id not unique.
--     --
--     -- This ID is composed of Snowflake ID.
--     id BIGINT NOT NULL,
--     created_at TIMESTAMP NOT NULL DEFAULT now(),

--     cluster_id BIGINT NOT NULL REFERENCES post_clusters(id),
--     cluster_keys_id BIGINT NOT NULL REFERENCES post_cluster_keys(id),

--     content TEXT NOT NULL
-- );
