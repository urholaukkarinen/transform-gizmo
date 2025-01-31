use bevy_app::{Plugin, PreUpdate};
use bevy_ecs::{
    event::EventWriter,
    schedule::IntoSystemConfigs,
    system::{Query, Res},
};
use bevy_picking::{
    backend::{HitData, PointerHits},
    pointer::{PointerId, PointerLocation},
    PickSet,
};

use crate::GizmoStorage;

pub struct TransformGizmoPickingPlugin;

impl Plugin for TransformGizmoPickingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(PreUpdate, update_hits.in_set(PickSet::Backend));
    }
}

fn update_hits(
    storage: Res<GizmoStorage>,
    mut output: EventWriter<PointerHits>,
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
            .map(|(entity, _gizmo)| {
                // TODO: Provide camera entity based on the window the pointer was in?? idk
                (*entity, HitData::new(*entity, 0.0, None, None))
            })
            .collect::<Vec<_>>();

        // TODO: Use a purpose-picked order for hits. It should be below ui and egui, in front of f32::NEG_INFINITY
        // TODO: Perhapse this should be configurable through a resource? Or just use camera order +/- an offset like what UI picking does?

        output.send(PointerHits::new(*pointer_id, hits, 0.0));
    }
}
