use super::{walk_dir, FsEntity};
use bevy::prelude::*;
use crossbeam_channel::bounded;
use std::{fs, thread};
use tracing::debug;
use valuable::{Valuable, Value};

#[macro_export]
macro_rules! relative_to {
    ($path:expr, $root_path:expr) => {
        $path
            .strip_prefix(<std::path::PathBuf as AsRef<std::path::Path>>::as_ref(
                &($root_path),
            ))
            .unwrap()
    };
}

#[derive(Component)]
pub struct FsRootComponent;

#[derive(Component, Deref)]
pub struct FsEntityKey(pub String);

impl Valuable for FsEntityKey {
    fn as_value(&self) -> valuable::Value<'_> {
        Value::String(self.0.as_str())
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        visit.visit_value(self.as_value());
    }
}

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

pub struct DiskUsageWalkConfig {
    pub root_path: std::path::PathBuf,
}

impl DiskUsageWalkConfig {
    pub fn new(path: String) -> Self {
        Self {
            root_path: fs::canonicalize(path).unwrap(),
        }
    }
}

impl Default for DiskUsageWalkConfig {
    fn default() -> Self {
        Self {
            root_path: fs::canonicalize(".").unwrap(),
        }
    }
}

pub struct DiskUsagePlugin;
impl Plugin for DiskUsagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiskUsageWalkConfig>()
            .add_startup_system(start_dir_walk)
            .add_system(spawn_fs_entities)
            .add_system(establish_parentage)
            .add_system(increment_ancestor_sizes_on_add);
    }
}

fn start_dir_walk(mut commands: Commands, config: Res<DiskUsageWalkConfig>) {
    let root_path = &config.root_path;
    info!(root_path = root_path.as_value(), "starting directory walk");

    let (send_channel, receive_channel) = bounded::<FsEntity>(64);
    let root_path_for_move = root_path.clone(); // We need a scope-appropriate lifetime
    thread::spawn(move || {
        for entity_res in walk_dir(root_path_for_move).unwrap() {
            let entity = entity_res.unwrap();
            match send_channel.send(entity) {
                Ok(_) => {}
                Err(e) => error!(error = %e, "Error encountered while sending"),
            }
        }
    });

    commands.insert_resource(FsStreamReceiver(receive_channel));
    commands.insert_resource(FsEntityMap(bevy::utils::HashMap::new()));
}

fn spawn_fs_entities(
    mut commands: Commands,
    mut fs_entity_map: ResMut<FsEntityMap>,
    fs_entity_stream: ResMut<FsStreamReceiver>,
    config: Res<DiskUsageWalkConfig>,
) {
    for fs_entity in fs_entity_stream.try_iter() {
        let rel_path = relative_to!(fs_entity.path, config.root_path);
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
    config: Res<DiskUsageWalkConfig>,
) {
    for (child_entity, fs_key, fs_entity) in added_fs_entities.iter() {
        let path = &fs_entity.path;
        let rel_path = relative_to!(path, config.root_path);
        debug!(path = fs_key.as_value(), "establishing parentage");
        if let Some(parent_path) = rel_path.parent() {
            debug!(
                path = fs_key.as_value(),
                parent_path = parent_path.as_value(),
                "linking to parent",
            );
            let parent_key: String = parent_path.to_string_lossy().into();
            let parent_entity = fs_entity_map.get(&parent_key).unwrap();
            commands.entity(*parent_entity).add_child(child_entity);
        } else {
            debug!(
                path = fs_key.as_value(),
                "adding FsRootEntityComponent marker to root fs entity"
            );
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
    config: Res<DiskUsageWalkConfig>,
) {
    for (fs_key, fs_entity) in added_fs_entities.iter() {
        let rel_path = relative_to!(fs_entity.path, config.root_path);
        let key: String = rel_path.to_string_lossy().into();
        let size_in_bytes = all_sizes
            .get(*fs_entity_map.get(&key).unwrap())
            .unwrap()
            .size_in_bytes;

        if size_in_bytes == 0 {
            debug!(
                path = fs_entity.path.as_value(),
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
                    new_size = ancestor_agg_size.size_in_bytes,
                );
            } else {
                error!("(error!!!)");
            }
        }
    }
}
