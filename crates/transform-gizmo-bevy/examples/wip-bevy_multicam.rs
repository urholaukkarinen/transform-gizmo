// TODO: Does not currently work!

//! An example with two cameras in split screen
//! Adapted from the official bevy example:
//! <https://bevyengine.org/examples/3D%20Rendering/split-screen/>
//! See the project root's `examples` directory for more examples
//!
//! NOTE: DOES NOT WORK!

use bevy::prelude::*;
use bevy_render::camera::Viewport;
use bevy_window::WindowResized;
use transform_gizmo_bevy::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TransformGizmoPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, set_camera_viewports)
        .run();
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // camera left
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(1.0, 3.0, -5.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GizmoCamera,
        LeftCamera,
    ));

    // camera right
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(-3.0, 2.0, -4.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        GizmoCamera,
        RightCamera,
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

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let mut left_camera = left_camera.single_mut();
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });

        let mut right_camera = right_camera.single_mut();
        right_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(window.resolution.physical_width() / 2, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });
    }
}
