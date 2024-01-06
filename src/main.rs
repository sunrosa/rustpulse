use inputbot::{from_keybd_key, handle_input_events, KeybdKey};

fn main() {
    register_bindings();
    handle_input_events();
}

fn register_bindings() {
    KeybdKey::bind_all(keypress_handler);
}

fn keypress_handler(key: KeybdKey) {
    println!("{key:?}");
}
