use std::{env, path::Path};

fn main() {
    // Release builds don't include assets, and so, attempts to load them will fail
    bevy_embasset::include_all_assets(
        &Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets"),
    );
}
