use super::{SpriteCountDiagnosticsPlugin, SPRITE_COUNT};
use crate::debug::PrettierFormatter;
use bevy::{
    diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    log::LogSettings,
    prelude::{Plugin, *},
};
use std::panic;
use tracing_subscriber::{
    fmt::format::FmtSpan, prelude::*, registry::Registry, EnvFilter,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let old_handler = panic::take_hook();
        panic::set_hook(Box::new(move |infos| {
            println!("{}", tracing_error::SpanTrace::capture());
            old_handler(infos);
        }));

        app
            // Tracing
            .add_startup_system_to_stage(StartupStage::PreStartup, setup_tracing)
            // Diagnostics collection
            .add_plugin(SpriteCountDiagnosticsPlugin)
            .add_plugin(EntityCountDiagnosticsPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            // Diagnostics display
            .add_startup_system(setup_diagnostics_ui)
            .add_system(display_diagnostics);
    }
}

fn setup_tracing(maybe_settings: Option<Res<LogSettings>>) {
    let default_filter = if let Some(settings) = maybe_settings {
        let level_str = settings.level.to_string();
        format!("{},{}", level_str.to_lowercase(), settings.filter)
    } else {
        "warn".to_string()
    };
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_filter))
        .unwrap();
    let filter_str = filter_layer.to_string();

    tracing::subscriber::set_global_default(
        Registry::default()
            .with(filter_layer)
            .with(tracing_error::ErrorLayer::default())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_span_events(FmtSpan::CLOSE)
                    .event_format(PrettierFormatter::default()),
            ),
    )
    .unwrap();

    info!(filter = filter_str.as_value(), "Tracing configured");
}

#[derive(Component)]
struct DiagnosticsText;

fn setup_diagnostics_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // NOTE: This might not successfully load...but it's not entirely clear how to
    // properly handle the failure.
    let font = asset_server.load("fonts/Hack-Regular.ttf");

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(16.0),
                    left: Val::Px(16.0),
                    ..default()
                },
                ..default()
            },
            text: Text::with_section(
                "",
                TextStyle {
                    font: font.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
                TextAlignment::default(),
            ),
            ..default()
        })
        .insert(DiagnosticsText);

    commands.spawn_bundle(UiCameraBundle::default());
}

fn display_diagnostics(
    mut text: Query<&mut Text, With<DiagnosticsText>>,
    diagnostics: Res<Diagnostics>,
) {
    if text.is_empty() {
        return;
    }

    let mut display_text = String::from("");

    if let Some(fps_diag) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average_fps) = fps_diag.average() {
            display_text.push_str(format!("{:.2} fps\n", average_fps).as_str());
        }
    }

    if let Some(entity_count_diag) =
        diagnostics.get(EntityCountDiagnosticsPlugin::ENTITY_COUNT)
    {
        if let Some(entity_count) = entity_count_diag.value() {
            display_text.push_str(format!("{} entities\n", entity_count).as_str());
        }
    }

    if let Some(sprite_count_diag) = diagnostics.get(SPRITE_COUNT) {
        if let Some(sprite_count) = sprite_count_diag.value() {
            display_text.push_str(format!("{} sprites\n", sprite_count).as_str());
        }
    }

    let mut text = text.single_mut();
    text.sections[0].value = display_text;
}
