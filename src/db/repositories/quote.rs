use diesel::prelude::*;
use crate::db::entities::quote::Quote;
use crate::db::schema::quotes::dsl::*;

pub struct QuoteRepository;

impl QuoteRepository {
    pub fn get_quotes(&self, limit: Option<i64>, connection: &mut PgConnection) -> QueryResult<Vec<Quote>> {
        quotes
            .limit(limit.unwrap_or(10))
            .load( connection)
    }

    pub fn get_quote(&self, other_id: String, connection: &mut PgConnection) -> QueryResult<Quote> {

        quotes
            .find(other_id)
            .first(connection)
    }

    pub fn remove(&self, other_id: String, connection: &mut PgConnection) -> QueryResult<usize> {

        return diesel::delete(
            quotes
            .find(other_id)
        ).execute(connection);
    }

    pub fn insert(&self, quote_new: Quote, connection: &mut PgConnection) -> QueryResult<usize> {

        return diesel::insert_into(quotes)
            .values(&quote_new)
            .execute(connection);
    }

    pub fn update(&self, quote_new: Quote, connection: &mut PgConnection) -> QueryResult<usize> {
        return diesel::update(quotes)
            .set(&quote_new)
            .execute(connection);
    }
}
