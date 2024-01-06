use inputbot::KeybdKey;
use inquire::Select;
use sqlx::Row;

use crate::db;

pub async fn query(db_path: &str) {
    let selection = Select::new("Query >", vec!["All keypresses", "Each keypresses"])
        .prompt()
        .unwrap();

    match selection {
        "All keypresses" => total_keypresses(db_path).await,
        "Each keypresses" => each_keypresses(db_path).await,
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

    println!(
        "{} keys have been pressed. This includes all control keys.",
        keypress_count
    );
}

async fn each_keypresses(db_path: &str) {
    let mut db = db::initialize_db(db_path).await;

    let mut output = String::new();
    for i in 0x08..0xBB {
        if let KeybdKey::OtherKey(_) = KeybdKey::from(i) {
            continue;
        }

        let presses: i64 = sqlx::query("SELECT COUNT(*) as count FROM keypresses WHERE key == ?")
            .bind(i as i64)
            .fetch_one(&mut db)
            .await
            .unwrap()
            .get(0);

        output += &format!("{:?}: {}\n", KeybdKey::from(i), presses);
    }

    println!("{output}");
}
