use std::{collections::HashMap, env, fs, path, sync::mpsc, thread};
use visual_du::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let root_path_buf = fs::canonicalize(args.first().map_or("../target", |p| p))?;
    let moved_root_path_buf = root_path_buf.clone();

    let (send_channel, receive_channel) = mpsc::sync_channel::<FsEntity>(64);
    thread::spawn(move || {
        for entity in walk_dir(moved_root_path_buf).unwrap() {
            send_channel.send(entity.unwrap()).unwrap();
        }
    });

    let mut sizes_by_path: HashMap<String, u64> = HashMap::new();
    for entity in receive_channel {
        let abs_path: path::PathBuf = entity.path.clone();
        let rel_path = abs_path.strip_prefix(&root_path_buf.parent().unwrap())?;
        let path_ancestors = rel_path
            .ancestors()
            .take_while(|p| !p.as_os_str().is_empty());
        for path in path_ancestors {
            let key = path.to_str().unwrap();
            let size = sizes_by_path.entry(key.into()).or_insert(0);
            *size += entity.size_in_bytes();

            println!("{key:?} is now {size}b (+{delta})", delta = entity.size_in_bytes());
        }
    }

    Ok(())
}
