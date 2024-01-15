use egui::{Color32, Stroke, Ui};
use glam::{DMat4, DVec3};
use std::ops::RangeInclusive;

use crate::math::{
    intersect_plane, ray_to_plane_origin, ray_to_ray, round_to_interval, segment_to_segment,
};
use crate::painter::Painter3d;
use crate::subgizmo::SubGizmo;
use crate::{GizmoDirection, GizmoMode, GizmoResult, Ray, WidgetData};

const ARROW_FADE: RangeInclusive<f64> = (0.95)..=(0.99);
const PLANE_FADE: RangeInclusive<f64> = (0.70)..=(0.86);

/// Picks given translation subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_translation(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f64> {
    let origin = subgizmo.config.translation;
    let dir = subgizmo.normal();
    let scale = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let length = scale as f64;

    let ray_length = 1e+14;

    let (ray_t, subgizmo_t) = segment_to_segment(
        ray.origin,
        ray.origin + ray.direction * ray_length,
        origin,
        origin + dir * length,
    );

    let ray_point = ray.origin + ray.direction * ray_length * ray_t;
    let subgizmo_point = origin + dir * length * subgizmo_t;
    let dist = (ray_point - subgizmo_point).length();

    let dot = subgizmo
        .config
        .gizmo_view_forward
        .dot(subgizmo.normal())
        .abs();
    let visibility =
        (1.0 - (dot - *ARROW_FADE.start()) / (*ARROW_FADE.end() - *ARROW_FADE.start())).min(1.0);

    subgizmo.update_state_with(ui, |state: &mut TranslationState| {
        state.start_point = subgizmo_point;
        state.last_point = subgizmo_point;
        state.current_delta = DVec3::ZERO;
        state.visibility = visibility;
    });

    if visibility > 0.0 && dist <= subgizmo.config.focus_distance as f64 {
        Some(ray.origin.distance(ray_point))
    } else {
        None
    }
}

pub(crate) fn draw_translation(subgizmo: &SubGizmo, ui: &Ui) {
    let state = subgizmo.state::<TranslationState>(ui);

    if state.visibility <= 0.0001 {
        return;
    }

    let mut color = subgizmo.color();
    if state.visibility < 1.0 {
        color = Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (state.visibility * 255.0) as u8,
        );
    }

    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * translation_transform(subgizmo),
        subgizmo.config.viewport,
    );

    let direction = subgizmo.local_normal();

    let width = subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width;
    let length = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let arrow_length = width * 2.4;
    let length = length - arrow_length;

    let start = direction * width as f64;
    let end = direction * length as f64;

    painter.line_segment(start, end, (subgizmo.config.visuals.stroke_width, color));
    painter.arrow(
        end,
        end + direction * arrow_length as f64,
        (subgizmo.config.visuals.stroke_width * 1.2, color),
    );
}

