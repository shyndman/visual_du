use crate::{
    du_plugin::{FsAggregateSize, FsEntityComponent, FsEntityKey, FsRootComponent},
    WindowSize,
};
use bevy::{prelude::*, sprite::Anchor};
use grouping_by::GroupingBy;

const ROOT_COLOR: Color = Color::rgb(0.097, 0.104, 0.120);
const FILE_COLOR: Color = Color::rgb(0.502, 0.502, 0.502);
const TRANSPARENT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.0);
const _SMALL_SLICE_COLOR: Color = Color::rgb(0.231, 0.240, 0.263);
const LAYER_HEIGHT: f32 = 20.0;
const GAP_WIDTH: f32 = 2.0;

const MIN_LIGHTNESS: f32 = 0.62;
const MAX_LIGHTNESS: f32 = 0.9;

#[derive(Component, Copy, Clone, Debug)]
struct DescendentColorRange {
    start: f32,
    end: f32,
}

impl DescendentColorRange {
    fn len(&self) -> f32 {
        self.end - self.start
    }

    fn sub_range(&self, fraction_start: f32, fraction_len: f32) -> DescendentColorRange {
        let start = self.start + fraction_start * self.len();
        DescendentColorRange {
            start,
            end: start + fraction_len * self.len(),
        }
    }

    fn get_color(&self, fraction_start: f32, depth: u16) -> Color {
        let lightness_fraction = ((depth - 1) as f32).clamp(0.0, 5.0) / 5.0;
        let lightness = MIN_LIGHTNESS + lightness_fraction * (MAX_LIGHTNESS - MIN_LIGHTNESS);
        Color::hsl(self.start + fraction_start * self.len(), 1.0, lightness)
    }
}

impl Default for DescendentColorRange {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 360.0,
        }
    }
}

pub struct FsGraphPlugin;

impl Plugin for FsGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, initialize_root_fs_entity_sprite)
            .add_system(scale_root_fs_to_window.after(initialize_fs_entity_sprites))
            .add_system_to_stage(CoreStage::PreUpdate, initialize_fs_entity_sprites)
            .add_system(position_children_by_size.after(initialize_fs_entity_sprites));
    }
}

/// The root is an invisible sprite with the height of a layer, and the width of the canvas
fn initialize_root_fs_entity_sprite(
    mut commands: Commands,
    root_fs_entity: Query<
        (Entity, &FsEntityKey),
        (With<FsEntityComponent>, Added<FsRootComponent>),
    >,
    window_size: Res<WindowSize>,
) {
    if let Ok((entity, fs_key)) = root_fs_entity.get_single() {
        eprintln!("{}: creating sprite", fs_key);
        commands
            .entity(entity)
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: ROOT_COLOR,
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform {
                    translation: (-window_size.0 / 2.0).extend(0.0),
                    scale: Vec3::new(window_size.x, LAYER_HEIGHT, 1.0),
                    ..default()
                },
                visibility: Visibility { is_visible: true },
                ..default()
            })
            .insert(DescendentColorRange {
                start: 0.0,
                end: 360.0,
            });
    }
}

fn scale_root_fs_to_window(
    mut root_fs_transform_query: Query<&mut Transform, With<FsRootComponent>>,
    window_size: Res<WindowSize>,
) {
    if !root_fs_transform_query.is_empty() && window_size.is_changed() {
        let mut transform = root_fs_transform_query.single_mut();
        let size_vec = window_size.0;
        transform.translation = (size_vec / -2.0).extend(0.0);
        transform.scale.x = size_vec.x;
    }
}

fn initialize_fs_entity_sprites(
    mut commands: Commands,
    new_parented_fs_entities_query: Query<
        (Entity, &FsEntityKey, &FsEntityComponent),
        (With<FsEntityComponent>, Added<Parent>),
    >,
) {
    for (entity, fs_key, fs_entity) in new_parented_fs_entities_query.iter() {
        eprintln!("{}: creating sprite", fs_key);
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: if !fs_entity.is_dir() {
                    FILE_COLOR
                } else {
                    TRANSPARENT_COLOR
                },
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 1.0 + 2.0 / LAYER_HEIGHT, 0.0),
                ..default()
            },
            visibility: Visibility { is_visible: false },
            ..default()
        });
        if fs_entity.is_dir() {
            entity_commands.insert(DescendentColorRange::default());
        }
    }
}

