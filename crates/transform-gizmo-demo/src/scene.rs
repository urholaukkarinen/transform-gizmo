use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_mod_picking::prelude::*;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::default());

    // Ground
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: materials.add(Color::NONE),
            transform: Transform::from_xyz(0.0, -0.5, 0.0).with_scale(Vec3::new(100.0, 1.0, 100.0)),
            ..default()
        },
        NoDeselect,
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
    ));

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
                        1.0,
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
