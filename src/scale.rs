use egui::{Color32, Stroke, Ui};
use glam::{DMat4, DVec3};
use std::ops::RangeInclusive;

use crate::math::{ray_to_plane_origin, round_to_interval, segment_to_segment, world_to_screen};
use crate::painter::Painter3d;
use crate::subgizmo::SubGizmo;
use crate::translation::{
    translation_plane_binormal, translation_plane_local_origin, translation_plane_size,
    translation_plane_tangent,
};
use crate::{GizmoMode, GizmoResult, Ray, WidgetData};

const ARROW_FADE: RangeInclusive<f64> = (0.95)..=(0.99);
const PLANE_FADE: RangeInclusive<f64> = (0.70)..=(0.86);

/// Picks given scale subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_scale(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f64> {
    let origin = subgizmo.config.translation;
    let dir = subgizmo.config.rotation * subgizmo.local_normal();
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

    let start_delta = distance_from_origin_2d(subgizmo, ui)?;

    let dot = subgizmo
        .config
        .gizmo_view_forward
        .dot(subgizmo.normal())
        .abs();
    let visibility =
        (1.0 - (dot - *ARROW_FADE.start()) / (*ARROW_FADE.end() - *ARROW_FADE.start())).min(1.0);

    subgizmo.update_state_with(ui, |state: &mut ScaleState| {
        state.start_scale = subgizmo.config.scale;
        state.start_delta = start_delta;
        state.visibility = visibility;
    });

    if visibility > 0.0 && dist <= subgizmo.config.focus_distance as f64 {
        Some(ray.origin.distance(ray_point))
    } else {
        None
    }
}

pub(crate) fn draw_scale(subgizmo: &SubGizmo, ui: &Ui) {
    let state = subgizmo.state::<ScaleState>(ui);

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
        subgizmo.config.view_projection * scale_transform(subgizmo),
        subgizmo.config.viewport,
    );

    let direction = subgizmo.local_normal();

    let width = subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width;
    let length = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let end_stroke_width = subgizmo.config.visuals.stroke_width * 2.5;
    let end_length = subgizmo.config.scale_factor * end_stroke_width;
    let length = length - end_length;

    let start = direction * width as f64;
    let end = direction * length as f64;

    painter.line_segment(start, end, (subgizmo.config.visuals.stroke_width, color));
    painter.line_segment(
        end,
        end + direction * end_length as f64,
        (end_stroke_width, color),
    );
}

/// Updates given scale subgizmo.
/// If the subgizmo is active, returns the scale result.
pub(crate) fn update_scale(subgizmo: &SubGizmo, ui: &Ui, _ray: Ray) -> Option<GizmoResult> {
    let state = subgizmo.state::<ScaleState>(ui);

    let mut delta = distance_from_origin_2d(subgizmo, ui)?;
    delta /= state.start_delta;

    if subgizmo.config.snapping {
        delta = round_to_interval(delta, subgizmo.config.snap_scale as f64);
    }
    delta = delta.max(1e-4) - 1.0;

    let offset = DVec3::ONE + (subgizmo.local_normal() * delta);

    let new_scale = state.start_scale * offset;

    Some(GizmoResult {
        scale: new_scale.as_vec3().into(),
        rotation: subgizmo.config.rotation.as_f32().into(),
        translation: subgizmo.config.translation.as_vec3().into(),
        mode: GizmoMode::Scale,
        value: offset.as_vec3().to_array(),
    })
}

/// Picks given scale plane subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_scale_plane(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f64> {
    let origin = scale_plane_global_origin(subgizmo);

    let normal = subgizmo.normal();

    let (t, dist_from_origin) = ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let start_delta = distance_from_origin_2d(subgizmo, ui)?;

    let dot = subgizmo
        .config
        .gizmo_view_forward
        .dot(subgizmo.normal())
        .abs();
    let visibility = (1.0
        - ((1.0 - dot) - *PLANE_FADE.start()) / (*PLANE_FADE.end() - *PLANE_FADE.start()))
    .min(1.0);

    subgizmo.update_state_with(ui, |state: &mut ScaleState| {
        state.start_scale = subgizmo.config.scale;
        state.start_delta = start_delta;
        state.visibility = visibility;
    });

    if visibility > 0.0 && dist_from_origin <= translation_plane_size(subgizmo) {
        Some(t)
    } else {
        None
    }
}

/// Updates given scale plane subgizmo.
/// If the subgizmo is active, returns the scale result.
pub(crate) fn update_scale_plane(subgizmo: &SubGizmo, ui: &Ui, _ray: Ray) -> Option<GizmoResult> {
    let state = subgizmo.state::<ScaleState>(ui);

    let mut delta = distance_from_origin_2d(subgizmo, ui)?;
    delta /= state.start_delta;

    if subgizmo.config.snapping {
        delta = round_to_interval(delta, subgizmo.config.snap_scale as f64);
    }
    delta = delta.max(1e-4) - 1.0;

    let binormal = translation_plane_binormal(subgizmo.direction);
    let tangent = translation_plane_tangent(subgizmo.direction);
    let direction = (binormal + tangent).normalize();

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

pub(crate) fn draw_scale_plane(subgizmo: &SubGizmo, ui: &Ui) {
    let state = subgizmo.state::<ScaleState>(ui);

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
        subgizmo.config.view_projection * scale_transform(subgizmo),
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

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ScaleState {
    start_scale: DVec3,
    start_delta: f64,
    visibility: f64,
}

impl WidgetData for ScaleState {}

fn scale_transform(subgizmo: &SubGizmo) -> DMat4 {
    DMat4::from_rotation_translation(subgizmo.config.rotation, subgizmo.config.translation)
}

pub(crate) fn scale_plane_global_origin(subgizmo: &SubGizmo) -> DVec3 {
    let origin = translation_plane_local_origin(subgizmo);
    subgizmo.config.rotation * origin + subgizmo.config.translation
}

fn distance_from_origin_2d(subgizmo: &SubGizmo, ui: &Ui) -> Option<f64> {
    let cursor_pos = ui.input(|i| i.pointer.hover_pos())?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, DVec3::new(0.0, 0.0, 0.0))?;

    Some(cursor_pos.distance(gizmo_pos) as f64)
}
