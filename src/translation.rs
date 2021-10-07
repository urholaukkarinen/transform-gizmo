use egui::{Color32, PointerButton, Response};
use glam::{Mat4, Vec3};

use crate::math::{ray_to_ray, round_to_interval, segment_to_segment};
use crate::painter::Painter3d;
use crate::subgizmo::SubGizmo;
use crate::{GizmoMode, GizmoResult, Ray};

/// Picks given translation subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_translation(subgizmo: &SubGizmo, ray: Ray) -> Option<f32> {
    let origin = subgizmo.config.translation;
    let dir = subgizmo.normal();
    let scale = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let length = scale;
    let ray_length = 10000.0;

    let (ray_t, subgizmo_t) = segment_to_segment(
        ray.origin,
        ray.origin + ray.direction * ray_length,
        origin,
        origin + dir * length,
    );

    let ray_point = ray.origin + ray.direction * ray_length * ray_t;
    let subgizmo_point = origin + dir * length * subgizmo_t;
    let dist = (ray_point - subgizmo_point).length();

    subgizmo.update_state_with(|state| {
        state.focused = false;
        state.translation.start_point = subgizmo_point;
        state.translation.last_point = subgizmo_point;
        state.translation.current_delta = Vec3::ZERO;
    });

    if dist <= subgizmo.config.focus_distance {
        Some(ray.origin.distance(ray_point))
    } else {
        None
    }
}

pub(crate) fn draw_translation(subgizmo: &SubGizmo) {
    let transform = if subgizmo.config.local_space() {
        Mat4::from_rotation_translation(subgizmo.config.rotation, subgizmo.config.translation)
    } else {
        Mat4::from_translation(subgizmo.config.translation)
    };

    let painter = Painter3d::new(
        subgizmo.ui.painter().clone(),
        subgizmo.config.view_projection * transform,
        subgizmo.config.viewport,
    );

    let direction = subgizmo.local_normal();

    let state = subgizmo.state();

    let color = if state.focused {
        subgizmo
            .config
            .visuals
            .highlight_color
            .unwrap_or_else(|| subgizmo.color())
    } else {
        subgizmo.color()
    };

    let alpha = if state.focused {
        subgizmo.config.visuals.highlight_alpha
    } else {
        subgizmo.config.visuals.inactive_alpha
    };

    let color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);

    let width = subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width;
    let length = subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size;
    let arrow_half_width = width * 2.0;
    let arrow_length = arrow_half_width + length * 0.1;
    let length = length - arrow_length;

    let start = direction * width;
    let end = direction * length;

    painter.line_segment(start, end, (subgizmo.config.visuals.stroke_width, color));

    let cross = direction.cross(subgizmo.config.view_forward()) * arrow_half_width;

    painter.polygon(
        &[end + cross, end - cross, end + (direction * arrow_length)],
        color,
        (0.0, color),
    );
}

/// Updates given translation subgizmo.
/// If the subgizmo is active, returns the translation result.
pub(crate) fn update_translation(
    subgizmo: &SubGizmo,
    ray: Ray,
    interaction: &Response,
) -> Option<GizmoResult> {
    let state = subgizmo.state();

    let mut new_point = point_on_axis(subgizmo, ray);
    let mut new_delta = new_point - state.translation.start_point;

    let delta_length = new_delta.length();
    if subgizmo.config.snapping && delta_length > 1e-5 {
        new_delta = new_delta / delta_length
            * round_to_interval(delta_length, subgizmo.config.snap_distance);
        new_point = state.translation.start_point + new_delta;
    }

    let (scale, rotation, mut translation) =
        subgizmo.config.model_matrix.to_scale_rotation_translation();
    translation += new_point - state.translation.last_point;

    subgizmo.update_state_with(|state| {
        state.active = interaction.dragged_by(PointerButton::Primary);
        state.translation.last_point = new_point;
        state.translation.current_delta = new_delta;
    });

    Some(GizmoResult {
        transform: Mat4::from_scale_rotation_translation(scale, rotation, translation)
            .to_cols_array_2d(),
        mode: GizmoMode::Translate,
        value: state.translation.current_delta.to_array(),
    })
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct TranslationState {
    start_point: Vec3,
    last_point: Vec3,
    current_delta: Vec3,
}

/// Finds the nearest point on line that points in translation subgizmo direction
fn point_on_axis(subgizmo: &SubGizmo, ray: Ray) -> Vec3 {
    let origin = subgizmo.config.translation;
    let direction = subgizmo.normal();

    let (_ray_t, subgizmo_t) = ray_to_ray(ray.origin, ray.direction, origin, direction);

    origin + direction * subgizmo_t
}
