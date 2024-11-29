CREATE TYPE registration_mode AS ENUM (
    'open', 'require-approval', 'closed'
);

CREATE TABLE instance_settings (
    id SERIAL PRIMARY KEY,
    created TIMESTAMP NOT NULL DEFAULT now(),
    registration_mode registration_mode NOT NULL DEFAULT 'open',

    require_email_registration BOOLEAN NOT NULL DEFAULT false,
    require_email_verification BOOLEAN NOT NULL DEFAULT false,

    require_captcha BOOLEAN NOT NULL DEFAULT false,

    updated TIMESTAMP
);
