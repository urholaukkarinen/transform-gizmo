use egui::{Stroke, Ui};
use glam::{Mat4, Vec3};

use crate::math::{ray_to_plane_origin, round_to_interval, segment_to_segment, world_to_screen};
use crate::painter::Painter3d;
use crate::subgizmo::SubGizmo;
use crate::translation::{
    translation_plane_binormal, translation_plane_local_origin, translation_plane_size,
    translation_plane_tangent,
};
use crate::{GizmoMode, GizmoResult, Ray, WidgetData};

/// Picks given scale subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_scale(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f32> {
    let origin = subgizmo.config.translation;
    let dir = subgizmo.config.rotation * subgizmo.local_normal();
    let scale = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let length = scale;
    let ray_length = 1e+5;

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

    subgizmo.update_state_with(ui, |state: &mut ScaleState| {
        state.start_scale = subgizmo.config.scale;
        state.start_delta = start_delta;
    });

    if dist <= subgizmo.config.focus_distance {
        Some(ray.origin.distance(ray_point))
    } else {
        None
    }
}

pub(crate) fn draw_scale(subgizmo: &SubGizmo, ui: &Ui) {
    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * scale_transform(subgizmo),
        subgizmo.config.viewport,
    );

    let direction = subgizmo.local_normal();

    let color = subgizmo.color();

    let width = subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width;
    let length = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let end_stroke_width = subgizmo.config.visuals.stroke_width * 2.5;
    let end_length = subgizmo.config.scale_factor * end_stroke_width;
    let length = length - end_length;

    let start = direction * width;
    let end = direction * length;

    painter.line_segment(start, end, (subgizmo.config.visuals.stroke_width, color));
    painter.line_segment(end, end + direction * end_length, (end_stroke_width, color));
}

/// Updates given scale subgizmo.
/// If the subgizmo is active, returns the scale result.
pub(crate) fn update_scale(subgizmo: &SubGizmo, ui: &Ui, _ray: Ray) -> Option<GizmoResult> {
    let state = subgizmo.state::<ScaleState>(ui);

    let mut delta = distance_from_origin_2d(subgizmo, ui)?;
    delta /= state.start_delta;

    if subgizmo.config.snapping {
        delta = round_to_interval(delta, subgizmo.config.snap_scale);
    }
    delta = delta.max(1e-4) - 1.0;

    let offset = Vec3::ONE + (subgizmo.local_normal() * delta);

    let new_scale = state.start_scale * offset;

    Some(GizmoResult {
        scale: new_scale.into(),
        rotation: subgizmo.config.rotation.into(),
        translation: subgizmo.config.translation.into(),
        mode: GizmoMode::Scale,
        value: offset.to_array(),
    })
}

/// Picks given scale plane subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_scale_plane(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f32> {
    let origin = scale_plane_global_origin(subgizmo);

    let normal = subgizmo.normal();

    let (t, dist_from_origin) = ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let start_delta = distance_from_origin_2d(subgizmo, ui)?;

    subgizmo.update_state_with(ui, |state: &mut ScaleState| {
        state.start_scale = subgizmo.config.scale;
        state.start_delta = start_delta;
    });

    if dist_from_origin <= translation_plane_size(subgizmo) {
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
        delta = round_to_interval(delta, subgizmo.config.snap_scale);
    }
    delta = delta.max(1e-4) - 1.0;

    let binormal = translation_plane_binormal(subgizmo.direction);
    let tangent = translation_plane_tangent(subgizmo.direction);
    let direction = (binormal + tangent).normalize();

    let offset = Vec3::ONE + (direction * delta);

    let new_scale = state.start_scale * offset;

    Some(GizmoResult {
        scale: new_scale.into(),
        rotation: subgizmo.config.rotation.into(),
        translation: subgizmo.config.translation.into(),
        mode: GizmoMode::Scale,
        value: offset.to_array(),
    })
}

pub(crate) fn draw_scale_plane(subgizmo: &SubGizmo, ui: &Ui) {
    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * scale_transform(subgizmo),
        subgizmo.config.viewport,
    );

    let color = subgizmo.color();

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
    start_scale: Vec3,
    start_delta: f32,
}

impl WidgetData for ScaleState {}

fn scale_transform(subgizmo: &SubGizmo) -> Mat4 {
    Mat4::from_rotation_translation(subgizmo.config.rotation, subgizmo.config.translation)
}

pub(crate) fn scale_plane_global_origin(subgizmo: &SubGizmo) -> Vec3 {
    let origin = translation_plane_local_origin(subgizmo);
    subgizmo.config.rotation * origin + subgizmo.config.translation
}

fn distance_from_origin_2d(subgizmo: &SubGizmo, ui: &Ui) -> Option<f32> {
    let cursor_pos = ui.input(|i| i.pointer.hover_pos())?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, Vec3::new(0.0, 0.0, 0.0))?;

    Some(cursor_pos.distance(gizmo_pos))
}