fn position_children_by_size(
    changed_fs_entity_parents_query: Query<
        (Entity, &FsEntityKey, &FsAggregateSize, &Children),
        (
            With<FsEntityComponent>,
            With<Children>,
            Or<(Changed<Children>, Changed<FsAggregateSize>)>,
        ),
    >,
    fs_entity_query: Query<(&FsEntityKey, &FsEntityComponent, &FsAggregateSize)>,
    mut fs_entity_style_query: Query<
        (
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
            Option<&mut DescendentColorRange>,
        ),
        With<FsEntityComponent>,
    >,
    root_fs_entity_query: Query<Entity, With<FsRootComponent>>,
) {
    if root_fs_entity_query.is_empty() {
        return;
    }

    let root_entity = root_fs_entity_query.single();
    let root_fs_size = fs_entity_query
        .get_component::<FsAggregateSize>(root_entity)
        .unwrap();
    // This value is in screen coordinates
    let root_scale = fs_entity_style_query
        .get_component::<Transform>(root_entity)
        .unwrap()
        .scale
        .clone();

    for (parent, parent_key, parent_fs_size, children) in changed_fs_entity_parents_query.iter() {
        eprintln!("{}: positioning children", parent_key);

        let parent_color_range = *(fs_entity_style_query
            .get_component::<DescendentColorRange>(parent)
            .unwrap());

        let x_screen_to_world = |x: f32| {
            let fs_fraction =
                parent_fs_size.size_in_bytes as f32 / root_fs_size.size_in_bytes as f32;
            let parent_screen_w = fs_fraction * root_scale.x;
            let fraction_of_parent = x / parent_screen_w;
            eprintln!("  fraction_of_parent={}", fraction_of_parent);

            fraction_of_parent
        };

        let children_by_visibility = children.iter().grouping_by(|child| {
            fs_entity_query
                .get_component::<FsAggregateSize>(**child)
                .unwrap()
                .size_in_bytes
                > 0
        });

        // Hide small children
        let mut combined_invisible_children_size: u64 = 0;
        let default = vec![];
        for child in children_by_visibility.get(&false).unwrap_or(&default) {
            let child = *child;
            let mut child_vis = fs_entity_style_query
                .get_component_mut::<Visibility>(*child)
                .unwrap();
            child_vis.is_visible = false;

            let child_size = fs_entity_query
                .get_component::<FsAggregateSize>(*child)
                .unwrap();
            combined_invisible_children_size += child_size.size_in_bytes;
        }
        let _combined_invisible_children_size_fraction =
            combined_invisible_children_size as f32 / parent_fs_size.size_in_bytes as f32;

        // Show and position larger children
        let visible_children: Vec<&&Entity> = children_by_visibility
            .get(&true)
            .unwrap_or(&default)
            .iter()
            .collect();
        if visible_children.is_empty() {
            continue;
        }

        let number_of_gaps = visible_children.len() - 1;
        let gap_width = x_screen_to_world(GAP_WIDTH);
        let total_gap_width = number_of_gaps as f32 * gap_width;

        eprintln!(
            "  number of gaps={}\n  gap_width={}\n  total_gap_width={}",
            number_of_gaps, gap_width, total_gap_width
        );

        let mut child_fraction_start = 0.0;
        let mut child_translate_x = 0.0;
        for child in visible_children {
            let child = *child;

            let child_entity_res = fs_entity_query.get(*child);
            let child_visuals_res = fs_entity_style_query.get_mut(*child);

            // eprintln!("  {}: {:?}", i, child_visuals_res);
            if let (
                Ok((child_key, child_fs_entity, child_size)),
                Ok((mut child_sprite, mut child_transform, mut child_vis, child_hue_range)),
            ) = (child_entity_res, child_visuals_res)
            {
                child_vis.is_visible = true;

                let fraction_of_parent =
                    child_size.size_in_bytes as f32 / parent_fs_size.size_in_bytes as f32;

                eprintln!("  {} is {}%", child_key, fraction_of_parent * 100.0);
                let display_scale = fraction_of_parent - total_gap_width * fraction_of_parent;
                child_transform.translation.x = child_translate_x;
                child_transform.scale.x = display_scale;

                if child_fs_entity.is_dir() {
                    let new_child_color_range =
                        parent_color_range.sub_range(child_fraction_start, fraction_of_parent);
                    eprintln!("  range={:?}", new_child_color_range);

                    let mut child_hue_range = child_hue_range.unwrap();
                    *child_hue_range = new_child_color_range;

                    let new_sprite_color =
                        new_child_color_range.get_color(0.0, child_fs_entity.depth);
                    child_sprite.color = new_sprite_color;
                    eprintln!("  dir_color={:?}", new_sprite_color);
                }

                eprintln!("  {:?}", child_transform);
                child_translate_x += child_transform.scale.x + gap_width;
                child_fraction_start += fraction_of_parent;
            }
        }
    }
}
