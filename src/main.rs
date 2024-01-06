use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use inputbot::{handle_input_events, KeybdKey};
use log::{debug, error, info, trace};
use sqlx::{migrate::MigrateDatabase, Connection, QueryBuilder, Sqlite, SqliteConnection};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    initialize_log();

    let keypress_queue: Arc<Mutex<Vec<(DateTime<Utc>, KeybdKey)>>> =
        Arc::new(Mutex::new(Vec::new()));
    register_bindings(keypress_queue.clone());

    {
        let keypress_queue = keypress_queue.clone();
        tokio::task::spawn(async move {
            let db_path = "events.db";

            let exit_after_commit = Arc::new(Mutex::new(false));

            {
                let exit_after_commit = exit_after_commit.clone();
                tokio::task::spawn(async move {
                    trace!("Registering ctrl-c handler...");
                    tokio::signal::ctrl_c()
                        .await
                        .expect("Error registering ctrl-c handler.");

                    info!("Exiting after next database commit...");
                    *exit_after_commit.lock().await = true;
                });
            }

            if !Sqlite::database_exists(db_path).await.unwrap_or(false) {
                info!("Creating database at {db_path}...");
                match Sqlite::create_database(db_path).await {
                    Ok(_) => info!("Success creating database."),
                    Err(error) => error_panic(format!("Error creating database: {}", error)),
                }
            } else {
                debug!("Database exists at {db_path}.");
            }

            debug!("Connecting to database...");
            let mut db = SqliteConnection::connect(db_path).await.unwrap();
            debug!("Connected to database.");

            sqlx::query("CREATE TABLE IF NOT EXISTS keypresses (id INTEGER PRIMARY KEY NOT NULL, timestamp INTEGER NOT NULL, key INTEGER NOT NULL);").execute(&mut db).await.unwrap();

            loop {
                trace!("Top of commit loop.");
                tokio::time::sleep(Duration::from_secs(20)).await;

                let mut keypress_lock = keypress_queue.lock().await;
                let keypresses = keypress_lock.drain(..).into_iter();

                if keypresses.len() != 0 {
                    trace!("Building database query...");
                    let mut query_builder: QueryBuilder<Sqlite> =
                        QueryBuilder::new("INSERT INTO keypresses (timestamp, key) ");

                    query_builder.push_values(keypresses, |mut b, key| {
                        b.push_bind(key.0.timestamp())
                            .push_bind(u64::from(key.1) as i64);
                    });

                    debug!("Committing to database...");
                    query_builder.build().execute(&mut db).await.unwrap();
                }

                if *exit_after_commit.lock().await {
                    info!("Exiting...");
                    std::process::exit(0);
                }
            }
        });
    }

    info!("Keyboard event handler to come alive...");
    handle_input_events();
}

fn register_bindings(keypress_queue: Arc<Mutex<Vec<(DateTime<Utc>, KeybdKey)>>>) {
    debug!("Registering all bindings...");
    KeybdKey::bind_all(move |key| {
        keypress_queue.blocking_lock().push((Utc::now(), key));
    });
}

fn initialize_log() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.9f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .apply()
        .unwrap();

    info!(
        "STARTED {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
}

fn error_panic(output: String) {
    error!("{output}");
    panic!("{output}");
}
