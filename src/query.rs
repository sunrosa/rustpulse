use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime, Utc};
use inputbot::KeybdKey;
use inquire::{DateSelect, Select};
use sqlx::Row;

use crate::db;

pub async fn query(db_path: &str) {
    let selection = Select::new(
        "Query >",
        vec![
            "All keypresses",
            "Each keypresses",
            "Each keypresses sorted",
            "Specific keypresses",
        ],
    )
    .prompt()
    .unwrap();

    match selection {
        "All keypresses" => total_keypresses(db_path).await,
        "Each keypresses" => each_keypresses(db_path, false).await,
        "Each keypresses sorted" => each_keypresses(db_path, true).await,
        "Specific keypresses" => specific_keypresses(db_path).await,
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

async fn each_keypresses(db_path: &str, sort: bool) {
    let mut db = db::initialize_db(db_path).await;

    let filters = select_filters();

    let mut keys: Vec<(KeybdKey, i64)> = Vec::new();
    for i in 0x08..0xBB {
        if let KeybdKey::OtherKey(_) = KeybdKey::from(i) {
            continue;
        }

        let row = sqlx::query("SELECT COUNT(*), timestamp FROM keypresses WHERE key == ?")
            .bind(i as i64)
            .fetch_one(&mut db)
            .await
            .unwrap();

        let (presses, timestamp): (i64, i64) = (row.get(0), row.get(1));

        if filters.start_date.is_some()
            && NaiveDate::from(NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap())
                < filters.start_date.unwrap()
        {
            continue;
        }

        if filters.end_date.is_some()
            && NaiveDate::from(NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap())
                > filters.end_date.unwrap()
        {
            continue;
        }

        keys.push((KeybdKey::from(i), presses));
    }

    if sort {
        keys.sort_by(|a, b| b.1.cmp(&a.1));
    }

    let mut output = String::new();
    for key in keys {
        output += &format!("{:?}: {}\n", key.0, key.1);
    }

    println!("{output}");
}

async fn specific_keypresses(db_path: &str) {
    let mut db = db::initialize_db(db_path).await;

    let key = select_key();

    let presses: i64 = sqlx::query("SELECT COUNT(*) as count FROM keypresses WHERE key == ?")
        .bind(u64::from(key) as i64)
        .fetch_one(&mut db)
        .await
        .unwrap()
        .get(0);

    println!("{:?}: {}", key, presses);
}

fn select_key() -> KeybdKey {
    let selection = Select::new(
        "Key >",
        vec![
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
            "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "0", ")", "1", "!", "2", "@", "3", "#",
            "4", "$", "5", "%", "6", "^", "7", "&", "8", "*", "9", "(", "`", "~", "/", "?", ",",
            ".", "-", "_", ";", ":", "[", "{", "]", "}", "=", "+", "\\", "|", "'", "\"", "F1",
            "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "SPACE", "CAPS",
            "LSHIFT", "RSHIFT", "LCTRL", "RCTRL", "LALT", "RALT", "LSUPER", "RSUPER", "ESC",
        ],
    )
    .prompt()
    .unwrap();

    let mut selection_char: Option<char> = None;
    if selection.len() == 1 {
        selection_char = Some(selection.chars().next().unwrap());
    }

    let mut key: Option<KeybdKey> = None;
    if selection_char.is_some() {
        key = inputbot::get_keybd_key(selection_char.unwrap());
    }

    let key = match selection {
        "F1" => KeybdKey::F1Key,
        "F2" => KeybdKey::F2Key,
        "F3" => KeybdKey::F3Key,
        "F4" => KeybdKey::F4Key,
        "F5" => KeybdKey::F5Key,
        "F6" => KeybdKey::F6Key,
        "F7" => KeybdKey::F7Key,
        "F8" => KeybdKey::F8Key,
        "F9" => KeybdKey::F9Key,
        "F10" => KeybdKey::F10Key,
        "F11" => KeybdKey::F11Key,
        "F12" => KeybdKey::F12Key,
        "SPACE" => KeybdKey::SpaceKey,
        "CAPS" => KeybdKey::CapsLockKey,
        "LSHIFT" => KeybdKey::LShiftKey,
        "RSHIFT" => KeybdKey::RShiftKey,
        "LCTRL" => KeybdKey::LControlKey,
        "RCTRL" => KeybdKey::RControlKey,
        "LALT" => KeybdKey::LAltKey,
        "RALT" => KeybdKey::RAltKey,
        "LSUPER" => KeybdKey::LSuper,
        "RSUPER" => KeybdKey::RSuper,
        "ESC" => KeybdKey::EscapeKey,
        _ => key.unwrap(),
    };

    key
}

struct Filters {
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
}

impl Filters {
    fn new() -> Filters {
        Filters {
            start_date: None,
            end_date: None,
        }
    }
}

fn select_filters() -> Filters {
    let mut filters = Filters::new();
    loop {
        let selection = Select::new("Filters >", vec!["Done", "Start date", "End date"])
            .prompt()
            .unwrap();
        match selection {
            "Done" => break,
            "Start date" => {
                filters.start_date = Some(DateSelect::new("Start date >").prompt().unwrap());
            }
            "End date" => filters.end_date = Some(DateSelect::new("End date >").prompt().unwrap()),
            _ => unreachable!(),
        }
    }

    filters
}
