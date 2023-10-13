use serde::{Serialize, Deserialize};
use validator::Validate;

use crate::db::schema::quotes;

#[derive(Serialize, Deserialize, Queryable, Debug, Insertable, Clone, AsChangeset)]
#[diesel(table_name = quotes)]
pub struct Quote {
    pub id: String,
    pub author: String,
    pub quote: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ApiPayloadQuote {
    #[validate(length(min = 10))]
    pub author: String,
    #[validate(length(min = 5))]
    pub quote: String,
}
