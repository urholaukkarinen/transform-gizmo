use bevy::{prelude::*, window::WindowResolution};

use transform_gizmo_bevy::*;

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
        .add_systems(Update, update)
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
        gizmo_modes: enum_set!(GizmoMode::Rotate | GizmoMode::Scale | GizmoMode::Translate),
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

fn update(mut gizmo_options: ResMut<GizmoOptions>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if gizmo_options.gizmo_modes.contains(GizmoMode::Rotate) {
            gizmo_options.gizmo_modes.remove(GizmoMode::Rotate);
        } else {
            gizmo_options.gizmo_modes.insert(GizmoMode::Rotate);
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyT) {
        if gizmo_options.gizmo_modes.contains(GizmoMode::Translate) {
            gizmo_options.gizmo_modes.remove(GizmoMode::Translate);
        } else {
            gizmo_options.gizmo_modes.insert(GizmoMode::Translate);
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        if gizmo_options.gizmo_modes.contains(GizmoMode::Scale) {
            gizmo_options.gizmo_modes.remove(GizmoMode::Scale);
        } else {
            gizmo_options.gizmo_modes.insert(GizmoMode::Scale);
        }
    }
}
