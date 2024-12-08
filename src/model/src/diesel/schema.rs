// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "key_rotation_frequency"))]
    pub struct KeyRotationFrequency;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "registration_mode"))]
    pub struct RegistrationMode;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::KeyRotationFrequency;
    use super::sql_types::RegistrationMode;

    instance_settings (id) {
        id -> Int4,
        created -> Timestamp,
        default_key_rotation_frequency -> KeyRotationFrequency,
        registration_mode -> RegistrationMode,
        require_email_registration -> Bool,
        require_email_verification -> Bool,
        require_captcha -> Bool,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_keys (id) {
        id -> Int8,
        user_id -> Int8,
        created -> Timestamp,
        expires_at -> Timestamp,
        public_key -> Text,
        encrypted_secret_key -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::KeyRotationFrequency;

    users (id) {
        id -> Int8,
        created -> Timestamp,
        #[max_length = 20]
        name -> Varchar,
        admin -> Bool,
        #[max_length = 30]
        display_name -> Nullable<Varchar>,
        key_rotation_frequency -> KeyRotationFrequency,
        #[max_length = 254]
        email -> Nullable<Varchar>,
        email_verified -> Bool,
        access_key_hash -> Text,
        encrypted_symmetric_key -> Text,
        salt -> Text,
        updated -> Nullable<Timestamp>,
    }
}

diesel::joinable!(user_keys -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    instance_settings,
    user_keys,
    users,
);
