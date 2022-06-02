use crate::debug::DebugPlugin;
use bevy::{
    log::{LogPlugin, LogSettings},
    math::const_vec2,
    pbr::PbrPlugin,
    prelude::*,
    winit::WinitSettings,
};
use tracing::Level;
// Include the embedded assets when we're in release mode
#[cfg(not(debug_assertions))]
include!(concat!(env!("OUT_DIR"), "/add_embasset_assets.rs"));

const WINDOW_COLOR: Color = Color::rgb(0.161, 0.173, 0.2);
const INITIAL_WINDOW_SIZE: Vec2 = const_vec2!([1280.0, 800.0]);

#[derive(Deref, DerefMut)]
pub struct WindowSize(pub Vec2);

pub struct AppScaffoldPlugin {
    pub title: &'static str,
    pub bin_module_path: &'static str,
}
impl Plugin for AppScaffoldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinitSettings::game())
            .insert_resource(WindowDescriptor {
                title: self.title.into(),
                width: INITIAL_WINDOW_SIZE.x,
                height: INITIAL_WINDOW_SIZE.y,
                ..default()
            })
            .insert_resource(WindowSize(INITIAL_WINDOW_SIZE))
            .insert_resource(ClearColor(WINDOW_COLOR))
            .insert_resource(LogSettings {
                filter: "warn".into(),
                level: Level::WARN,
            });

        // We insert these plugins/resources before adding DefaultPlugins, if we're in
        // a debug build
        #[cfg(debug_assertions)]
        {
            app.insert_resource(LogSettings {
                filter: format!("{}=trace,visual_du=info", self.bin_module_path),
                level: Level::WARN,
            });
        }

        app.add_plugin(DebugPlugin)
            .add_plugins_with(DefaultPlugins, |group| {
                // If we're in release mode, ensure we hijack the asset plugin so that
                // we load assets embedded in the executable (using Embasset, check out
                // build.dart), instead of the file system (it's not there!).
                #[cfg(not(debug_assertions))]
                let group = {
                    use bevy::asset::AssetPlugin;
                    use bevy_embasset::EmbassetPlugin;

                    group.add_before::<AssetPlugin, _>(EmbassetPlugin::new(
                        add_embasset_assets,
                    ))
                };

                // We disable the log plugin so we can setup the tracing subscriber the
                // way we like it...
                group
                    .disable::<LogPlugin>()
                    // ...and we also disable PBR because it's expensive even if it's
                    // doing nothing (big data movement to the GPU, involving lights???).
                    .disable::<PbrPlugin>()
            })
            .add_startup_system(create_ui_camera)
            .add_system(update_window_size);
    }
}

fn create_ui_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}


fn update_window_size(windows: Res<Windows>, mut window_size: ResMut<WindowSize>) {
    if let Some(window) = windows.get_primary() {
        let new_window_size = Vec2::new(window.width(), window.height());
        if window_size.0 != new_window_size {
            info!(
                width = window.width(),
                height = window.height(),
                "window has new size"
            );
            window_size.0 = new_window_size;
        }
    }
}
