use bevy::{
    picking::pointer::{PointerInteraction, PointerPress},
    prelude::*,
};
use bevy_mod_outline::*;
use transform_gizmo_bevy::GizmoTarget;

#[derive(Component, Clone, Copy)]
pub struct PickSelection {
    pub is_selected: bool,
}

/// Integrates picking with gizmo and highlighting.
pub struct GizmoPickingPlugin;

impl Plugin for GizmoPickingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(OutlinePlugin)
            .add_plugins(MeshPickingPlugin)
            .add_systems(PreUpdate, toggle_picking_enabled)
            .add_systems(Update, update_picking)
            .add_systems(Update, manage_selection);
    }
}

fn toggle_picking_enabled(
    gizmo_targets: Query<&GizmoTarget>,
    mut picking_settings: ResMut<PickingPlugin>,
) {
    // Picking is disabled when any of the gizmos is focused or active.

    picking_settings.is_enabled = gizmo_targets
        .iter()
        .all(|target| !target.is_focused() && !target.is_active());
}

pub fn update_picking(
    mut targets: Query<
        (
            Entity,
            &PickSelection,
            &mut OutlineVolume,
            Option<&GizmoTarget>,
        ),
        Changed<PickSelection>,
    >,
    mut commands: Commands,
) {
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

pub fn manage_selection(
    pointers: Query<&PointerInteraction, Changed<PointerPress>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut pick_selection: Query<&mut PickSelection>,
) {
    // don't continue if the pointer was just pressed.
    if !mouse.just_released(MouseButton::Left) {
        return;
    };
    let pointer = match pointers.single() {
        Ok(pointer) => pointer,
        Err(err) => match err {
            bevy::ecs::query::QuerySingleError::NoEntities(_) => {
                // warn!(err);
                return;
            }
            bevy::ecs::query::QuerySingleError::MultipleEntities(_) => {
                warn!("demo only works with one pointer. delete extra pointer sources!");
                return;
            }
        },
    };
    if let Some((e, _)) = pointer.first() {
        let Ok(root) = pick_selection.get(*e).map(|n| n.is_selected) else {
            return;
        };

        if !keys.pressed(KeyCode::ShiftLeft) {
            for mut pick in &mut pick_selection {
                pick.is_selected = false;
            }
        }

        let Ok(mut pick) = pick_selection.get_mut(*e) else {
            return;
        };
        pick.is_selected = root;
        pick.is_selected ^= true;
    }
}
