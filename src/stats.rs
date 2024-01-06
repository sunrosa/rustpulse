use std::ops::Deref;

use chrono::{DateTime, Utc};
use inputbot::KeybdKey;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

static PRESSES: Lazy<Mutex<Vec<(DateTime<Utc>, KeybdKey)>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub async fn add_keypress(key: KeybdKey) {
    PRESSES.lock().await.push((Utc::now(), key));
}

pub fn add_keypress_blocking(key: KeybdKey) {
    PRESSES.blocking_lock().push((Utc::now(), key));
}

pub fn get_keypresses<'a>() -> &'a Mutex<Vec<(DateTime<Utc>, KeybdKey)>> {
    PRESSES.deref()
}

pub async fn reset_keypresses() {
    *PRESSES.lock().await = Vec::new();
}
