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

    pub fn remove(&self, other_id: String) -> QueryResult<usize> {
        let mut connection = establish_connection();

        return diesel::delete(
            quotes
            .find(other_id)
        ).execute(&mut connection);
    }

    pub fn insert(&self, quote_new: Quote) -> QueryResult<usize> {
        let mut connection = establish_connection();
        return diesel::insert_into(quotes)
            .values(&quote_new)
            .execute(&mut connection);
    }

    pub fn update(&self, quote_new: Quote) -> QueryResult<usize> {
        let mut connection = establish_connection();
        return diesel::update(quotes)
            .set(&quote_new)
            .execute(&mut connection);
    }
}
