use super::mouse_interactions_plugin::Hoverable;
use crate::{
    app_scaffold::WindowSize,
    fs::{FsAggregateSize, FsEntityComponent, FsEntityKey, FsRootComponent},
    ui::DescendentColorRange,
};
use bevy::{prelude::*, sprite::Anchor};
use grouping_by::GroupingBy;
use tracing::debug;
use valuable::Valuable;

const ROOT_COLOR: Color = Color::rgba(0.097, 0.104, 0.120, 0.0);
const FILE_COLOR: Color = Color::rgb(0.502, 0.502, 0.502);
const TRANSPARENT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.0);
const _SMALL_SLICE_COLOR: Color = Color::rgb(0.231, 0.240, 0.263);
const LAYER_HEIGHT: f32 = 36.0;
const GAP_WIDTH: f32 = 0.5;

const MIN_CHILD_WIDTH: f32 = 1.0;
const MIN_CHILD_WIDTH_WITH_GAP: f32 = MIN_CHILD_WIDTH + GAP_WIDTH;

pub struct DiskUsageTreeOptions {
    max_depth: u16,
}

impl Default for DiskUsageTreeOptions {
    fn default() -> Self {
        Self { max_depth: 6 }
    }
}

#[derive(Component)]
struct DiskUsageTreeViewTransformRoot;

pub struct DiskUsageTreeViewPlugin;
impl Plugin for DiskUsageTreeViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiskUsageTreeOptions>()
            .add_startup_system(create_transform_root)
            .add_system(scale_transform_root_to_window)
            .add_system_to_stage(CoreStage::PreUpdate, initialize_fs_root_entity_sprite)
            .add_system_to_stage(CoreStage::PreUpdate, initialize_fs_entity_sprites)
            .add_system(handle_hover)
            .add_system(invalidate_tree_from_root)
            .add_system(update_hover_sprite.after(invalidate_tree_from_root));
    }
}

/// Creates a set of transforms that acts as the root of all sprites drawn by this graph
fn create_transform_root(mut commands: Commands, window_size: Res<WindowSize>) {
    let window_size = window_size.0;
    let transform = root_transform_for_window_size(window_size);

    debug!(
        window_size = ?window_size,
        transform.translation = ?transform.translation,
        transform.scale = ?transform.scale,
        "creating transform root"
    );

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.36,
                },
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 2.0),
                scale: Vec3::new(300.0, 300.0, 1.0),
                ..default()
            },
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(HoverSprite(None));

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: TRANSPARENT_COLOR,
                anchor: Anchor::Center,
                ..default()
            },
            transform: transform,
            ..default()
        })
        .insert(DiskUsageTreeViewTransformRoot);
}

/// Keeps the transform root sized with the window
fn scale_transform_root_to_window(
    mut transform_root_query: Query<&mut Transform, With<DiskUsageTreeViewTransformRoot>>,
    window_size: Res<WindowSize>,
) {
    if !transform_root_query.is_empty() && window_size.is_changed() {
        let mut transform = transform_root_query.single_mut();
        let window_size = window_size.0;
        *transform = root_transform_for_window_size(window_size);

        debug!(
            window_size = window_size.to_array().as_value(),
            translation = transform.translation.to_array().as_value(),
            scale = transform.scale.to_array().as_value(),
            "transform root resized to reflect new window size"
        )
    }
}

fn root_transform_for_window_size(window_size: Vec2) -> Transform {
    Transform {
        translation: (window_size / -2.0).extend(0.0)
            + Vec3::new(window_size.x / 10.0, window_size.y / 4.0, 0.0),
        scale: Vec3::new(window_size.x - window_size.x / 5.0, LAYER_HEIGHT, 1.0),
        ..default()
    }
}

