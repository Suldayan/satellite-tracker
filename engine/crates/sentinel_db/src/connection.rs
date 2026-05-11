use postgres::{Client, NoTls};
use dotenvy::dotenv;
use crate::error::{DbError, DbResult};

pub fn connect() -> DbResult<Client> {
    dotenv().ok();

    let conn_str = std::env::var("DATABASE_URL")
        .map_err(|_| DbError::MissingConnectionString)?;

    Client::connect(&conn_str, NoTls).map_err(DbError::Connection)
}