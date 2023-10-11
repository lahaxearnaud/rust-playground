use diesel::prelude::*;
use crate::db::entities::quote::Quote;
use crate::db::schema::quotes::dsl::*;
use crate::db::connexion::establish_connection;

pub struct QuoteRepository;

impl QuoteRepository {
    pub fn get_quotes(&self, limit: Option<i64>) -> QueryResult<Vec<Quote>> {
        let mut connection = establish_connection();

        quotes
            .limit(limit.unwrap_or(10))
            .load(&mut connection)
    }

    pub fn get_quote(&self, other_id: String) -> QueryResult<Quote> {
        let mut connection = establish_connection();

        quotes
            .find(other_id)
            .first(&mut connection)
    }

    pub fn remove(&self, other_id: String) {
        let mut connection = establish_connection();

        diesel::delete(
            quotes
            .find(other_id)
        )
        .execute(&mut connection)
        .expect("Error deleting posts");
    }
}
