use crate::math::{ray_to_plane_origin, segment_to_segment};
use egui::{Stroke, Ui};
use std::ops::RangeInclusive;

use crate::painter::Painter3d;
use crate::subgizmo::scale::ScaleState;
use crate::subgizmo::translation::TranslationState;
use crate::subgizmo::{SubGizmo, SubGizmoKind};
use crate::{GizmoDirection, Ray};
use glam::{DMat4, DVec3};
const ARROW_FADE: RangeInclusive<f64> = 0.95..=0.99;
const PLANE_FADE: RangeInclusive<f64> = 0.70..=0.86;

#[derive(Debug, Copy, Clone)]
pub(crate) struct PickResult {
    pub subgizmo_point: DVec3,
    pub visibility: f64,
    pub picked: bool,
    pub t: f64,
}

pub(crate) fn pick_arrow(subgizmo: &SubGizmo, ray: Ray) -> PickResult {
    let origin = subgizmo.config.translation;
    let dir = subgizmo.normal();
    let length = (subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size) as f64;

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

    let dot = subgizmo.config.gizmo_view_forward.dot(dir).abs();

    let visibility =
        (1.0 - (dot - *ARROW_FADE.start()) / (*ARROW_FADE.end() - *ARROW_FADE.start())).min(1.0);

    let picked = visibility > 0.0 && dist <= subgizmo.config.focus_distance as f64;

    PickResult {
        subgizmo_point,
        visibility,
        picked,
        t: ray_t,
    }
}

pub(crate) fn pick_plane(subgizmo: &SubGizmo, ray: Ray) -> PickResult {
    let origin = plane_global_origin(subgizmo);

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

    let picked = visibility > 0.0 && dist_from_origin <= plane_size(subgizmo);

    PickResult {
        subgizmo_point: ray_point,
        visibility,
        picked,
        t,
    }
}

pub(crate) fn draw_arrow(subgizmo: &SubGizmo, ui: &Ui) {
    let visibility = if subgizmo.kind == SubGizmoKind::ScaleVector {
        subgizmo.state::<ScaleState>(ui).visibility
    } else {
        subgizmo.state::<TranslationState>(ui).visibility
    };

    if visibility <= 0.0001 {
        return;
    }

    let color = subgizmo.color().gamma_multiply(visibility);

    let transform = if subgizmo.config.local_space() {
        DMat4::from_rotation_translation(subgizmo.config.rotation, subgizmo.config.translation)
    } else {
        DMat4::from_translation(subgizmo.config.translation)
    };

    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * transform,
        subgizmo.config.viewport,
    );

    let direction = subgizmo.local_normal();
    let width = subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width;
    let length = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;

    let start = direction * width as f64;
    let end = direction * length as f64;
    painter.line_segment(start, end, (subgizmo.config.visuals.stroke_width, color));

    if subgizmo.kind == SubGizmoKind::ScaleVector {
        let end_stroke_width = subgizmo.config.visuals.stroke_width * 2.5;
        let end_length = subgizmo.config.scale_factor * end_stroke_width;

        painter.line_segment(
            end,
            end + direction * end_length as f64,
            (end_stroke_width, color),
        );
    } else {
        let arrow_length = width * 2.4;

        painter.arrow(
            end,
            end + direction * arrow_length as f64,
            (subgizmo.config.visuals.stroke_width * 1.2, color),
        );
    }
}

pub(crate) fn draw_plane(subgizmo: &SubGizmo, ui: &Ui) {
    let visibility = if subgizmo.kind == SubGizmoKind::ScalePlane {
        subgizmo.state::<ScaleState>(ui).visibility
    } else {
        subgizmo.state::<TranslationState>(ui).visibility
    };

    if visibility <= 0.0001 {
        return;
    }

    let color = subgizmo.color().gamma_multiply(visibility);

    let transform = if subgizmo.config.local_space() {
        DMat4::from_rotation_translation(subgizmo.config.rotation, subgizmo.config.translation)
    } else {
        DMat4::from_translation(subgizmo.config.translation)
    };

    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * transform,
        subgizmo.config.viewport,
    );

    let scale = plane_size(subgizmo) * 0.5;
    let a = plane_binormal(subgizmo.direction) * scale;
    let b = plane_tangent(subgizmo.direction) * scale;
    let origin = plane_local_origin(subgizmo);

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

pub(crate) fn plane_binormal(direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::Y,
        GizmoDirection::Y => DVec3::Z,
        GizmoDirection::Z => DVec3::X,
        GizmoDirection::Screen => DVec3::ZERO, // Unused
    }
}

pub(crate) fn plane_tangent(direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::Z,
        GizmoDirection::Y => DVec3::X,
        GizmoDirection::Z => DVec3::Y,
        GizmoDirection::Screen => DVec3::ZERO, // Unused
    }
}

pub(crate) fn plane_size(subgizmo: &SubGizmo) -> f64 {
    (subgizmo.config.scale_factor
        * (subgizmo.config.visuals.gizmo_size * 0.1 + subgizmo.config.visuals.stroke_width * 2.0))
        as f64
}

pub(crate) fn plane_local_origin(subgizmo: &SubGizmo) -> DVec3 {
    let offset = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size * 0.4;

    let a = plane_binormal(subgizmo.direction);
    let b = plane_tangent(subgizmo.direction);
    (a + b) * offset as f64
}

pub(crate) fn plane_global_origin(subgizmo: &SubGizmo) -> DVec3 {
    let mut origin = plane_local_origin(subgizmo);
    if subgizmo.config.local_space() {
        origin = subgizmo.config.rotation * origin;
    }
    origin + subgizmo.config.translation
}
