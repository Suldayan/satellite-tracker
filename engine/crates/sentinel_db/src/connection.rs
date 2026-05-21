use postgres::{Client, NoTls};
use crate::error::{DbError, DbResult};

pub fn connect(database_url: &str) -> DbResult<Client> {
    Client::connect(database_url, NoTls).map_err(DbError::Connection)
}