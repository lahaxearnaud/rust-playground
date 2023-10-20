use std::env;
use diesel::{PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;

pub fn build_db_pool () -> Pool<ConnectionManager<PgConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(database_url);
    return diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid");
}
