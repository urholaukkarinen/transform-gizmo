use bevy_app::{Plugin, PreUpdate};
use bevy_ecs::{
    message::MessageWriter,
    schedule::IntoScheduleConfigs,
    system::{Query, Res},
};
use bevy_picking::{
    PickingSystems,
    backend::{HitData, PointerHits},
    pointer::{PointerId, PointerLocation},
};

use crate::GizmoStorage;

pub struct TransformGizmoPickingPlugin;

impl Plugin for TransformGizmoPickingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(PreUpdate, update_hits.in_set(PickingSystems::Backend));
    }
}

fn update_hits(
    storage: Res<GizmoStorage>,
    mut output: MessageWriter<PointerHits>,
    pointers: Query<(&PointerId, &PointerLocation)>,
) {
    let gizmos = storage
        .entity_gizmo_map
        .iter()
        .filter_map(|(entity, uuid)| storage.gizmos.get(uuid).map(|gizmo| (*entity, gizmo)))
        .collect::<Vec<_>>();

    for (pointer_id, pointer_location) in &pointers {
        let Some(location) = &pointer_location.location else {
            continue;
        };
        let hits = gizmos
            .iter()
            .filter(|(_entity, gizmo)| {
                gizmo.pick_preview((location.position.x, location.position.y))
            })
            .map(|(entity, _gizmo)| (*entity, HitData::new(*entity, 0.0, None, None)))
            .collect::<Vec<_>>();

        output.write(PointerHits::new(*pointer_id, hits, 0.0));
    }
}
