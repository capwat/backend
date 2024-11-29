use chrono::NaiveDateTime;
use diesel::deserialize::QueryableByName;

pub struct ConnectPoolInfo {
    pub base_url: String,
    pub db_name: String,
}

#[derive(Debug, QueryableByName)]
#[diesel(table_name = databases)]
#[allow(unused)]
pub struct DatabaseEntry {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

diesel::table! {
    databases (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp
    }
}
