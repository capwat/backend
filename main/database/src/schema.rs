// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int8,
        created_at -> Timestamp,
        #[max_length = 25]
        name -> Varchar,
        email -> Nullable<Varchar>,
        #[max_length = 30]
        display_name -> Nullable<Varchar>,
        password_hash -> Text,
        updated_at -> Nullable<Timestamp>,
    }
}
