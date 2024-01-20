use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

use bevy_flycam::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<LineMaterial>::default()))
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, lorenz_system)
        .run();
}

#[derive(Resource)]
struct LorenzPostition {
    translation: Vec3,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-100., 0., 150.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCam,
    ));

    commands.insert_resource(LorenzPostition {
        translation: Vec3::new(0.1, 0., 0.1),
    })
}

const A: f32 = 10.;
const B: f32 = 8. / 3.;
const C: f32 = 28.;
const DT: f32 = 0.001;

fn lorenz_system(
    mut lorenz: ResMut<LorenzPostition>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for _ in 0..50 {
        let previous_translation = lorenz.translation;

        let dx = A * (lorenz.translation.y - lorenz.translation.x);

        let dy = lorenz.translation.x * (C - lorenz.translation.z) - lorenz.translation.y;

        let dz = lorenz.translation.x * lorenz.translation.y - B * lorenz.translation.z;

        lorenz.translation.x += dx * DT;

        lorenz.translation.y += dy * DT;

        lorenz.translation.z += dz * DT;

        let h = map_range((-13., 13.), (25., 35.), lorenz.translation.x);
        let l = map_range((-28., 28.), (0.3, 0.7), lorenz.translation.y);

        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(LineStrip {
                points: vec![previous_translation, lorenz.translation],
            })),
            material: materials.add(LineMaterial {
                color: Color::hsla(h, 0.8, l, 0.5),
            }),
            ..default()
        });
    }
}

/**
Maps a number from an input range to an output range

# Panics
The function will panic if the type T passed does not implement
Copy, std::ops::Add, std::ops::Sub, std::ops::Mul, std::ops::Div

# Examples

```rust
let nums = vec![-1.,0.5, -0.4, -0.3, 0.8, 1, 0.2];
for num in nums {
    map_range((-1,1),(0,1), nums);
}
```
*/

fn map_range<T: Copy>(from_range: (T, T), to_range: (T, T), s: T) -> T
where
    T: std::ops::Add<T, Output = T>
        + std::ops::Sub<T, Output = T>
        + std::ops::Mul<T, Output = T>
        + std::ops::Div<T, Output = T>,
{
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of points that will have a line drawn between each consecutive points
#[derive(Debug, Clone)]
pub struct LineStrip {
    pub points: Vec<Vec3>,
}

impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        Mesh::new(PrimitiveTopology::LineStrip)
            // Add the point positions as an attribute
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line.points)
    }
}
