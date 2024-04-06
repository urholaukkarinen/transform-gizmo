use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_mod_picking::prelude::*;

use transform_gizmo_bevy::GizmoCamera;

use crate::camera::PanOrbitCamera;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
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
        Camera3dBundle {
            transform: camera_transform.looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GizmoCamera,
    ));

    let cube_mesh = meshes.add(Cuboid::default());

    let cube_count: i32 = 3;

    let colors = [Color::RED, Color::GREEN, Color::BLUE];

    for i in 0..cube_count {
        commands
            .spawn((
                PbrBundle {
                    mesh: cube_mesh.clone(),
                    material: materials.add(colors[i as usize % colors.len()]),
                    transform: Transform::from_xyz(
                        -(cube_count / 2) as f32 * 1.5 + (i as f32 * 1.5),
                        0.0,
                        0.0,
                    ),
                    ..default()
                },
                PickableBundle {
                    selection: PickSelection { is_selected: true },
                    ..default()
                },
            ))
            .insert(OutlineBundle {
                outline: OutlineVolume {
                    visible: false,
                    colour: Color::WHITE,
                    width: 2.0,
                },
                ..default()
            });
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
