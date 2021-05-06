use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, SecondsFormat, Utc};
use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::types::{ToSql, Type};
use warp::Filter;
use warp::http::{Response, StatusCode};

use crate::dto::{Configuration, DbState, StatsV3Query, StatsV4Query};

mod dto;

const PNG: &[u8] = include_bytes!("resources/tracking-pixel.png");
const DEFAULT_CONFIG_FILE_NAME: &str = "statistics_collector.toml";
const DEFAULT_CERTIFICATE_FILE_NAME: &str = "db_cert.pem";

const STATISTICS_STATEMENT_V1: &str = "insert into statistics \
(item_name, item_id, user_id, session_id, timestamp, protocol_version, cache_nonce) \
values ($1, $2, $3, $4, $5, $6, $7)";
const STATISTICS_STATEMENT_TYPES_V1: &[Type] = &[
    Type::TEXT, Type::VARCHAR, Type::VARCHAR, Type::VARCHAR,
    Type::INET, Type::TIMESTAMP, Type::INT2, Type::VARCHAR,
];

// schema v2 replaces world_id with world_url
const STATISTICS_STATEMENT_V2: &str = "insert into statistics \
(item_name, item_id, user_id, session_id, world_url, timestamp, protocol_version, neos_version,\
cache_nonce, client_major_version, client_minor_version) \
values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)";
const STATISTICS_STATEMENT_TYPES_V2: &[Type] = &[
    Type::TEXT, Type::VARCHAR, Type::VARCHAR, Type::VARCHAR, Type::VARCHAR,
    Type::INET, Type::TIMESTAMP, Type::INT2, Type::VARCHAR, Type::VARCHAR,
    Type::INT2, Type::INT2,
];

type Db = Arc<DbState>;

#[tokio::main]
async fn main() {
    println!("[{}] Initializing {} {}", iso_string(), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let server_address: SocketAddr = ([0, 0, 0, 0], 3034).into();

    let server_cert = get_default_config_file(DEFAULT_CERTIFICATE_FILE_NAME)
        .and_then(|path|
            fs::read(path)
                .map_err(|e| format!("{:?}", e))
        )
        .and_then(|file_contents|
            Certificate::from_pem(&file_contents)
                .map_err(|e| format!("{:?}", e))
        )
        .unwrap_or_else(|e| {
            eprintln!("[{}] Could not read server certificate: {}", iso_string(), e);
            std::process::exit(1);
        });

    let config: Configuration = get_default_config_file(DEFAULT_CONFIG_FILE_NAME)
        .and_then(|path|
            fs::read_to_string(path)
                .map_err(|e| format!("{:?}", e))
        )
        .and_then(|file_contents|
            toml::from_str(&file_contents)
                .map_err(|e| format!("{:?}", e))
        )
        .unwrap_or_else(|e| {
            eprintln!("[{}] Could not read config: {}", iso_string(), e);
            std::process::exit(1);
        });

    let connector = TlsConnector::builder()
        .disable_built_in_roots(true)
        .add_root_certificate(server_cert)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("[{}] Could not build tls connector: {}", iso_string(), e);
            std::process::exit(1);
        });
    let connector = MakeTlsConnector::new(connector);

    let (client, connection) = tokio_postgres::Config::new()
        .user(&config.db_user)
        .password(&config.db_password)
        .dbname(&config.db_name)
        .host(&config.db_host)
        .port(config.db_port)
        .application_name(format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_str())
        .connect(connector).await
        .unwrap_or_else(|e| {
            eprintln!("[{}] Could not connect to database: {}", iso_string(), e);
            std::process::exit(1);
        });

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("[{}] database connection error: {}", iso_string(), e);
        }
    });

    let insert_statement_v1 = client.prepare_typed(STATISTICS_STATEMENT_V1, STATISTICS_STATEMENT_TYPES_V1).await
        .unwrap_or_else(|e| {
            eprintln!("[{}] Could not prepare statement: {}", iso_string(), e);
            std::process::exit(1);
        });

    let insert_statement_v2 = client.prepare_typed(STATISTICS_STATEMENT_V2, STATISTICS_STATEMENT_TYPES_V2).await
        .unwrap_or_else(|e| {
            eprintln!("[{}] Could not prepare statement: {}", iso_string(), e);
            std::process::exit(1);
        });

    let db_state = Arc::new(DbState { client, insert_statement_v1, insert_statement_v2 });

    let info = warp::path::end()
        .and(warp::get())
        .map(|| format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

    // never distributed: safe to delete
    // original version
    // /stats/v1/<item_name>/<item_id>/<user_id>/<session_id>/<world_id>/<cache_nonce>/a.png
    let stats_v1 = warp::path!("stats" / "v1" / String / String / String / String / String / String / "a.png")
        .and(warp::get())
        .and(with_state(db_state.clone()))
        .and_then(stats_v1_handler);

    // never used: safe to delete
    // failed experiment in collecting less data
    // /stats/v2/<item_name>/<item_id>/<user_id>/<cache_nonce>/a.png
    let stats_v2 = warp::path!("stats" / "v2" / String / String / String / String / "a.png")
        .and(warp::get())
        .and(with_state(db_state.clone()))
        .and_then(stats_v2_handler);

    // possibly in use
    // switches from path to query parameters
    // /stats/v3/<cache_nonce>/a.png?n=<item_name>&i=<item_id>&u=<user_id>&s=<session_id>
    let stats_v3 = warp::path!("stats" / "v3" / String / "a.png")
        .and(warp::get())
        .and(warp::query::<StatsV3Query>())
        .and(with_state(db_state.clone()))
        .and_then(stats_v3_handler);

    // current version
    // adds world_url field
    // /stats/v4/<cache_nonce>/a.png?n=<item_name>&i=<item_id>&u=<user_id>&s=<session_id>&w=<world_url>
    let stats_v4 = warp::path!("stats" / "v4" / String / "a.png")
        .and(warp::get())
        .and(warp::query::<StatsV4Query>())
        .and(with_state(db_state))
        .and_then(stats_v4_handler);

    let routes = info
        .or(stats_v1)
        .or(stats_v2)
        .or(stats_v3)
        .or(stats_v4);

    println!("[{}] Starting web server on {}...", iso_string(), server_address);
    warp::serve(routes)
        .run(server_address)
        .await;
}

