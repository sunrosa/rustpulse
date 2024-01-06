mod stats;

use std::{ops::Deref, thread, time::Duration};

use inputbot::{handle_input_events, KeybdKey};
use sqlx::{migrate::MigrateDatabase, QueryBuilder, Sqlite, SqlitePool};

#[tokio::main]
async fn main() {
    register_bindings();

    tokio::task::spawn(async {
        let db_path = "events.db";

        if !Sqlite::database_exists(db_path).await.unwrap_or(false) {
            println!("Creating database at {db_path}...");
            match Sqlite::create_database(db_path).await {
                Ok(_) => println!("Success creating database."),
                Err(error) => panic!("Error creating database: {}", error),
            }
        } else {
            println!("Database already exists at {db_path}.");
        }

        let db = SqlitePool::connect(db_path).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS keypresses (id INTEGER PRIMARY KEY NOT NULL, timestamp INTEGER NOT NULL, key INTEGER NOT NULL);").execute(&db).await.unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(20)).await;

            println!("Committing to database...");

            let mut query_builder: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT INTO keypresses (timestamp, key) ");

            query_builder.push_values(stats::get_keypresses().lock().await.iter(), |mut b, key| {
                b.push_bind(key.0.timestamp())
                    .push_bind(u64::from(key.1) as i64);
            });

            query_builder.build().execute(&db).await.unwrap();

            stats::reset_keypresses().await;
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
