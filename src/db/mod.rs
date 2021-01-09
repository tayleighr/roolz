mod postgres;

use {
    lazy_static::lazy_static,
    diesel::{
        r2d2::{ Pool, ConnectionManager, PooledConnection},
        pg::PgConnection
    },
    std::env,
    crate::error::*
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
        Handler {
            db_connection:
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

pub type DBResult<T> = std::result::Result<T, DBError>;

#[derive(Debug)]
pub enum DBError {
    Conflict(diesel::result::Error),
    Connection(diesel::result::ConnectionError),
    App(AppError)
}

impl ErrorMeta for DBError {
    fn code(&self) -> Option<StatusCode> {
        match &*self {
            DBError::Conflict(_) => Some(StatusCode::CONFLICT),
            DBError::Connection(_) => Some(StatusCode::SERVICE_UNAVAILABLE),
            DBError::App(e) => e.code()
        }
    }

    fn typ(&self) -> String {
        String::from("DBError")
    }

    fn kind(&self) -> String {
        match &*self {
            DBError::Conflict(_) => String::from("Conflict"),
            DBError::Connection(_) => String::from("Connection"),
            DBError::App(e) => e.kind()
        }
    }

    fn origin(&self) -> String {
        match &*self {
            DBError::Conflict(_) => String::from("diesel::result::Error"),
            DBError::Connection(_) => String::from("diesel::result::ConnectionError"),
            DBError::App(e) => e.origin()
        }
    }

    fn reason(&self) -> String {
        use std::error::Error;

        if let Some(source) = self.source() {
            source.to_string()
        } else {
            String::from("No reason provided")
        }
    }
}

impl std::error::Error for DBError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            DBError::Conflict( ref e ) => Some(e),
            DBError::Connection( ref e ) => Some(e),
            DBError::App( ref e ) => Some(e)
        }
    }
}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            DBError::Conflict( e ) => e.fmt(f),
            DBError::Connection( e ) => e.fmt(f),
            DBError::App( e ) => e.fmt(f)
        }
    }
}

impl std::convert::From<DBError> for AppError {
    fn from(error: DBError) -> Self {
        AppError::From(Box::new(error))
    }
}

impl std::convert::From<AppError> for DBError {
    fn from(error: AppError) -> Self {
        DBError::App(error)
    }
}

impl std::convert::From<diesel::result::Error> for DBError {
    fn from(e: diesel::result::Error) -> DBError {
        DBError::Conflict(e)
    }
}

impl std::convert::From<diesel::result::ConnectionError> for DBError {
    fn from(e: diesel::result::ConnectionError) -> DBError {
        DBError::Connection(e)
    }
}

impl ForApp<diesel::result::Error> for DBError {
    fn for_app(e: diesel::result::Error) -> AppError {
        Self::from(e).into()
    }
}

impl ForApp<diesel::result::ConnectionError> for DBError {
    fn for_app(e: diesel::result::ConnectionError) -> AppError {
        Self::from(e).into()
    }
}