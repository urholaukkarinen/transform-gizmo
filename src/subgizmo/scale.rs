use egui::Ui;
use glam::DVec3;

use crate::math::{round_to_interval, world_to_screen};

use crate::subgizmo::common::{plane_binormal, plane_tangent, PickResult};
use crate::subgizmo::{SubGizmo, SubGizmoKind, SubGizmoState};
use crate::{GizmoMode, GizmoResult, Ray};

/// Picks given scale subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_scale(
    subgizmo: &SubGizmo,
    ui: &Ui,
    ray: Ray,
    pick_fn: impl Fn(&SubGizmo, Ray) -> PickResult,
) -> Option<f64> {
    let pick_result = pick_fn(subgizmo, ray);
    let start_delta = distance_from_origin_2d(subgizmo, ui)?;

    subgizmo.update_state_with(ui, |state: &mut SubGizmoState<ScaleState>| {
        state.start_scale = subgizmo.config.scale;
        state.start_delta = start_delta;
        state.visibility = pick_result.visibility as _;
    });

    if pick_result.picked {
        Some(pick_result.t)
    } else {
        None
    }
}

/// Updates given scale subgizmo.
pub(crate) fn update_scale(subgizmo: &SubGizmo, ui: &Ui) -> Option<GizmoResult> {
    let state = subgizmo.state::<ScaleState>(ui);
    let mut delta = distance_from_origin_2d(subgizmo, ui)?;
    delta /= state.start_delta;

    if subgizmo.config.snapping {
        delta = round_to_interval(delta, subgizmo.config.snap_scale as f64);
    }
    delta = delta.max(1e-4) - 1.0;

    let direction = if subgizmo.kind == SubGizmoKind::ScalePlane {
        let binormal = plane_binormal(subgizmo.direction);
        let tangent = plane_tangent(subgizmo.direction);
        (binormal + tangent).normalize()
    } else {
        subgizmo.local_normal()
    };

    let offset = DVec3::ONE + (direction * delta);
    let new_scale = state.start_scale * offset;

    Some(GizmoResult {
        scale: new_scale.as_vec3().into(),
        rotation: subgizmo.config.rotation.as_f32().into(),
        translation: subgizmo.config.translation.as_vec3().into(),
        mode: GizmoMode::Scale,
        value: offset.as_vec3().to_array(),
    })
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ScaleState {
    start_scale: DVec3,
    start_delta: f64,
}

fn distance_from_origin_2d(subgizmo: &SubGizmo, ui: &Ui) -> Option<f64> {
    let cursor_pos = ui.input(|i| i.pointer.hover_pos())?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, DVec3::new(0.0, 0.0, 0.0))?;

    Some(cursor_pos.distance(gizmo_pos) as f64)
}
