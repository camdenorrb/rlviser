use bevy::{
    prelude::*,
    render::mesh::{self, PrimitiveTopology},
};
use byteorder::{LittleEndian, ReadBytesExt};
use rand::Rng;
use std::{
    f32::consts::PI,
    fs::{read_dir, File},
    io,
    path::Path,
};
use warbler_grass::prelude::*;

#[derive(Resource)]
pub struct GrassLod(u8);

impl Default for GrassLod {
    fn default() -> Self {
        GrassLod(2)
    }
}

impl GrassLod {
    #[inline]
    pub fn get(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn set(&mut self, lod: u8) {
        self.0 = lod;
    }
}

pub fn get_grass(lod: u8) -> (Vec<Vec3>, f32, Transform) {
    if lod == 0 {
        return (Vec::new(), 1.5, Transform::from_scale(Vec3::splat(10.)));
    }

    let mut rand = rand::thread_rng();

    if lod == 1 {
        (
            (-375..375)
                .step_by(3)
                .flat_map(|x| (-495..495).step_by(3).map(move |z| Vec3::new(x as f32, 1., z as f32)))
                .map(|pos| pos + Vec3::new(rand.gen_range(-2.0..2.), 0., rand.gen_range(-2.0..2.)))
                .collect::<Vec<_>>(),
            1.5,
            Transform::from_scale(Vec3::splat(10.)),
        )
    } else {
        (
            (-375 * 2..375 * 2)
                .step_by(3)
                .flat_map(|x| (-495 * 2..495 * 2).step_by(3).map(move |z| Vec3::new(x as f32, 1., z as f32)))
                .map(|pos| pos + Vec3::new(rand.gen_range(-2.0..2.), 0., rand.gen_range(-2.0..2.)))
                .collect::<Vec<_>>(),
            3.,
            Transform::from_scale(Vec3::splat(5.)),
        )
    }
}

pub struct FieldLoaderPlugin;

impl Plugin for FieldLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WarblersPlugin).insert_resource(GrassLod::default()).add_startup_system(load_field);
    }
}

fn load_field(mut commands: Commands, grass_lod: Res<GrassLod>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Get all files in ./collision_meshes/soccar/*.cmf
    let raw_mesh = MeshBuilder::combine(
        &read_dir("./collision_meshes/soccar")
            .unwrap()
            .flatten()
            .flat_map(|entry| MeshBuilder::from_file(entry.path()))
            .collect::<Vec<MeshBuilder>>(),
    );

    let inverted_mesh = raw_mesh.clone().invert_indices().build_mesh(50.);
    let mesh = raw_mesh.build_mesh(50.);

    // load the files into the game with their material

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.2, 0.2),
            alpha_mode: AlphaMode::Opaque,
            ..default()
        }),
        transform: Transform::from_xyz(0., 1., 0.).looking_to(-Vec3::Y, Vec3::Z),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(inverted_mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.2, 0.2, 0.2, 0.85),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_xyz(0., 1., 0.).looking_to(-Vec3::Y, Vec3::Z),
        ..default()
    });

    // load the side walls

    let mut side_wall_1_transform = Transform::from_xyz(4096., 900., 0.);
    side_wall_1_transform.rotate_local_y(-PI / 2.);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(7000., 1200.)))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.2, 0.2, 0.2, 0.9),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        transform: side_wall_1_transform,
        ..default()
    });

    let mut side_wall_2_transform = Transform::from_xyz(-4096., 900., 0.);
    side_wall_2_transform.rotate_local_y(PI / 2.);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(7000., 1200.)))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.2, 0.2, 0.2, 0.9),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        transform: side_wall_2_transform,
        ..default()
    });

    // load floor

    let mut floor_transform = Transform::default();
    floor_transform.rotate_local_x(-PI / 2.);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(7500., 10800.)))),
        material: materials.add(Color::rgb(0.7, 1., 0.7).into()),
        transform: floor_transform,
        ..default()
    });

    // load ceiling

    let ceiling_material = StandardMaterial {
        base_color: Color::rgba(0.2, 0.2, 0.2, 0.85),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    };

    let mut ceiling_transform = Transform::from_xyz(0., 2060., 0.);
    ceiling_transform.rotate_local_x(PI / 2.);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(6950., 8725.)))),
        material: materials.add(ceiling_material),
        transform: ceiling_transform,
        ..default()
    });

    // load grass

    let (positions, height, transform) = get_grass(grass_lod.get());

    commands.spawn(WarblersExplicitBundle {
        grass: Grass::new(positions, height),
        spatial: SpatialBundle { transform, ..default() },
        ..default()
    });
}

/// A collection of inter-connected triangles.
#[derive(Clone, Debug, Default)]
struct MeshBuilder {
    ids: Vec<u32>,
    verts: Vec<f32>,
}

impl MeshBuilder {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;

        let ids_len = file.read_i32::<LittleEndian>()? * 3;
        let verts_len = file.read_i32::<LittleEndian>()? * 3;

        let ids = (0..ids_len).map(|_| file.read_i32::<LittleEndian>().map(|x| x as u32)).collect::<io::Result<Vec<_>>>()?;
        let verts = (0..verts_len - verts_len % 3).map(|_| file.read_f32::<LittleEndian>()).collect::<io::Result<Vec<_>>>()?;

        // Verify that the triangle data is correct
        let max_vert = verts.len() as u32 / 3;
        for &id in &ids {
            if id >= max_vert {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid triangle data"));
            }
        }

        Ok(Self { ids, verts })
    }

    #[must_use]
    /// Combine different meshes all into one
    pub fn combine(other_meshes: &[Self]) -> Self {
        let n_ids = other_meshes.iter().map(|mesh| mesh.ids.len()).sum();
        let mut ids: Vec<u32> = Vec::with_capacity(n_ids);
        let mut id_offset = 0;

        for m in other_meshes {
            ids.extend(m.ids.iter().map(|id| id + id_offset));
            id_offset += m.verts.len() as u32 / 3;
        }

        let verts: Vec<f32> = other_meshes.iter().flat_map(|m| m.verts.clone()).collect();

        Self { ids, verts }
    }

    #[must_use]
    pub fn invert_indices(self) -> Self {
        let mut ids = self.ids;
        for chunk in ids.chunks_exact_mut(3) {
            chunk.swap(1, 2);
        }
        Self { ids, verts: self.verts }
    }

    #[must_use]
    // Build the Bevy Mesh
    pub fn build_mesh(self, scale: f32) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(mesh::Indices::U32(self.ids)));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.verts.chunks(3).map(|chunk| [chunk[0] * scale, chunk[1] * scale, chunk[2] * scale]).collect::<Vec<_>>(),
        );
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
        mesh
    }
}