fn with_state<T: Clone + Send>(state: T) -> impl Filter<Extract=(T, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn iso_string() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn empty_string_to_null(string: Option<&str>) -> Option<&str> {
    string.filter(|s| !s.trim().is_empty())
}

fn get_default_config_file(filename: &str) -> Result<PathBuf, String> {
    std::env::current_exe()
        .map_err(|e| format!("{:?}", e))
        .and_then(|path|
            path.parent()
                .map(|p| p.to_path_buf().join(filename))
                .ok_or("Could not find parent directory of this executable".to_string())
        )
}

async fn stats_v1_handler(
    item_name: String, item_id: String, user_id: String, session_id: String, _world_id: String,
    cache_nonce: String, db: Db,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let timestamp = Utc::now();
    let item_name = &percent_encoding::percent_decode_str(&item_name).decode_utf8_lossy();
    let item_id = &percent_encoding::percent_decode_str(&item_id).decode_utf8_lossy();
    let user_id = &percent_encoding::percent_decode_str(&user_id).decode_utf8_lossy();
    let session_id = &percent_encoding::percent_decode_str(&session_id).decode_utf8_lossy();
    let cache_nonce = &percent_encoding::percent_decode_str(&cache_nonce).decode_utf8_lossy();
    let protocol_version = 1;

    println!(
        "[{}] Got v1 statistics: {} {} {} {} {}",
        timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        item_name,
        item_id,
        user_id,
        session_id,
        cache_nonce,
    );

    Ok(
        statistics_result_to_response(
            record_statistics_v1(
                db, Some(item_name), Some(item_id), Some(user_id),
                Some(session_id), timestamp, protocol_version, cache_nonce,
            ).await
        )
    )
}

async fn stats_v2_handler(
    item_name: String, item_id: String, user_id: String, cache_nonce: String, db: Db,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let timestamp = Utc::now();
    let item_name = &percent_encoding::percent_decode_str(&item_name).decode_utf8_lossy();
    let item_id = &percent_encoding::percent_decode_str(&item_id).decode_utf8_lossy();
    let user_id = &percent_encoding::percent_decode_str(&user_id).decode_utf8_lossy();
    let cache_nonce = &percent_encoding::percent_decode_str(&cache_nonce).decode_utf8_lossy();
    let protocol_version = 2;

    println!(
        "[{}] Got v2 statistics: {} {} {} {}",
        timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        item_name,
        item_id,
        user_id,
        cache_nonce,
    );

    Ok(
        statistics_result_to_response(
            record_statistics_v1(
                db, Some(item_name), Some(item_id), Some(user_id),
                None, timestamp, protocol_version, cache_nonce,
            ).await
        )
    )
}

async fn stats_v3_handler(
    cache_nonce: String, query_params: StatsV3Query, db: Db,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let timestamp = Utc::now();
    let item_name = query_params.n.as_ref().map(String::as_str);
    let item_id = query_params.i.as_ref().map(String::as_str);
    let user_id = query_params.u.as_ref().map(String::as_str);
    let session_id = query_params.s.as_ref().map(String::as_str);
    let cache_nonce = &percent_encoding::percent_decode_str(&cache_nonce).decode_utf8_lossy();
    let protocol_version = 3;

    println!(
        "[{}] Got v3 statistics: {:?} {:?} {:?} {:?} {}",
        timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        item_name,
        item_id,
        user_id,
        session_id,
        cache_nonce,
    );

    Ok(
        statistics_result_to_response(
            record_statistics_v1(
                db, item_name, item_id, user_id, session_id,
                timestamp, protocol_version, cache_nonce,
            ).await
        )
    )
}

async fn stats_v4_handler(
    cache_nonce: String, query_params: StatsV4Query, db: Db,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let timestamp = Utc::now();
    let item_name = query_params.n.as_ref().map(String::as_str);
    let item_id = query_params.i.as_ref().map(String::as_str);
    let user_id = query_params.u.as_ref().map(String::as_str);
    let session_id = query_params.s.as_ref().map(String::as_str);
    let world_url = query_params.w.as_ref().map(String::as_str);
    let neos_version = query_params.v.as_ref().map(String::as_str);
    let cache_nonce = &percent_encoding::percent_decode_str(&cache_nonce).decode_utf8_lossy();
    let client_major_version = query_params.c1;
    let client_minor_version = query_params.c2;
    let protocol_version = 4;

    println!(
        "[{}] Got v4 statistics: {:?} {:?} {:?} {:?} {:?} {:?} {} {:?} {:?}",
        timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        item_name,
        item_id,
        user_id,
        session_id,
        world_url,
        neos_version,
        cache_nonce,
        client_major_version,
        client_minor_version,
    );

    Ok(
        statistics_result_to_response(
            record_statistics_v2(
                db, item_name, item_id, user_id, session_id,
                world_url, timestamp, protocol_version, neos_version, cache_nonce,
                client_major_version, client_minor_version,
            ).await
        )
    )
}

fn statistics_result_to_response(result: Result<(), String>) -> Box<dyn warp::Reply> {
    match result {
        Ok(_) => {
            Box::new(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "image/png")
                    .body(PNG)
            )
        }
        Err(e) => {
            eprintln!("[{}] {}", iso_string(), e);
            Box::new(
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "text/plain")
                    .body(format!("{:?}", e))
            )
        }
    }
}

