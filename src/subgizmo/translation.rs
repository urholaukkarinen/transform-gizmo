use egui::Ui;
use glam::DVec3;

use crate::math::{intersect_plane, ray_to_ray, round_to_interval};

use crate::subgizmo::common::{plane_binormal, plane_global_origin, plane_tangent, PickResult};
use crate::subgizmo::{SubGizmo, SubGizmoKind, SubGizmoState};
use crate::{GizmoMode, GizmoResult, Ray, WidgetData};

/// Picks given translation subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_translation(
    subgizmo: &SubGizmo,
    ui: &Ui,
    ray: Ray,
    pick_fn: impl Fn(&SubGizmo, Ray) -> PickResult,
) -> Option<f64> {
    let pick_result = pick_fn(subgizmo, ray);

    subgizmo.update_state_with(ui, |state: &mut SubGizmoState<TranslationState>| {
        state.start_point = pick_result.subgizmo_point;
        state.last_point = pick_result.subgizmo_point;
        state.current_delta = DVec3::ZERO;
        state.visibility = pick_result.visibility as _;
    });

    if pick_result.picked {
        Some(pick_result.t)
    } else {
        None
    }
}

/// Updates given translation subgizmo.
/// If the subgizmo is active, returns the translation result.
pub(crate) fn update_translation(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
    let state = subgizmo.state::<TranslationState>(ui);

    let mut new_point = if subgizmo.kind == SubGizmoKind::TranslationVector {
        point_on_axis(subgizmo, ray)
    } else {
        point_on_plane(subgizmo.normal(), plane_global_origin(subgizmo), ray)?
    };

    let mut new_delta = new_point - state.start_point;

    if subgizmo.config.snapping {
        new_delta = if subgizmo.kind == SubGizmoKind::TranslationVector {
            snap_translation_vector(subgizmo, new_delta)
        } else {
            snap_translation_plane(subgizmo, new_delta)
        };
        new_point = state.start_point + new_delta;
    }

    subgizmo.update_state_with(ui, |state: &mut SubGizmoState<TranslationState>| {
        state.last_point = new_point;
        state.current_delta = new_delta;
    });

    let new_translation = subgizmo.config.translation + new_point - state.last_point;

    Some(GizmoResult {
        scale: subgizmo.config.scale.as_vec3().into(),
        rotation: subgizmo.config.rotation.as_f32().into(),
        translation: new_translation.as_vec3().into(),
        mode: GizmoMode::Translate,
        value: state.current_delta.as_vec3().to_array(),
    })
}

fn snap_translation_vector(subgizmo: &SubGizmo, new_delta: DVec3) -> DVec3 {
    let delta_length = new_delta.length();
    if delta_length > 1e-5 {
        new_delta / delta_length
            * round_to_interval(delta_length, subgizmo.config.snap_distance as f64)
    } else {
        new_delta
    }
}

fn snap_translation_plane(subgizmo: &SubGizmo, new_delta: DVec3) -> DVec3 {
    let mut binormal = plane_binormal(subgizmo.direction);
    let mut tangent = plane_tangent(subgizmo.direction);
    if subgizmo.config.local_space() {
        binormal = subgizmo.config.rotation * binormal;
        tangent = subgizmo.config.rotation * tangent;
    }
    let cb = new_delta.cross(-binormal);
    let ct = new_delta.cross(tangent);
    let lb = cb.length();
    let lt = ct.length();
    let n = subgizmo.normal();

    if lb > 1e-5 && lt > 1e-5 {
        binormal * round_to_interval(lt, subgizmo.config.snap_distance as f64) * (ct / lt).dot(n)
            + tangent
                * round_to_interval(lb, subgizmo.config.snap_distance as f64)
                * (cb / lb).dot(n)
    } else {
        new_delta
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct TranslationState {
    start_point: DVec3,
    last_point: DVec3,
    current_delta: DVec3,
}

impl WidgetData for TranslationState {}

/// Finds the nearest point on line that points in translation subgizmo direction
fn point_on_axis(subgizmo: &SubGizmo, ray: Ray) -> DVec3 {
    let origin = subgizmo.config.translation;
    let direction = subgizmo.normal();

    let (_ray_t, subgizmo_t) = ray_to_ray(ray.origin, ray.direction, origin, direction);

    origin + direction * subgizmo_t
}

fn point_on_plane(plane_normal: DVec3, plane_origin: DVec3, ray: Ray) -> Option<DVec3> {
    let mut t = 0.0;
    if !intersect_plane(
        plane_normal,
        plane_origin,
        ray.origin,
        ray.direction,
        &mut t,
    ) {
        None
    } else {
        Some(ray.origin + ray.direction * t)
    }
}
