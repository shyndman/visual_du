mod debug;
mod fs;
mod ui;

use crate::{
    debug::DebugPlugin,
    fs::du_plugin::*,
    ui::{mouse_interactions_plugin::*, tree_view_plugin::DiskUsageTreeViewPlugin},
};
#[cfg(debug_assertions)]
use bevy::log::LogSettings;
use bevy::{log::LogPlugin, math::const_vec2, prelude::*, winit::WinitSettings};
use bevy_framepace::{FramepacePlugin, FramerateLimit};
use std::env;
#[cfg(debug_assertions)]
use tracing::Level;

const WINDOW_COLOR: Color = Color::rgb(0.161, 0.173, 0.2);
const INITIAL_WINDOW_WIDTH: f32 = 1280.0;
const INITIAL_WINDOW_HEIGHT: f32 = 800.0;
const INITIAL_WINDOW_SIZE: Vec2 =
    const_vec2!([INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT]);

#[derive(Deref, DerefMut)]
struct WindowSize(pub Vec2);

fn main() {
    let mut args = env::args().skip(1);
    let root_path: String = args.next().unwrap_or("/Users/shyndman/dev/dart-sdk".into());
    let mut application = App::new();
    application
        .insert_resource(DiskUsageRootPath::from(root_path))
        .insert_resource(WinitSettings::game())
        .insert_resource(WindowDescriptor {
            title: "Visual Disk Usage".into(),
            width: INITIAL_WINDOW_SIZE.x,
            height: INITIAL_WINDOW_SIZE.y,
            ..default()
        })
        .insert_resource(WindowSize(INITIAL_WINDOW_SIZE))
        .insert_resource(ClearColor(WINDOW_COLOR));

    // We insert these plugins/resources before adding DefaultPlugins
    #[cfg(debug_assertions)]
    {
        application.insert_resource(LogSettings {
            filter: "main=trace".to_string(),
            level: Level::WARN,
        });
    }

    application
        .add_plugin(DebugPlugin)
        .add_plugins_with(
            DefaultPlugins,
            // We disable the log plugin so we can setup the tracing subscriber the way we like it
            |group| group.disable::<LogPlugin>(),
        )
        .add_plugin(MouseInteractionsPlugin)
        .add_plugin(FramepacePlugin {
            framerate_limit: FramerateLimit::Manual(30),
            warn_on_frame_drop: false,
        })
        .add_startup_system(setup_cameras)
        .add_plugin(DiskUsagePlugin)
        .add_plugin(DiskUsageTreeViewPlugin)
        .add_system(update_window_size);

    application.run();
}

fn setup_cameras(mut commands: Commands) {
    // Cameras
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(InputCamera);
    commands.spawn_bundle(UiCameraBundle::default());
}

fn update_window_size(windows: Res<Windows>, mut window_size: ResMut<WindowSize>) {
    if let Some(window) = windows.get_primary() {
        let new_window_size = Vec2::new(window.width(), window.height());
        if window_size.0 != new_window_size {
            window_size.0 = new_window_size;
        }
    }
}
