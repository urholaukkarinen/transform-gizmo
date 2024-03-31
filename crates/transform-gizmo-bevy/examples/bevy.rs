use bevy::{prelude::*, window::WindowResolution};

use transform_gizmo_bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(800.0, 600.0),
                title: "transform-gizmo-bevy example".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TransformGizmoPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let transform = Transform::from_xyz(5.0, 5.0, 5.0);

    commands.spawn((
        Camera3dBundle {
            transform: transform.looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GizmoCamera,
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight::default(),
        transform: Transform::from_xyz(5.0, 3.0, 5.0),
        ..default()
    });

    commands.insert_resource(GizmoOptions {
        gizmo_modes: enum_set!(GizmoMode::Rotate | GizmoMode::Scale),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid {
                half_size: Vec3::splat(1.0),
            })),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(-1.5, 0.0, 0.0)),
            ..default()
        },
        GizmoTarget::default(),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid {
                half_size: Vec3::splat(1.0),
            })),
            material: materials.add(Color::WHITE),
            transform: Transform::from_translation(Vec3::new(1.5, 0.0, 0.0)),
            ..default()
        },
        GizmoTarget::default(),
    ));
}
