use postgres::{Client, NoTls};
use std::error::Error;

pub fn connect() -> Result<Client, Box<dyn Error>> {
    let client = Client::connect(
        "host=localhost user=postgres password=secret dbname=gisdb",
        NoTls,
    )?;

    Ok(client)
}