use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("satbounty")]
pub struct Db(pub sqlx::SqlitePool);
