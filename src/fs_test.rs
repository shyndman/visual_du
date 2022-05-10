mod walk_dir_level_order;

use crate::walk_dir_level_order::*;
use std::{env, error::Error, fs::canonicalize, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let root_path_buf = canonicalize(args.first().map_or(".", |path| path))?;

    thread::spawn(move || {
        walk_dir(root_path_buf.as_path()).unwrap();
    })
    .join()
    .unwrap();

    Ok(())
}

fn walk_dir(root_path: &std::path::Path) -> Result<(), Box<dyn Error>> {
    for entity in walk_dir_in_level_order(root_path)? {
        let entry = entity?;
        println!(
            "{:4}b: ({}) {}",
            entry.size(),
            entry.depth,
            entry
                .path
                .strip_prefix(root_path.parent().unwrap())?
                .to_str()
                .unwrap()
        );
    }

    Ok(())
}
