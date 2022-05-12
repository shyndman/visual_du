mod du_plugin;
mod fs_graph_plugin;
mod input_plugin;
pub mod walk_dir_level_order;

use crate::du_plugin::*;
use crate::fs_graph_plugin::FsGraphPlugin;
use crate::input_plugin::*;
use bevy::winit::WinitSettings;
use bevy::{math::const_vec2, prelude::*};
use bevy_framepace::{FramepacePlugin, FramerateLimit};

const WINDOW_COLOR: Color = Color::rgb(0.161, 0.173, 0.2);
const INITIAL_WINDOW_WIDTH: f32 = 1280.0;
const INITIAL_WINDOW_HEIGHT: f32 = 800.0;
const INITIAL_WINDOW_SIZE: Vec2 = const_vec2!([INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT]);

#[derive(Deref, DerefMut)]
struct WindowSize(Vec2);

fn main() {
    App::new()
        .insert_resource(DiskUsageRootPath::from("example"))
        .insert_resource(WinitSettings::game())
        .insert_resource(WindowDescriptor {
            title: "Visual Disk Usage".into(),
            width: INITIAL_WINDOW_SIZE.x,
            height: INITIAL_WINDOW_SIZE.y,
            ..default()
        })
        .insert_resource(WindowSize(INITIAL_WINDOW_SIZE))
        .insert_resource(ClearColor(WINDOW_COLOR))
        .add_plugins(DefaultPlugins)
        .add_plugin(MouseInputPlugin)
        .add_plugin(FramepacePlugin {
            framerate_limit: FramerateLimit::Manual(30),
            warn_on_frame_drop: false,
        })
        .add_startup_system(setup)
        .add_plugin(DiskUsagePlugin)
        .add_plugin(FsGraphPlugin)
        .add_system(update_window_size)
        // .add_system(transform_on_key)
        // .add_system(scale_root_on_window_resize)
        .run();
}

fn setup(mut commands: Commands) {
    // Cameras
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(InputCamera);
}

fn update_window_size(windows: Res<Windows>, mut window_size: ResMut<WindowSize>) {
    if let Some(window) = windows.get_primary() {
        let new_window_size = Vec2::new(window.width(), window.height());
        if window_size.0 != new_window_size {
            window_size.0 = new_window_size;
        }
    }
}
