use log::{debug, info, error};
use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};

pub async fn initialize_db(db_path: &str) -> SqliteConnection {
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

    db
}

fn error_panic(output: String) {
    error!("{output}");
    panic!("{output}");
}
