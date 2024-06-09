//! A very simple example
//! See the project root's `examples` directory for more examples

use bevy::prelude::*;
use transform_gizmo_bevy::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TransformGizmoPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(1.0, 3.0, -5.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GizmoCamera,
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::GREEN),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        GizmoTarget::default(),
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
