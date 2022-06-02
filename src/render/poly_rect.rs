use bevy::{
    prelude::{Assets, Color, Deref, Handle, Mesh, Plugin},
    render::mesh::{Indices, PrimitiveTopology},
};

/// The number of horizontal sections in the poly rect mesh
const POLY_RECT_SECTION_COUNT: usize = 40;

pub struct PolyRectMeshPlugin;
impl Plugin for PolyRectMeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mesh = create_poly_rect_mesh(POLY_RECT_SECTION_COUNT);
        let handle = app
            .world
            .get_resource_mut::<Assets<Mesh>>()
            .unwrap()
            .add(mesh);
        app.insert_resource(PolyRectMeshHandle(handle));
    }
}

#[derive(Deref)]
pub struct PolyRectMeshHandle(pub Handle<Mesh>);

/// Creates and returns a poly rect [`Mesh`] with the specified number of (horizontal)
/// segments. This mesh
fn create_poly_rect_mesh(x_section_count: usize) -> Mesh {
    let edge_vertex_count: usize = x_section_count + 1;
    let rect_vertex_count: usize = edge_vertex_count * 2;

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // Compute vertex positions for the mesh
    let top = (0..=x_section_count)
        .map(|i| [i as f32 / x_section_count as f32 - 0.5, 0.5, 0.0]);
    let bottom = (0..=x_section_count)
        .map(|i| [i as f32 / x_section_count as f32 - 0.5, -0.5, 0.0]);
    let v_positions: Vec<[f32; 3]> = top.chain(bottom).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_positions);

    // Provide the triangle indices (counter clockwise ordering)
    let v_indices = Indices::U32(
        (0..x_section_count)
            .flat_map(|top_i| {
                let top_i = top_i as u32;
                let bottom_i = top_i + edge_vertex_count as u32;
                // These are the two triangles for a horizontal slice of the poly rect
                [
                    // tri 1
                    top_i + 1,
                    top_i,
                    bottom_i,
                    // tri 2
                    bottom_i,
                    bottom_i + 1,
                    top_i + 1,
                ]
            })
            .collect(),
    );
    mesh.set_indices(Some(v_indices));

    // Add some colors to satisfy the shader
    // let mut rand = rand::thread_rng();
    let v_colors: Vec<_> = (0..rect_vertex_count)
        .map(|_| Color::rgb(0.05, 0.05, 0.05))
        .map(Color::as_linear_rgba_u32)
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_colors);

    mesh
}
