// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "registration_mode"))]
    pub struct RegistrationMode;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RegistrationMode;

    instance_settings (id) {
        id -> Int4,
        created -> Timestamp,
        registration_mode -> RegistrationMode,
        require_email_registration -> Bool,
        require_email_verification -> Bool,
        require_captcha -> Bool,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        created -> Timestamp,
        updated -> Nullable<Timestamp>,
        #[max_length = 20]
        name -> Varchar,
        #[max_length = 30]
        display_name -> Nullable<Varchar>,
        #[max_length = 254]
        email -> Nullable<Varchar>,
        email_verified -> Bool,
        access_key_hash -> Text,
        root_classic_pk -> Text,
        root_encrypted_classic_sk -> Text,
        root_pqc_pk -> Text,
        root_encrypted_pqc_sk -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    instance_settings,
    users,
);