/// Updates given translation subgizmo.
/// If the subgizmo is active, returns the translation result.
pub(crate) fn update_translation(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
    let state = subgizmo.state::<TranslationState>(ui);

    let mut new_point = point_on_axis(subgizmo, ray);
    let mut new_delta = new_point - state.start_point;

    if subgizmo.config.snapping {
        new_delta = snap_translation_vector(subgizmo, new_delta);
        new_point = state.start_point + new_delta;
    }

    subgizmo.update_state_with(ui, |state: &mut TranslationState| {
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

/// Picks given translation plane subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_translation_plane(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f64> {
    let origin = translation_plane_global_origin(subgizmo);

    let normal = subgizmo.normal();

    let (t, dist_from_origin) = ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let ray_point = ray.origin + ray.direction * t;

    let dot = subgizmo
        .config
        .gizmo_view_forward
        .dot(subgizmo.normal())
        .abs();
    let visibility = (1.0
        - ((1.0 - dot) - *PLANE_FADE.start()) / (*PLANE_FADE.end() - *PLANE_FADE.start()))
    .min(1.0);

    subgizmo.update_state_with(ui, |state: &mut TranslationState| {
        state.start_point = ray_point;
        state.last_point = ray_point;
        state.current_delta = DVec3::ZERO;
        state.visibility = visibility;
    });

    if visibility > 0.0 && dist_from_origin <= translation_plane_size(subgizmo) {
        Some(t)
    } else {
        None
    }
}

pub(crate) fn draw_translation_plane(subgizmo: &SubGizmo, ui: &Ui) {
    let state = subgizmo.state::<TranslationState>(ui);

    if state.visibility <= 0.0001 {
        return;
    }

    let mut color = subgizmo.color();
    if state.visibility < 1.0 {
        color = Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (state.visibility * 255.0) as u8,
        );
    }

    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * translation_transform(subgizmo),
        subgizmo.config.viewport,
    );

    let scale = translation_plane_size(subgizmo) * 0.5;
    let a = translation_plane_binormal(subgizmo.direction) * scale;
    let b = translation_plane_tangent(subgizmo.direction) * scale;

    let origin = translation_plane_local_origin(subgizmo);

    painter.polygon(
        &[
            origin - b - a,
            origin + b - a,
            origin + b + a,
            origin - b + a,
        ],
        color,
        Stroke::NONE,
    );
}

/// Updates given translation subgizmo.
/// If the subgizmo is active, returns the translation result.
pub(crate) fn update_translation_plane(
    subgizmo: &SubGizmo,
    ui: &Ui,
    ray: Ray,
) -> Option<GizmoResult> {
    let state = subgizmo.state::<TranslationState>(ui);

    let mut new_point = point_on_plane(
        subgizmo.normal(),
        translation_plane_global_origin(subgizmo),
        ray,
    )?;
    let mut new_delta = new_point - state.start_point;

    if subgizmo.config.snapping {
        new_delta = snap_translation_plane(subgizmo, new_delta);
        new_point = state.start_point + new_delta;
    }

    subgizmo.update_state_with(ui, |state: &mut TranslationState| {
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

fn snap_translation_plane(subgizmo: &SubGizmo, new_delta: DVec3) -> DVec3 {
    let mut binormal = translation_plane_binormal(subgizmo.direction);
    let mut tangent = translation_plane_tangent(subgizmo.direction);
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
    visibility: f64,
}

impl WidgetData for TranslationState {}

fn translation_transform(subgizmo: &SubGizmo) -> DMat4 {
    if subgizmo.config.local_space() {
        DMat4::from_rotation_translation(subgizmo.config.rotation, subgizmo.config.translation)
    } else {
        DMat4::from_translation(subgizmo.config.translation)
    }
}

pub(crate) fn translation_plane_binormal(direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::Y,
        GizmoDirection::Y => DVec3::Z,
        GizmoDirection::Z => DVec3::X,
        GizmoDirection::Screen => DVec3::X, // Unused
    }
}

pub(crate) fn translation_plane_tangent(direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::Z,
        GizmoDirection::Y => DVec3::X,
        GizmoDirection::Z => DVec3::Y,
        GizmoDirection::Screen => DVec3::X, // Unused
    }
}

pub(crate) fn translation_plane_size(subgizmo: &SubGizmo) -> f64 {
    (subgizmo.config.scale_factor
        * (subgizmo.config.visuals.gizmo_size * 0.1 + subgizmo.config.visuals.stroke_width * 2.0))
        as f64
}

pub(crate) fn translation_plane_local_origin(subgizmo: &SubGizmo) -> DVec3 {
    let offset = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size * 0.4;

    let a = translation_plane_binormal(subgizmo.direction);
    let b = translation_plane_tangent(subgizmo.direction);
    (a + b) * offset as f64
}

pub(crate) fn translation_plane_global_origin(subgizmo: &SubGizmo) -> DVec3 {
    let mut origin = translation_plane_local_origin(subgizmo);
    if subgizmo.config.local_space() {
        origin = subgizmo.config.rotation * origin;
    }
    origin + subgizmo.config.translation
}

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
