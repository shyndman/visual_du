mod du_fs_plugin;
mod du_tree_view_plugin;
mod mouse_interactions_plugin;
pub mod walk_dir_level_order;

use crate::du_fs_plugin::*;
use crate::du_tree_view_plugin::DiskUsageTreeViewPlugin;
use crate::mouse_interactions_plugin::*;
use bevy::log::{LogPlugin, LogSettings};
use bevy::winit::WinitSettings;
use bevy::{math::const_vec2, prelude::*};
use bevy_framepace::{FramepacePlugin, FramerateLimit};
use tracing::Level;
use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};

const WINDOW_COLOR: Color = Color::rgb(0.161, 0.173, 0.2);
const INITIAL_WINDOW_WIDTH: f32 = 1280.0;
const INITIAL_WINDOW_HEIGHT: f32 = 800.0;
const INITIAL_WINDOW_SIZE: Vec2 = const_vec2!([INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT]);

#[derive(Deref, DerefMut)]
struct WindowSize(pub Vec2);

fn main() {
    let mut application = App::new();
    application
        .insert_resource(DiskUsageRootPath::from("example/example_hierarchy/"))
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(WindowDescriptor {
            title: "Visual Disk Usage".into(),
            width: INITIAL_WINDOW_SIZE.x,
            height: INITIAL_WINDOW_SIZE.y,
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowSize(INITIAL_WINDOW_SIZE))
        .insert_resource(ClearColor(WINDOW_COLOR));

    // We insert these plugins/resources before adding DefaultPlugins
    #[cfg(debug_assertions)]
    {
        application.insert_resource(LogSettings {
            filter: "main=debug".to_string(),
            level: Level::WARN,
        });
    }

    application
        .add_startup_system(setup_tracing)
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
        .add_startup_system(setup)
        .add_plugin(DiskUsagePlugin)
        .add_plugin(DiskUsageTreeViewPlugin)
        .add_system(update_window_size);

    application.run();
}

fn setup_tracing(settings: Res<LogSettings>) {
    let default_filter = { format!("{},{}", settings.level, settings.filter) };
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_filter))
        .unwrap();
    let subscriber = Registry::default()
        .with(filter_layer)
        .with(tracing_error::ErrorLayer::default())
        .with(tracing_subscriber::fmt::layer().compact().without_time());

    tracing::subscriber::set_global_default(subscriber).unwrap();
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