/// The root is an invisible sprite with the height of a layer, and the width of the canvas
fn initialize_fs_root_entity_sprite(
    mut commands: Commands,
    fs_root_query: Query<
        (Entity, &FsEntityKey),
        (With<FsEntityComponent>, Added<FsRootComponent>),
    >,
    transform_root_query: Query<
        Entity,
        (With<DiskUsageTreeViewTransformRoot>, Without<Children>),
    >,
) {
    if transform_root_query.is_empty() || fs_root_query.is_empty() {
        return;
    }

    // Create a new sprite for the root fs
    debug!("initializing root sprite");
    let (fs_root, fs_root_key) = fs_root_query.single();
    commands
        .entity(fs_root)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: ROOT_COLOR,
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                translation: Vec3::ZERO,
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        })
        .insert(Hoverable {
            debug_tag: fs_root_key.to_string(),
            ..default()
        })
        .insert(DescendentColorRange::default());

    // Add it to the transform root
    let transform_root = transform_root_query.single();
    commands.entity(transform_root).add_child(fs_root);
}

fn initialize_fs_entity_sprites(
    mut commands: Commands,
    new_parented_fs_entities_query: Query<
        (Entity, &FsEntityKey, &FsEntityComponent),
        (
            With<FsEntityComponent>,
            Without<FsRootComponent>,
            Added<Parent>,
        ),
    >,
    options: Res<DiskUsageTreeOptions>,
) {
    for (entity, fs_key, fs_entity) in new_parented_fs_entities_query.iter() {
        if fs_entity.depth > options.max_depth {
            debug!(
                key = fs_key.as_value(),
                depth = fs_entity.depth,
                max_depth = options.max_depth,
                "path is beyond max depth"
            );
            continue;
        }

        debug!(key = fs_key.as_value(), "creating sprite");
        let mut entity_commands = commands.entity(entity);
        entity_commands
            .insert_bundle(SpriteBundle {
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
                    translation: Vec3::new(0.0, 1.0 + GAP_WIDTH / LAYER_HEIGHT, 0.0),
                    ..default()
                },
                visibility: Visibility { is_visible: false },
                ..default()
            })
            .insert(Hoverable {
                debug_tag: fs_key.to_string(),
                ..default()
            });
        if fs_entity.is_dir() {
            entity_commands.insert(DescendentColorRange::default());
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct HoverSprite(Option<Entity>);

fn handle_hover(
    changed_hoverables_query: Query<
        (
            Entity,
            &Hoverable,
            &GlobalTransform,
            ChangeTrackers<Hoverable>,
        ),
        (
            With<FsEntityComponent>,
            Without<FsRootComponent>,
            Changed<Hoverable>,
        ),
    >,
    mut hover_sprite_query: Query<
        (&mut HoverSprite, &mut Transform, &mut Visibility),
        With<HoverSprite>,
    >,
) {
    let only_changed_hoverables = changed_hoverables_query
        .iter()
        .filter(|(_, _, _, tracker)| tracker.is_changed() && !tracker.is_added());
    let change_count = only_changed_hoverables.count();

    if hover_sprite_query.is_empty() || change_count == 0 {
        return;
    }

    let (mut hover_sprite, mut hover_transform, mut hover_vis) =
        hover_sprite_query.single_mut();
    let mut has_hovered = false;
    for (hoverable_entity, hoverable, transform, change_tracker) in
        changed_hoverables_query.iter()
    {
        info!(
            hoverable_changed = change_tracker.is_changed(),
            debug_tag = hoverable.debug_tag.as_value(),
            hovered = hoverable.is_hovered,
            "handle_hover()"
        );

        if hoverable.is_hovered {
            has_hovered = true;

            info!(
                translation = transform.translation.to_array().as_value(),
                scale = transform.scale.to_array().as_value(),
                "applying transform"
            );

            hover_sprite.0 = Some(hoverable_entity);
            hover_vis.is_visible = true;
            hover_transform.scale = transform.scale;
            hover_transform.translation =
                Vec3::new(transform.translation.x, transform.translation.y, 2.0);
        }
    }

    if !has_hovered {
        warn!("Hiding the hover sprite");
        hover_vis.is_visible = false;
        hover_sprite.0 = None;
    }
}

fn update_hover_sprite(
    mut hover_sprite_query: Query<(&HoverSprite, &mut Transform, &mut Visibility)>,
    hoverable_transforms: Query<
        &GlobalTransform,
        (With<FsEntityComponent>, Without<FsRootComponent>),
    >,
) {
    if hover_sprite_query.is_empty() {
        return;
    }

    let (hover_sprite, mut hover_transform, mut hover_vis) =
        hover_sprite_query.single_mut();
    if hover_sprite.is_none() {
        return;
    }

    let global_transform = hoverable_transforms.get(hover_sprite.0.unwrap()).unwrap();
    hover_vis.is_visible = true;
    hover_transform.scale = global_transform.scale;
    hover_transform.translation = Vec3::new(
        global_transform.translation.x,
        global_transform.translation.y,
        2.0,
    );
}

fn invalidate_tree_from_root(
    fs_root_query: Query<
        (Entity, Or<(Changed<FsAggregateSize>, Changed<Children>)>),
        (With<FsRootComponent>, With<Children>),
    >,
    fs_entity_details_query: Query<(
        &FsEntityKey,
        &FsEntityComponent,
        &FsAggregateSize,
        Option<&Children>,
    )>,
    mut fs_entity_mutable_details_query: Query<
        (
            // Styling
            &mut Transform,
            &mut GlobalTransform,
            &mut Sprite,
            &mut Visibility,
            Option<&mut DescendentColorRange>,
        ),
        (
            With<FsEntityComponent>,
            Without<DiskUsageTreeViewTransformRoot>,
        ),
    >,
    transform_root_changed_query: Query<
        (&Transform, Changed<Transform>),
        With<DiskUsageTreeViewTransformRoot>,
    >,
    tree_options: Res<DiskUsageTreeOptions>,
    // These two values are initialized to their defaults by Local, and remain empty. We use these
    // as error fallbacks. Note that we've had to use a static lifetime for
    // `default_entity_ref_vec`, which is fine because it contains no entity refs.
    default_children_iter: Local<Children>,
    default_entity_ref_vec: Local<Vec<(&'static Entity, f32, f32)>>,
) {
    let fs_root_res = fs_root_query.get_single();
    let transform_root_changed_res = transform_root_changed_query.get_single();

    let (
        fs_root,
        root_transform,
        tree_needs_redraw,
        fs_root_changed,
        root_transform_changed,
    ) = match (fs_root_res, transform_root_changed_res) {
        (
            Ok((fs_root, fs_root_changed)),
            Ok((root_transform, root_transform_changed)),
        ) => (
            fs_root,
            root_transform,
            fs_root_changed || root_transform_changed,
            fs_root_changed,
            root_transform_changed,
        ),

        _ => return,
    };

    if tree_needs_redraw {
        let span = info_span!("du_tree::invalidate()");
        let _enter_guard = span.enter();

        info!(
            fs_root_changed = fs_root_changed,
            root_transform_changed = root_transform_changed,
            "disk usage tree visuals invalidated"
        );

        let root_global_transform: GlobalTransform = (*root_transform).into();
        invalidate_subtree_recursive(
            &root_global_transform,
            &fs_root,
            &fs_entity_details_query,
            &mut fs_entity_mutable_details_query,
            &tree_options,
            &default_children_iter,
            &default_entity_ref_vec,
        );
    }
}

/// This is not a system — it is invoked
fn invalidate_subtree_recursive(
    parent_global_transform: &GlobalTransform,
    fs_parent: &Entity,
    fs_entity_details_query: &Query<(
        &FsEntityKey,
        &FsEntityComponent,
        &FsAggregateSize,
        Option<&Children>,
    )>,
    fs_entity_mutable_details_query: &mut Query<
        (
            // Styling
            &mut Transform,
            &mut GlobalTransform,
            &mut Sprite,
            &mut Visibility,
            Option<&mut DescendentColorRange>,
        ),
        (
            With<FsEntityComponent>,
            Without<DiskUsageTreeViewTransformRoot>,
        ),
    >,
    tree_options: &Res<DiskUsageTreeOptions>,
    default_children_iter: &Local<Children>,
    default_entity_ref_vec: &Local<Vec<(&'static Entity, f32, f32)>>,
) {
    let (parent_fs_key, parent_fs_entity, parent_fs_size, maybe_children) =
        fs_entity_details_query.get(*fs_parent).unwrap();
    let maybe_parent_color_range: Option<DescendentColorRange> =
        fs_entity_mutable_details_query
            .get_component::<DescendentColorRange>(*fs_parent)
            .ok()
            .map(|rng| *rng); // This dereference returns the immutable borrow

    if parent_fs_entity.depth >= tree_options.max_depth {
        return;
    }

    debug!(
        key = parent_fs_key.as_value(),
        global.translation = ?parent_global_transform.translation,
        global.scale = ?parent_global_transform.scale,
        "invalidating subtree"
    );

    // Determine the visibility of children. Any child whose coloured region is less than 1 logical
    // pixel will not be displayed.

    let bytes_to_fractional_x =
        |bytes: u64| bytes as f32 / parent_fs_size.size_in_bytes as f32;
    let fractional_x_to_screen_x = |fractional_x: f32, total_screen_w: Option<f32>| {
        fractional_x * total_screen_w.unwrap_or(parent_global_transform.scale.x)
    };
    let screen_x_to_fractional_x =
        |screen_x: f32| screen_x / parent_global_transform.scale.x;

    let children_by_visibility = maybe_children
        .unwrap_or(default_children_iter)
        .iter()
        .filter(|child| fs_entity_details_query.contains(**child))
        .map(|child| {
            let child_fs_size = fs_entity_details_query
                .get_component::<FsAggregateSize>(*child)
                .unwrap();
            let fractional_w = bytes_to_fractional_x(child_fs_size.size_in_bytes);
            let screen_w = fractional_x_to_screen_x(fractional_w, None);
            (child, fractional_w, screen_w)
        })
        .grouping_by(|(_child, _fractional_w, screen_w)| {
            *screen_w >= MIN_CHILD_WIDTH_WITH_GAP
        });
    let visible_children = children_by_visibility
        .get(&true)
        .unwrap_or(default_entity_ref_vec);
    let hidden_children = children_by_visibility
        .get(&false)
        .unwrap_or(default_entity_ref_vec);

    // Determine whether we should include a synthetic child of the parent that groups together all
    // the files too small to display.
    let hidden_children_screen_w = hidden_children
        .iter()
        .map(|(_, _, screen_w)| *screen_w)
        .sum::<f32>();
    let use_group_for_hidden_children =
        hidden_children_screen_w > MIN_CHILD_WIDTH_WITH_GAP;

    // Calculate the number of gaps, and the unit proportion that will cut into the children's
    // space
    let number_of_gaps =
        (visible_children.len() + use_group_for_hidden_children as usize).max(1) - 1;
    let total_gap_screen_w = number_of_gaps as f32 * GAP_WIDTH;
    let available_screen_w_minus_gaps =
        parent_global_transform.scale.x - total_gap_screen_w;

    debug!(
        visible_children_count = visible_children.len(),
        number_of_gaps,
        use_group_for_hidden_children,
        hidden_children_screen_w,
        total_gap_screen_w,
        available_screen_w_minus_gaps,
        "layout parameters determined"
    );

    let mut fractional_x = 0_f32; // Running child x coordinate, in the range [0..1]
    let mut remainder_screen_w = 0_f32;

    let maybe_last_visible_child = visible_children.last();
    for (child, child_fractional_w, _) in visible_children {
        // For some reason the grouping_by() operator maps to references of the elements, so we
        // deref up front, shadowing the references
        let (child, child_fractional_w) = (*child, *child_fractional_w);
        let is_last = maybe_last_visible_child.unwrap().0 == child;

        let (child_fs_key, child_fs, _, _) = fs_entity_details_query.get(*child).unwrap();
        let (
            mut child_transform,
            _child_global_transform,
            mut child_sprite,
            mut child_vis,
            maybe_child_color_range,
        ) = fs_entity_mutable_details_query.get_mut(*child).unwrap();

        // Set the child to visible
        child_vis.is_visible = true;

        // Update the child's position/size using fractional values
        let child_screen_w_minus_gaps = fractional_x_to_screen_x(
            child_fractional_w,
            Some(available_screen_w_minus_gaps),
        );
        remainder_screen_w +=
            child_screen_w_minus_gaps - child_screen_w_minus_gaps.floor();
        let child_screen_w_minus_gaps = child_screen_w_minus_gaps.floor();
        let child_fractional_w_minus_gaps = screen_x_to_fractional_x(
            child_screen_w_minus_gaps + if is_last { remainder_screen_w } else { 0.0 },
        );
        child_transform.scale.x = child_fractional_w_minus_gaps;
        child_transform.translation.x = fractional_x;

        debug!(
            key = child_fs_key.as_value(),
            child_screen_w_minus_gaps,
            child_fractional_w_minus_gaps,
            child_scale_x = child_transform.scale.x,
            child_translation_x = fractional_x,
            remainder_screen_w,
            "child positioned"
        );

        // If child is a directory, update the sprite color and descendent color range to reflect
        // the child's position in the parent
        if let Some(mut child_color_range) = maybe_child_color_range {
            let parent_color_range = maybe_parent_color_range.unwrap();
            *child_color_range =
                parent_color_range.sub_range(fractional_x, child_fractional_w);
            debug!(
                key = child_fs_key.as_value(),
                parent_range = parent_color_range.as_value(),
                child_range = child_color_range.as_value(),
                fractional_x,
                child_fractional_w,
                "determined child color range",
            );
            child_sprite.color = child_color_range.get_color(0.0, child_fs.depth);
        }

        // Increment x for the next child
        fractional_x +=
            child_fractional_w_minus_gaps + screen_x_to_fractional_x(GAP_WIDTH);
    }

    // Ensure that all the hidden children are marked hidden
    for (hidden_fs_child, _, screen_w) in hidden_children {
        let hidden_fs_child = *hidden_fs_child;
        let child_key = fs_entity_details_query
            .get_component::<FsEntityKey>(*hidden_fs_child)
            .unwrap();
        trace!(
            child_key = child_key.as_value(),
            child_screen_width = screen_w,
            "hiding child too small to display"
        );
        hide_subtree_recursive(
            hidden_fs_child,
            fs_entity_details_query,
            fs_entity_mutable_details_query,
        );
    }

    // Invalidate the subtrees of visible children
    for (child, _, _) in visible_children {
        let child = *child;

        let child_transform = fs_entity_mutable_details_query
            .get_component::<Transform>(*child)
            .unwrap();

        // Compute the global transform for the child (the one stored can be out of date)
        let child_global_transform =
            parent_global_transform.mul_transform(*child_transform);

        {
            let mut mut_child_global_transform = fs_entity_mutable_details_query
                .get_component_mut::<GlobalTransform>(*child)
                .unwrap();
            *mut_child_global_transform = child_global_transform;
        }

        // Invalidate the subtree rooted at child (if one exists)
        invalidate_subtree_recursive(
            &child_global_transform,
            child,
            fs_entity_details_query,
            fs_entity_mutable_details_query,
            tree_options,
            default_children_iter,
            default_entity_ref_vec,
        );
    }
}

fn hide_subtree_recursive(
    hidden_fs_parent: &Entity,
    fs_entity_details_query: &Query<(
        &FsEntityKey,
        &FsEntityComponent,
        &FsAggregateSize,
        Option<&Children>,
    )>,
    fs_entity_mutable_details_query: &mut Query<
        (
            &mut Transform,
            &mut GlobalTransform,
            &mut Sprite,
            &mut Visibility,
            Option<&mut DescendentColorRange>,
        ),
        (
            With<FsEntityComponent>,
            Without<DiskUsageTreeViewTransformRoot>,
        ),
    >,
) {
    if !fs_entity_mutable_details_query.contains(*hidden_fs_parent) {
        return;
    }

    let mut child_vis_res = fs_entity_mutable_details_query
        .get_component_mut::<Visibility>(*hidden_fs_parent)
        .unwrap();

    if child_vis_res.is_visible {
        child_vis_res.is_visible = false;
    } else {
        return; // stop at an invisible subtree
    }

    if let Some(children) = fs_entity_details_query
        .get_component::<Children>(*hidden_fs_parent)
        .ok()
    {
        for hidden_fs_child in children.iter() {
            hide_subtree_recursive(
                hidden_fs_child,
                fs_entity_details_query,
                fs_entity_mutable_details_query,
            );
        }
    }
}