async fn record_statistics_v1(
    db: Db, item_name: Option<&str>, item_id: Option<&str>, user_id: Option<&str>, session_id: Option<&str>,
    timestamp: DateTime<Utc>, protocol_version: u16, cache_nonce: &str,
) -> Result<(), String> {
    let item_name = empty_string_to_null(item_name);
    let item_id = empty_string_to_null(item_id);
    let user_id = empty_string_to_null(user_id);
    let session_id = empty_string_to_null(session_id);
    let protocol_version: i16 = protocol_version as i16;

    // (item_name, item_id, user_id, session_id, timestamp)
    let params: &[&(dyn ToSql + Sync)] = &[
        &item_name, &item_id, &user_id, &session_id, &timestamp.naive_utc(),
        &protocol_version, &cache_nonce,
    ];
    match db.client.execute(&db.insert_statement_v1, params).await {
        Ok(update_count) => if update_count == 1 {
            Ok(())
        } else {
            Err(format!("expected 1, but updated {} rows", update_count))
        },
        Err(e) => Err(format!("{:?}", e))
    }
}

async fn record_statistics_v2(
    db: Db, item_name: Option<&str>, item_id: Option<&str>, user_id: Option<&str>, session_id: Option<&str>,
    world_url: Option<&str>, timestamp: DateTime<Utc>, protocol_version: u16,
    neos_version: Option<&str>, cache_nonce: &str, client_major_version: Option<u16>,
    client_minor_version: Option<u16>,
) -> Result<(), String> {
    let item_name = empty_string_to_null(item_name);
    let item_id = empty_string_to_null(item_id);
    let user_id = empty_string_to_null(user_id);
    let session_id = empty_string_to_null(session_id);
    let world_url = empty_string_to_null(world_url);
    let protocol_version: i16 = protocol_version as i16;
    let client_major_version = client_major_version.map(|v| v as i16);
    let client_minor_version = client_minor_version.map(|v| v as i16);

    // (item_name, item_id, user_id, session_id, world_url, timestamp)
    let params: &[&(dyn ToSql + Sync)] = &[
        &item_name, &item_id, &user_id, &session_id, &world_url, &timestamp.naive_utc(),
        &protocol_version, &neos_version, &cache_nonce, &client_major_version, &client_minor_version,
    ];
    match db.client.execute(&db.insert_statement_v2, params).await {
        Ok(update_count) => if update_count == 1 {
            Ok(())
        } else {
            Err(format!("expected 1, but updated {} rows", update_count))
        },
        Err(e) => Err(format!("{:?}", e))
    }
}
