use inquire::Select;
use sqlx::Row;

use crate::db;

pub async fn query(db_path: &str) {
    let selection = Select::new("Query >", vec!["Total keypresses"])
        .prompt()
        .unwrap();

    match selection {
        "Total keypresses" => total_keypresses(db_path).await,
        _ => unreachable!(),
    }
}

async fn total_keypresses(db_path: &str) {
    let mut db = db::initialize_db(db_path).await;

    let row = sqlx::query("SELECT COUNT(*) as count FROM keypresses;")
        .fetch_one(&mut db)
        .await
        .unwrap();
    let keypress_count: i64 = row.get(0);

    println!("{}", keypress_count);
}
