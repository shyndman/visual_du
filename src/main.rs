mod fs;
mod trace;
mod ui;

#[cfg(debug_assertions)]
use ::tracing::Level;
use bevy::log::{LogPlugin, LogSettings};
use bevy::winit::WinitSettings;
use bevy::{math::const_vec2, prelude::*};
use bevy_framepace::{FramepacePlugin, FramerateLimit};
use fs::du_plugin::*;
use std::env;
use trace::fmt::PrettierFormatter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};
use ui::{mouse_interactions_plugin::*, tree_view_plugin::DiskUsageTreeViewPlugin};

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
            scale_factor_override: Some(2.0),
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowSize(INITIAL_WINDOW_SIZE))
        .insert_resource(ClearColor(WINDOW_COLOR));

    // We insert these plugins/resources before adding DefaultPlugins
    #[cfg(debug_assertions)]
    {
        application.insert_resource(LogSettings {
            filter: "main=trace".to_string(),
            level: Level::DEBUG,
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

fn setup_tracing(maybe_settings: Option<Res<LogSettings>>) {
    let default_filter = if let Some(settings) = maybe_settings {
        format!("{},{}", settings.level, settings.filter)
    } else {
        "".to_string()
    };
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_filter))
        .unwrap();

    let subscriber = Registry::default()
        .with(filter_layer)
        .with(tracing_error::ErrorLayer::default())
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .event_format(PrettierFormatter::default()),
        );

    ::tracing::subscriber::set_global_default(subscriber).unwrap();
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
