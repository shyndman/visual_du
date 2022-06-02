use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            RenderPipelineDescriptor, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
        },
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};
use visual_du::{
    app_scaffold::{AppScaffoldPlugin, WindowSize},
    render::poly_rect::{PolyRectMeshHandle, PolyRectMeshPlugin},
};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 1 })
        .add_plugin(AppScaffoldPlugin {
            title: "Custom Pipeline",
            bin_module_path: module_path!(),
        })
        .add_plugin(PolyRectMeshPlugin)
        .add_plugin(RenderPolyRectPlugin)
        .add_startup_system(create_camera)
        .add_startup_system(draw_poly_rects)
        .add_system(update_poly_rects);

    app.run();
}

fn create_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

const RECT_COLS: usize = 3;
const RECT_ROWS: usize = 10;
const RECT_COUNT: usize = RECT_COLS * RECT_ROWS;
const GAP_SIZE: f32 = 10.0;

/// System for creating a poly rect [Mesh], and registering it in the [Assets<Mesh>] resource.
fn draw_poly_rects(mut commands: Commands, poly_rect_handle: Res<PolyRectMeshHandle>) {
    for _i in 0..RECT_COUNT {
        // We can now spawn the entities for the polyrect and the camera
        commands.spawn_bundle((
            // We use a marker component to identify the custom colored meshes
            PolyRect::default(),
            // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
            Mesh2dHandle(poly_rect_handle.clone()),
            // These other components are needed for 2d meshes to be rendered
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            ComputedVisibility::default(),
        ));
    }
}

fn update_poly_rects(
    mut transform_query: Query<&mut Transform, With<PolyRect>>,
    window_size: Res<WindowSize>,
) {
    let min_x = -window_size.x / 2.0;
    let min_y = -window_size.y / 2.0;
    let width_per = window_size.x / RECT_COLS as f32;
    let height_per = window_size.y / RECT_ROWS as f32;

    info!(width_per, height_per, "update_poly_rects");

    for (i, mut t) in transform_query.iter_mut().enumerate() {
        let (c, r) = (i % RECT_COLS, i / RECT_COLS);

        *t = Transform {
            translation: Vec3::new(
                min_x + c as f32 * width_per,
                min_y + r as f32 * height_per + 50.0,
                0.0,
            ),
            scale: Vec3::new(width_per - GAP_SIZE, height_per - GAP_SIZE, 1.0),
            ..Transform::default()
        };
        break;
    }
}

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct PolyRect;

/// Custom pipeline for 2d meshes with vertex colors
pub struct PolyRectPipeline {
    /// This pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
    shader: Handle<Shader>,
}

impl FromWorld for PolyRectPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load our custom shader
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
            shader: world.resource::<AssetServer>().load(POLY_RECT_SHADER_PATH),
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedRenderPipeline for PolyRectPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            // Color
            VertexFormat::Uint32,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        RenderPipelineDescriptor {
            label: Some("poly_rect_pipeline".into()),
            vertex: VertexState {
                // Use our custom shader
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                // Use our custom vertex buffer
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: self.shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}

// This specifies how to render a colored 2d mesh
type DrawPolyRect = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);

/// Plugin that renders [`ColoredMesh2d`]s
pub struct RenderPolyRectPlugin;

/// Path to the shader used to render our mesh
const POLY_RECT_SHADER_PATH: &str = "shaders/example/poly_rect.wgsl";

impl Plugin for RenderPolyRectPlugin {
    fn build(&self, app: &mut App) {
        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawPolyRect>()
            .init_resource::<PolyRectPipeline>()
            .init_resource::<SpecializedRenderPipelines<PolyRectPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_poly_rects)
            .add_system_to_stage(RenderStage::Queue, queue_poly_rects);
    }
}

/// Extract the [`PolyRect`] marker component into the render app
pub fn extract_poly_rects(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<PolyRect>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (PolyRect,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`PolyRect`] using our custom pipeline and draw
/// function
#[allow(clippy::too_many_arguments)]
pub fn queue_poly_rects(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<PolyRectPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PolyRectPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<PolyRect>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawPolyRect>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) =
                colored_mesh2d.get(*visible_entity)
            {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |= Mesh2dPipelineKey::from_primitive_topology(
                        mesh.primitive_topology,
                    );
                }

                let pipeline_id = pipelines.specialize(
                    &mut pipeline_cache,
                    &colored_mesh2d_pipeline,
                    mesh2d_key,
                );

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before
                    // rendering, in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    }
}
