use bevy::prelude::*;
use std::env;
use visual_du::{
    app_scaffold::AppScaffoldPlugin,
    fs::{DiskUsagePlugin, DiskUsageWalkConfig},
    ui::{DiskUsageTreeViewPlugin, InputCamera, MouseInteractionsPlugin},
};

fn main() {
    let mut args = env::args().skip(1);
    let root_path: String = args.next().unwrap_or("./example".into());

    let mut app = App::new();
    app.add_plugin(AppScaffoldPlugin {
        title: "Visual Disk Usage",
        bin_module_path: module_path!(),
    })
    .insert_resource(DiskUsageWalkConfig::new(root_path))
    .add_plugin(MouseInteractionsPlugin)
    .add_plugin(DiskUsagePlugin)
    .add_plugin(DiskUsageTreeViewPlugin)
    .add_startup_system(setup_cameras)
    .run();
}

fn setup_cameras(mut commands: Commands) {
    // Cameras
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(InputCamera);
}
