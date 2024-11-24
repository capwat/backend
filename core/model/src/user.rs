use bon::Builder;
use capwat_api_types::util::Sensitive;
use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub created: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub password_hash: String,
}

#[derive(Debug, Builder)]
pub struct InsertUser<'a> {
    #[builder(into)]
    pub name: Sensitive<&'a str>,
    pub display_name: Option<Sensitive<&'a str>>,
    pub email: Option<Sensitive<&'a str>>,
    #[builder(into)]
    pub password_hash: Sensitive<&'a str>,
}

#[derive(Debug, Builder)]
pub struct UpdateUser<'a> {
    pub id: i64,
    #[builder(into)]
    pub name: Option<Sensitive<&'a str>>,
    pub display_name: Option<Option<Sensitive<&'a str>>>,
    #[builder(into)]
    pub email: Option<Sensitive<&'a str>>,
    #[builder(into)]
    pub password_hash: Option<Sensitive<&'a str>>,
}
