use diesel::{PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;

pub fn build_db_pool (database_url: String) -> Pool<ConnectionManager<PgConnection>> {
    let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(database_url);
    diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid")
}
