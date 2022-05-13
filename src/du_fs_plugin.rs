use crate::walk_dir_level_order::{walk_dir, FsEntity};
use bevy::prelude::*;
use crossbeam_channel::bounded;
use std::{fs, thread};
use tracing::debug;
use valuable::Valuable as ValuableTrait;
use valuable_derive::Valuable;

#[macro_export]
macro_rules! relative_to {
    ($path:expr, $root_path:ident) => {
        $path
            .strip_prefix(<std::path::PathBuf as AsRef<std::path::Path>>::as_ref(
                &($root_path),
            ))
            .unwrap()
    };
}

#[derive(Component)]
pub struct FsRootComponent;

#[derive(Component, Deref, Valuable)]
pub struct FsEntityKey(pub String);

#[derive(Component, Debug, Deref)]
pub struct FsEntityComponent(FsEntity);

#[derive(Component, Debug, Default)]
pub struct FsAggregateSize {
    pub size_in_bytes: u64,
}

#[derive(Deref)]
struct FsStreamReceiver(crossbeam_channel::Receiver<FsEntity>);

#[derive(Deref, DerefMut)]
struct FsEntityMap(bevy::utils::HashMap<String, Entity>);

#[derive(Deref)]
pub struct DiskUsageRootPath(std::path::PathBuf);

impl From<String> for DiskUsageRootPath {
    fn from(path: String) -> Self {
        Self(fs::canonicalize(path).unwrap())
    }
}

impl From<&str> for DiskUsageRootPath {
    fn from(path: &str) -> Self {
        Self(fs::canonicalize(path).unwrap())
    }
}

impl Default for DiskUsageRootPath {
    fn default() -> Self {
        ".".into()
    }
}

pub struct DiskUsagePlugin;
impl Plugin for DiskUsagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiskUsageRootPath>()
            .add_startup_system(start_dir_walk)
            .add_system(spawn_fs_entities)
            .add_system(establish_parentage)
            .add_system(increment_ancestor_sizes_on_add);
    }
}

fn start_dir_walk(mut commands: Commands, root_path: Res<DiskUsageRootPath>) {
    info!(root_path = root_path.as_value(), "starting directory walk");

    let (send_channel, receive_channel) = bounded::<FsEntity>(64);
    let root_path_for_move = root_path.clone(); // We need a scope-appropriate lifetime
    thread::spawn(move || {
        for entity_res in walk_dir(root_path_for_move).unwrap() {
            let entity = entity_res.unwrap();
            send_channel.send(entity).unwrap();
        }
    });

    commands.insert_resource(FsStreamReceiver(receive_channel));
    commands.insert_resource(FsEntityMap(bevy::utils::HashMap::new()));
}

fn spawn_fs_entities(
    mut commands: Commands,
    mut fs_entity_map: ResMut<FsEntityMap>,
    fs_entity_stream: ResMut<FsStreamReceiver>,
    root_path: Res<DiskUsageRootPath>,
) {
    for fs_entity in fs_entity_stream.try_iter() {
        let rel_path = relative_to!(fs_entity.path, root_path);
        let key: String = rel_path.to_string_lossy().into();
        debug!(path = rel_path.as_value(), "spawning entity");

        fs_entity_map.insert(
            key.clone(),
            commands
                .spawn()
                .insert(FsAggregateSize {
                    size_in_bytes: fs_entity.size_in_bytes(),
                })
                .insert(FsEntityKey(key))
                .insert(FsEntityComponent(fs_entity))
                .id(),
        );
    }
}

/// Establishes the parentage of fs entities in the data layer
fn establish_parentage(
    mut commands: Commands,
    added_fs_entities: Query<
        (Entity, &FsEntityKey, &FsEntityComponent),
        Added<FsEntityComponent>,
    >,
    fs_entity_map: Res<FsEntityMap>,
    root_path: Res<DiskUsageRootPath>,
) {
    for (child_entity, fs_key, fs_entity) in added_fs_entities.iter() {
        let path = &fs_entity.path;
        let rel_path = relative_to!(path, root_path);
        debug!(path = fs_key.as_value(), "establishing parentage");
        if let Some(parent_path) = rel_path.parent() {
            debug!(parent_path = parent_path.as_value(), "  linking to parent",);
            let parent_key: String = parent_path.to_string_lossy().into();
            let parent_entity = fs_entity_map.get(&parent_key).unwrap();
            commands.entity(*parent_entity).add_child(child_entity);
        } else {
            debug!("  is root — adding FsRootEntityComponent marker");
            commands.entity(child_entity).insert(FsRootComponent {});
        }
    }
}

fn increment_ancestor_sizes_on_add(
    added_fs_entities: Query<
        (&FsEntityKey, &FsEntityComponent),
        Added<FsEntityComponent>,
    >,
    mut all_sizes: Query<&mut FsAggregateSize>,
    fs_entity_map: Res<FsEntityMap>,
    root_path: Res<DiskUsageRootPath>,
) {
    for (fs_key, fs_entity) in added_fs_entities.iter() {
        let rel_path = relative_to!(fs_entity.path, root_path);
        let key: String = rel_path.to_string_lossy().into();
        let size_in_bytes = all_sizes
            .get(*fs_entity_map.get(&key).unwrap())
            .unwrap()
            .size_in_bytes;

        if size_in_bytes == 0 {
            debug!(
                path = fs_key.as_value(),
                "increasing ancestor sizes...skip (0 size)",
            );
            continue;
        } else {
            debug!(
                path = fs_key.as_value(),
                "increasing ancestor sizes +{size_in_bytes}b",
            );
        }

        // Loop over the path's ancestors, increasing the size of each by the size of the entity
        // we're processing.
        //
        // NOTE: The skip(1) is to skip the entity itself
        let ancestor_paths = rel_path.ancestors().skip(1);
        for ancestor_path in ancestor_paths {
            let entity_key: String = ancestor_path.to_string_lossy().into();
            let ancestor_entity: &Entity = fs_entity_map.get(&entity_key).unwrap();

            if let Ok(mut ancestor_agg_size) = all_sizes.get_mut(*ancestor_entity) {
                ancestor_agg_size.size_in_bytes += size_in_bytes;
                debug!(
                    path = ancestor_path.as_value(),
                    "  {new_size}b",
                    new_size = ancestor_agg_size.size_in_bytes,
                );
            } else {
                error!(" (error!!!)");
            }
        }
    }
}
