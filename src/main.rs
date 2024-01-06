mod stats;

use std::{sync::Arc, time::Duration};

use inputbot::{handle_input_events, KeybdKey};
use sqlx::{
    migrate::MigrateDatabase, Connection, QueryBuilder, Sqlite, SqliteConnection, SqlitePool,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    register_bindings();

    tokio::task::spawn(async {
        let db_path = "events.db";

        let exit_after_commit = Arc::new(Mutex::new(false));

        {
            let exit_after_commit_thread = exit_after_commit.clone();
            tokio::task::spawn(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Error registering ctrl-c handler.");

                println!("Exiting after next database commit...");
                *exit_after_commit_thread.lock().await = true;
            });
        }

        if !Sqlite::database_exists(db_path).await.unwrap_or(false) {
            println!("Creating database at {db_path}...");
            match Sqlite::create_database(db_path).await {
                Ok(_) => println!("Success creating database."),
                Err(error) => panic!("Error creating database: {}", error),
            }
        } else {
            println!("Database already exists at {db_path}.");
        }

        let mut db = SqliteConnection::connect(db_path).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS keypresses (id INTEGER PRIMARY KEY NOT NULL, timestamp INTEGER NOT NULL, key INTEGER NOT NULL);").execute(&mut db).await.unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(20)).await;

            let keypresses = stats::get_keypresses().lock().await;
            if keypresses.len() != 0 {
                println!("Committing to database...");

                let mut query_builder: QueryBuilder<Sqlite> =
                    QueryBuilder::new("INSERT INTO keypresses (timestamp, key) ");

                query_builder.push_values(keypresses.iter(), |mut b, key| {
                    b.push_bind(key.0.timestamp())
                        .push_bind(u64::from(key.1) as i64);
                });

                query_builder.build().execute(&mut db).await.unwrap();

                println!("Committed to database.");

                drop(keypresses);
                stats::reset_keypresses().await;
            }

            if *exit_after_commit.lock().await {
                std::process::exit(0);
            }
        }
    });

    handle_input_events();
}

fn register_bindings() {
    KeybdKey::bind_all(keypress_handler);
}

fn keypress_handler(key: KeybdKey) {
    // TODO: Make this go through an MPSC channel into another thread so as not to block.
    stats::add_keypress_blocking(key);
}
