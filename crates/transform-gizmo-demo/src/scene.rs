use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_mod_picking::{
    picking_core::PickingPluginsSettings, prelude::*, selection::SelectionPluginSettings,
};
use transform_gizmo_bevy::GizmoTarget;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(DefaultPickingPlugins.build())
            .add_plugins(OutlinePlugin)
            .insert_resource(SelectionPluginSettings {
                click_nothing_deselect_all: false,
                ..default()
            })
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, toggle_picking_enabled)
            .add_systems(Update, update);
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
    ));

    let cube_count: i32 = 3;

    let colors = [Color::RED, Color::GREEN, Color::BLUE];

    // Cubes
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

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn toggle_picking_enabled(
    gizmo_targets: Query<&GizmoTarget>,
    mut picking_settings: ResMut<PickingPluginsSettings>,
) {
    // Picking is disabled when any of the gizmos is focused or active.

    picking_settings.is_enabled = gizmo_targets
        .iter()
        .all(|target| !target.is_focused && !target.is_active);
}

fn update(
    mut commands: Commands,
    mut targets: Query<(
        Entity,
        &PickSelection,
        &mut OutlineVolume,
        Option<&GizmoTarget>,
    )>,
) {
    for (entity, pick_interaction, mut outline, gizmo_target) in targets.iter_mut() {
        let mut entity_cmd = commands.entity(entity);

        if pick_interaction.is_selected {
            if gizmo_target.is_none() {
                entity_cmd.insert(GizmoTarget::default());
            }

            outline.visible = true;
        } else {
            entity_cmd.remove::<GizmoTarget>();

            outline.visible = false;
        }
    }
}
