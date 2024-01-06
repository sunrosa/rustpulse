mod stats;

use std::{
    ops::{Deref, DerefMut},
    thread,
    time::Duration,
};

use inputbot::{handle_input_events, KeybdKey};

fn main() {
    register_bindings();

    thread::Builder::new()
        .name("Debugger".into())
        .spawn(|| loop {
            thread::sleep(Duration::from_secs(60));

            let mut output = String::new();
            for key in stats::get_keypresses().lock().unwrap().deref() {
                output += &format!("{}, {:?}\n", key.0.to_rfc3339(), key.1);
            }

            println!("{output}");

            stats::reset_keypresses();
        })
        .unwrap();

    handle_input_events();
}

fn register_bindings() {
    KeybdKey::bind_all(keypress_handler);
}

fn keypress_handler(key: KeybdKey) {
    stats::add_keypress(key);
}
