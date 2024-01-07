mod db;
mod keylogger;
mod query;

use std::{sync::Arc, time::Duration};

use log::info;
use tokio::{join, sync::Mutex, task::JoinHandle};

#[tokio::main]
async fn main() {
    initialize_log();

    let db_path = "events.db";
    let db = Arc::new(Mutex::new(db::initialize_db(db_path).await));

    let keylogger_handle: JoinHandle<()>;
    {
        let db = db.clone();
        keylogger_handle = tokio::task::spawn(async move { keylogger::log_keys(db).await });
    }

    // let query_handle: JoinHandle<()>;
    // {
    //     let db = db.clone();
    //     query_handle = tokio::task::spawn(async move {
    //         loop {
    //             query::query(db.clone()).await;
    //         }
    //     });
    // }

    let handle_results = join!(keylogger_handle /* , query_handle*/);
    handle_results.0.unwrap();
    // handle_results.1.unwrap();
}

fn initialize_log() {
    #[cfg(not(debug_assertions))]
    let crate_log_level = log::LevelFilter::Debug;

    #[cfg(debug_assertions)]
    let crate_log_level = log::LevelFilter::Trace;

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
        .level_for(env!("CARGO_PKG_NAME"), crate_log_level)
        .chain(fern::log_file("output.log").unwrap())
        .apply()
        .unwrap();

    info!(
        "STARTED {} {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        crate_log_level
    );
}
