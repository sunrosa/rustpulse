mod db;
mod keylogger;
mod query;

use std::sync::Arc;

use inquire::Select;
use log::info;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    initialize_log();

    let db_path = "events.db";
    let db = Arc::new(Mutex::new(db::initialize_db(db_path).await));

    {
        let db = db.clone();
        tokio::task::spawn(async move { keylogger::log_keys(db).await });
    }

    loop {
        query::query(db.clone()).await;
    }
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
