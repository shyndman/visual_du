use std::{
    cmp::Ordering,
    collections::VecDeque,
    fs::{self, read_dir, DirEntry},
    path::*,
    result::Result,
};

pub struct LevelOrderTraversal {
    pub root_path: PathBuf,
    queue: VecDeque<Result<FsEntity, Error>>,
}

impl LevelOrderTraversal {
    fn enque_children_of(&mut self, parent: &FsEntity) {
        if parent.is_dir() {
            match read_dir(parent.clone().path) {
                Ok(rd) => {
                    let mut child_entities: Vec<_> = rd
                        .map(|e| dir_entry_to_fs_entity(e, parent.depth + 1))
                        .collect();
                    child_entities.sort_by(|a, b| match (a, b) {
                        (Ok(a), Ok(b)) => a.path.to_str().cmp(&b.path.to_str()),
                        (Ok(_), Err(_)) => Ordering::Less,
                        (Err(_), Ok(_)) => Ordering::Greater,
                        (Err(_), Err(_)) => Ordering::Equal,
                    });
                    self.queue.extend(child_entities);
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}

fn dir_entry_to_fs_entity(
    dir_entry_res: Result<DirEntry, std::io::Error>,
    depth: u16,
) -> Result<FsEntity, Error> {
    let (maybe_dir_entry, metadata_res) = match dir_entry_res {
        Ok(ref dir_entry) => (Some(dir_entry), dir_entry.metadata()),
        Err(e) => (None, Err(e)),
    };

    metadata_res
        // If metadata is an Ok(), we know we have the entry too
        .map(|md| FsEntity {
            path: maybe_dir_entry.unwrap().path(),
            metadata: md,
            depth,
        })
        .map_err(|_e| Error {})
}

impl Iterator for LevelOrderTraversal {
    type Item = Result<FsEntity, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_current_res = self.queue.pop_front();
        if let Some(ref current_res) = maybe_current_res {
            if let Ok(current) = current_res {
                self.enque_children_of(current);
            }
        }
        maybe_current_res
    }
}

pub fn walk_dir_in_level_order<P: AsRef<Path>>(
    root_path: P,
) -> Result<LevelOrderTraversal, std::io::Error> {
    let root_path = root_path.as_ref().canonicalize()?;
    let root = fs::metadata(&root_path)
        .map(|md| FsEntity {
            path: root_path.clone(),
            metadata: md,
            depth: 0
        })
        .map_err(|_| Error {});

    Ok(LevelOrderTraversal {
        root_path,
        queue: VecDeque::from([root]),
    })
}

#[derive(Clone, Debug)]
pub struct FsEntity {
    pub path: PathBuf,
    pub metadata: fs::Metadata,
    pub depth: u16,
}

#[allow(dead_code)]
impl FsEntity {
    pub fn size(&self) -> u64 {
        self.metadata.len()
    }

    pub fn is_dir(&self) -> bool {
        self.metadata.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.metadata.is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.metadata.is_symlink()
    }
}

#[derive(Debug)]
pub struct Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
