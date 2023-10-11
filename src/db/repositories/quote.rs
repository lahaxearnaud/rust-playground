use diesel::prelude::*;
use crate::db::entities::quote::Quote;
use crate::db::schema::quotes::dsl::*;
use crate::db::connexion::establish_connection;

pub struct QuoteRepository;

impl QuoteRepository {
    pub fn get_quotes(&self) -> Vec<Quote> {
        let mut connection = establish_connection();

        quotes
            .limit(5)
            .load(&mut connection)
            .unwrap()
    }
}
