use serde::Deserialize;
use tokio_postgres::{Client, Statement};

#[derive(Deserialize)]
pub struct Configuration {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
}

pub struct DbState {
    pub client: Client,
    pub insert_statement_v1: Statement,
    pub insert_statement_v2: Statement,
}

#[derive(Deserialize)]
pub struct StatsV3Query {
    /// item name
    pub n: Option<String>,
    /// item id
    pub i: Option<String>,
    /// user id
    pub u: Option<String>,
    /// session_id
    pub s: Option<String>,
}

#[derive(Deserialize)]
pub struct StatsV4Query {
    /// item name
    pub n: Option<String>,
    /// item id
    pub i: Option<String>,
    /// user id
    pub u: Option<String>,
    /// session_id
    pub s: Option<String>,
    /// world_url
    pub w: Option<String>,
    /// neos version
    pub v: Option<String>,
    /// client major version
    pub c1: Option<u16>,
    /// client minor version
    pub c2: Option<u16>,
}
