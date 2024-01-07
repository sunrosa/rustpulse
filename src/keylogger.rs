use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use inputbot::{handle_input_events, KeybdKey};
use log::{debug, info, trace};
use sqlx::{QueryBuilder, Sqlite, SqliteConnection};
use tokio::sync::{mpsc::Sender, Mutex};

pub async fn log_keys(db: Arc<Mutex<SqliteConnection>>) {
    trace!("Begin initializing keylogger...");
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<(DateTime<Utc>, KeybdKey)>(10000);
    register_bindings(sender).await;

    let exit_after_commit = ctrl_c_handler().await;
    trace!("Ctrl-c post-register.");

    // Why won't you run???
    tokio::task::spawn(async move {
        loop {
            trace!("Top of commit loop.");
            tokio::time::sleep(Duration::from_secs(20)).await;

            let mut buffer: Vec<(DateTime<Utc>, KeybdKey)> = Vec::new();
            receiver.recv_many(&mut buffer, 10000).await;

            debug!("Begin MPSC reception. {} keys to commit.", buffer.len(),);

            if buffer.len() != 0 {
                commit_keys_to_db(db.clone(), buffer).await;
            }

            if *exit_after_commit.lock().await {
                info!("Exiting...");
                std::process::exit(0);
            }
        }
    });

    info!("Keyboard event handler to come alive...");
    handle_input_events();
}

async fn register_bindings(sender: Sender<(DateTime<Utc>, KeybdKey)>) {
    debug!("Registering all bindings...");
    KeybdKey::bind_all(move |key| {
        sender
            .blocking_send((Utc::now(), key))
            .expect("The receiver has hung up.");
    });
}

/// # Returns
/// A shared reference to a mutex that will be true if ctrl-c has been pressed once or more, or otherwise false.
async fn ctrl_c_handler() -> Arc<Mutex<bool>> {
    let exit = Arc::new(Mutex::new(false));

    {
        let exit = exit.clone();
        tokio::task::spawn(async move {
            trace!("Registering ctrl-c handler...");
            tokio::signal::ctrl_c()
                .await
                .expect("Error registering ctrl-c handler.");

            info!("Exiting after next database commit...");
            println!("Exiting after next database commit...");
            *exit.lock().await = true;
        });
    }

    exit
}

async fn commit_keys_to_db(
    db: Arc<Mutex<SqliteConnection>>,
    keypresses: Vec<(DateTime<Utc>, KeybdKey)>,
) {
    trace!("Building database query...");
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO keypresses (timestamp, key) ");

    query_builder.push_values(keypresses.iter(), |mut b, key| {
        b.push_bind(key.0.timestamp())
            .push_bind(u64::from(key.1) as i64);
    });

    debug!("Committing to database...");
    query_builder
        .build()
        .execute(&mut *db.lock().await)
        .await
        .unwrap();
}
