use std::{
    ops::Deref,
    sync::{Mutex, MutexGuard},
};

use chrono::{DateTime, Utc};
use inputbot::KeybdKey;
use once_cell::sync::Lazy;

static PRESSES: Lazy<Mutex<Vec<(DateTime<Utc>, KeybdKey)>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn add_keypress(key: KeybdKey) {
    PRESSES.lock().unwrap().push((Utc::now(), key));
}

pub fn get_keypresses<'a>() -> &'a Mutex<Vec<(DateTime<Utc>, KeybdKey)>> {
    PRESSES.deref()
}

pub fn reset_keypresses() {
    *PRESSES.lock().unwrap() = Vec::new();
}
