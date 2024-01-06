use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use crossbeam::channel::Sender;
use inputbot::{handle_input_events, KeybdKey};
use log::{debug, info, trace};
use sqlx::{QueryBuilder, Sqlite, SqliteConnection};
use tokio::sync::Mutex;

use crate::db;

pub async fn log_keys(db_path: &str) {
    let (sender, receiver) = crossbeam::channel::bounded::<(DateTime<Utc>, KeybdKey)>(10000);
    register_bindings(sender);

    let exit_after_commit = ctrl_c_handler().await;
    let mut db = db::initialize_db(db_path).await;

    {
        tokio::task::spawn(async move {
            loop {
                trace!("Top of commit loop.");
                tokio::time::sleep(Duration::from_secs(20)).await;

                debug!(
                    "Begin MPSC reception. {}/{} keys to commit.",
                    receiver.len(),
                    receiver.capacity().unwrap()
                );

                let keypresses: Vec<_> = receiver.try_iter().collect();

                if keypresses.len() != 0 {
                    commit_keys_to_db(&mut db, keypresses).await;
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

fn register_bindings(sender: Sender<(DateTime<Utc>, KeybdKey)>) {
    debug!("Registering all bindings...");
    KeybdKey::bind_all(move |key| {
        sender.send((Utc::now(), key)).unwrap();
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

async fn commit_keys_to_db(db: &mut SqliteConnection, keypresses: Vec<(DateTime<Utc>, KeybdKey)>) {
    trace!("Building database query...");
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO keypresses (timestamp, key) ");

    query_builder.push_values(keypresses.iter(), |mut b, key| {
        b.push_bind(key.0.timestamp())
            .push_bind(u64::from(key.1) as i64);
    });

    debug!("Committing to database...");
    query_builder.build().execute(db).await.unwrap();
}
