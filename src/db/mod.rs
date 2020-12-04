use std::env;
mod postgres;

use lazy_static::lazy_static;
use diesel::{
    r2d2::{ Pool, ConnectionManager, PooledConnection},
    pg::PgConnection
};

pub use diesel::pg::Pg as db_type;

pub fn db() -> DBPooledConnection {
    HANDLER.db_connection.get().
        expect(&format!("Error connecting to {}", db_url()))
}

type DBPool = Pool<ConnectionManager<PgConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

struct Handler {
    db_connection: DBPool
}

lazy_static! {
    static ref HANDLER: Handler = {
        Handler { db_connection:
            DBPool::builder().
            max_size(db_pool()).
            build(ConnectionManager::new(db_url())).
            expect("Failed to create db connection pool")
        }
    };
}

fn db_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}
fn db_pool() -> u32 { env::var("DB_POOL").expect("DB_POOL must be set").parse::<u32>().ok().expect("DB_POOL must be an unsigned integer") }