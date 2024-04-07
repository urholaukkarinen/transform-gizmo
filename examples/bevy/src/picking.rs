use bevy::prelude::*;
use bevy_mod_outline::*;
use bevy_mod_picking::{
    picking_core::PickingPluginsSettings, prelude::*, selection::SelectionPluginSettings,
};
use transform_gizmo_bevy::GizmoTarget;

/// Integrates picking with gizmo and highlighting.
pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(DefaultPickingPlugins.build())
            .add_plugins(OutlinePlugin)
            .insert_resource(SelectionPluginSettings {
                click_nothing_deselect_all: false,
                ..default()
            })
            .add_systems(PreUpdate, toggle_picking_enabled)
            .add_systems(Update, update_picking);
    }
}

fn toggle_picking_enabled(
    gizmo_targets: Query<&GizmoTarget>,
    mut picking_settings: ResMut<PickingPluginsSettings>,
) {
    // Picking is disabled when any of the gizmos is focused or active.

    picking_settings.is_enabled = gizmo_targets
        .iter()
        .all(|target| !target.is_focused() && !target.is_active());
}

fn update_picking(
    mut commands: Commands,
    mut targets: Query<(
        Entity,
        &PickSelection,
        &mut OutlineVolume,
        Option<&GizmoTarget>,
    )>,
) {
    // Continuously update entities based on their picking state

    for (entity, pick_interaction, mut outline, gizmo_target) in &mut targets {
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
