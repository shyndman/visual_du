use bevy::{prelude::*, render::camera::RenderTarget};
use tracing::debug;
use valuable::Valuable;

#[derive(Component, Default)]
pub struct Hoverable {
    pub is_hovered: bool,
    pub debug_tag: String,
}

#[derive(Deref, DerefMut)]
struct MouseCursorWorldPosition(Option<Vec2>);

/// Marks the camera that should be used when mapping cursor position into world coordinates
#[derive(Component)]
pub struct InputCamera;

pub struct MouseInteractionsPlugin;
impl Plugin for MouseInteractionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseCursorWorldPosition(Vec2::ZERO.into()))
            .add_system_to_stage(CoreStage::PreUpdate, update_cursor_position)
            .add_system(mark_hoverables);
    }
}

fn update_cursor_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<InputCamera>>,
    mut cursor_world_pos: ResMut<MouseCursorWorldPosition>,
    windows: Res<Windows>,
) {
    let (camera, camera_transform) = camera_query.single();
    // Get the window that the camera is displaying to (or the primary window)
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if window.cursor_position() == cursor_world_pos.0 {
        return;
    }

    if let Some(screen_pos) = window.cursor_position() {
        // Get the size of the window
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // Convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // Determine matrix for undoing the projection and camera transform
        let ndc_to_world =
            camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // Use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // Reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        // Cursor is inside the window, position given
        *cursor_world_pos = MouseCursorWorldPosition(Some(world_pos));
    } else {
        // Cursor is not inside the window
        *cursor_world_pos = MouseCursorWorldPosition(None);
    }
}

#[derive(Default, Deref, DerefMut)]
struct LastHovered(Option<Entity>);

fn mark_hoverables(
    cursor_world_pos: Res<MouseCursorWorldPosition>,
    mut hoverables_query: Query<(Entity, &mut Hoverable, &GlobalTransform, &Visibility)>,
    mut last_hovered: Local<LastHovered>,
) {
    if cursor_world_pos.0 == None {
        if cursor_world_pos.is_changed() {
            debug!("no cursor position — removing any existing hover states");
            if let Some(entity) = last_hovered.0 {
                let (_, mut hoverable, _, _) = hoverables_query.get_mut(entity).unwrap();
                hoverable.is_hovered = false;
            }
            last_hovered.0 = None;
        }
    } else {
        let span = trace_span!("finding hoverables under cursor");
        let _enter_guard = span.enter();

        let cursor_world_pos = cursor_world_pos.unwrap();
        let mut z_ordered_hoverables: Vec<(
            Entity,
            Mut<Hoverable>,
            &GlobalTransform,
            &Visibility,
        )> = hoverables_query
            .iter_mut()
            // Don't include hidden sprites
            .filter(|(_, _, _, vis)| vis.is_visible)
            .collect();
        z_ordered_hoverables.sort_by(|(_, _, t_a, _), (_, _, t_b, _)| {
            t_b.translation.z.total_cmp(&t_a.translation.z)
        });

        last_hovered.0 = None;
        for (entity, mut hoverable, transform, _) in z_ordered_hoverables {
            let min = transform.translation.truncate();
            let max = min + transform.scale.truncate();
            let new_is_hovered = min.x <= cursor_world_pos.x
                && min.y <= cursor_world_pos.y
                && cursor_world_pos.x <= max.x
                && cursor_world_pos.y <= max.y;

            if hoverable.is_hovered != new_is_hovered {
                hoverable.is_hovered = new_is_hovered;
                if new_is_hovered {
                    info!(debug_tag = hoverable.debug_tag.as_value(), "new hovered",);
                    last_hovered.0 = Some(entity);
                }
            }
        }
    }
}
