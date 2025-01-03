use bevy::color::palettes::css::{BLUE, LIME, RED};
use bevy::prelude::*;
use bevy_mod_outline::*;

use transform_gizmo_bevy::GizmoCamera;

use crate::camera::PanOrbitCamera;
use crate::picking::PickSelection;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_transform = Transform::from_xyz(5.0, 5.0, 5.0);

    commands.spawn((
        PanOrbitCamera {
            radius: camera_transform.translation.length(),
            ..Default::default()
        },
        Camera3d::default(),
        camera_transform.looking_at(Vec3::ZERO, Vec3::Y),
        GizmoCamera,
    ));

    let cube_mesh = meshes.add(Cuboid::default());

    let cube_count: i32 = 3;

    let colors: [Color; 3] = [RED.into(), LIME.into(), BLUE.into()];

    for i in 0..cube_count {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(materials.add(colors[i as usize % colors.len()])),
            Transform::from_xyz(-(cube_count / 2) as f32 * 1.5 + (i as f32 * 1.5), 0.0, 0.0),
            // Pick,
            OutlineVolume {
                visible: false,
                colour: Color::WHITE,
                width: 2.0,
            },
            PickSelection { is_selected: true },
            OutlineStencil::default(),
            OutlineMode::default(),
            ComputedOutline::default(),
        ));
    }
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
