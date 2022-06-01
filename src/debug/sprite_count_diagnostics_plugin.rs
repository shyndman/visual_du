use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    ecs::system::SystemParam,
    prelude::*,
    sprite::Sprite,
};

pub const SPRITE_COUNT: DiagnosticId =
    DiagnosticId::from_u128(264837939815642801051434963579989783448);

pub struct SpriteCountDiagnosticsPlugin;
impl Plugin for SpriteCountDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_system)
            .add_system(diagnostic_system);
    }
}

fn setup_system(mut diagnostics: ResMut<Diagnostics>) {
    diagnostics.add(Diagnostic::new(SPRITE_COUNT, "sprite_count", /* max_history_length` */ 20));
}

fn diagnostic_system(counter: SpriteCounter, mut diagnostics: ResMut<Diagnostics>) {
    let sprite_count = counter.count();
    diagnostics.add_measurement(SPRITE_COUNT, sprite_count as f64);
}

/// The [`SystemParam`] struct can contain any types that can also be included in a
/// system function signature.
#[derive(SystemParam)]
pub struct SpriteCounter<'w, 's> {
    sprites: Query<'w, 's, &'static Sprite>,
}
impl<'w, 's> SpriteCounter<'w, 's> {
    pub fn count(&self) -> usize {
        self.sprites.iter().len()
    }
}
