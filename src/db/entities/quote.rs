use serde::Serialize;

use crate::db::schema::quotes;

#[derive(Serialize, Queryable)]
pub struct Quote {
    pub id: String,
    pub author: String,
    pub quote: String,
}

#[derive(Insertable)]
#[table_name = "quotes"]
pub struct NewQuote<'a> {
    pub id: &'a str,
    pub author: &'a str,
    pub quote: &'a str,
}
