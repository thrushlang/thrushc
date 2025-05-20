use std::env;

fn main() {
    unsafe { env::set_var("CARGO_TERM_VERBOSE", "true") };

    if cfg!(target_os = "windows") {
        colored::control::set_override(true);
    }
}
