use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::Ui;
use glam::{Mat4, Quat, Vec2, Vec3};

use crate::math::{ray_to_plane_origin, rotation_align, round_to_interval, world_to_screen};
use crate::painter::Painter3d;
use crate::subgizmo::SubGizmo;
use crate::{GizmoDirection, GizmoMode, GizmoResult, Ray, WidgetData};

/// Picks given rotation subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_rotation(subgizmo: &SubGizmo, ui: &Ui, ray: Ray) -> Option<f32> {
    let radius = arc_radius(subgizmo);
    let config = subgizmo.config;
    let origin = config.translation;
    let normal = subgizmo.normal();
    let tangent = tangent(subgizmo);

    let (t, dist_from_gizmo_origin) =
        ray_to_plane_origin(normal, origin, ray.origin, ray.direction);
    let dist_from_gizmo_edge = (dist_from_gizmo_origin - radius).abs();

    let hit_pos = ray.origin + ray.direction * t;
    let dir_to_origin = (origin - hit_pos).normalize();
    let nearest_circle_pos = hit_pos + dir_to_origin * (dist_from_gizmo_origin - radius);

    let offset = (nearest_circle_pos - origin).normalize();

    let angle = if subgizmo.direction == GizmoDirection::Screen {
        f32::atan2(tangent.cross(normal).dot(offset), tangent.dot(offset))
    } else {
        let mut forward = config.view_forward();
        if config.left_handed {
            forward *= -1.0;
        }
        f32::atan2(offset.cross(forward).dot(normal), offset.dot(forward))
    };

    subgizmo.update_state_with(ui, |state: &mut RotationState| {
        let rotation_angle = rotation_angle(subgizmo, ui).unwrap_or(0.0);
        state.start_axis_angle = angle;
        state.start_rotation_angle = rotation_angle;
        state.last_rotation_angle = rotation_angle;
        state.current_delta = 0.0;
    });

    if dist_from_gizmo_edge <= config.focus_distance && angle.abs() < arc_angle(subgizmo) {
        Some(t)
    } else {
        None
    }
}

pub(crate) fn draw_rotation(subgizmo: &SubGizmo, ui: &Ui) {
    let state = subgizmo.state::<RotationState>(ui);
    let config = subgizmo.config;

    let transform = rotation_matrix(subgizmo);
    let painter = Painter3d::new(
        ui.painter().clone(),
        config.view_projection * transform,
        config.viewport,
    );

    let color = subgizmo.color();
    let stroke = (config.visuals.stroke_width, color);

    let radius = arc_radius(subgizmo);

    if !subgizmo.active {
        let angle = arc_angle(subgizmo);
        painter.arc(radius, FRAC_PI_2 - angle, FRAC_PI_2 + angle, stroke);
    } else {
        let start_angle = state.start_axis_angle + FRAC_PI_2;
        let end_angle = start_angle + state.current_delta;

        // The polyline does not get rendered correctly if
        // the start and end lines are exactly the same
        let end_angle = end_angle + 1e-5;

        painter.polyline(
            &[
                Vec3::new(start_angle.cos() * radius, 0.0, start_angle.sin() * radius),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(end_angle.cos() * radius, 0.0, end_angle.sin() * radius),
            ],
            stroke,
        );

        painter.circle(radius, stroke);

        // Draw snapping ticks
        if config.snapping {
            let stroke_width = stroke.0 / 2.0;
            for i in 0..((TAU / config.snap_angle) as usize + 1) {
                let angle = i as f32 * config.snap_angle + end_angle;
                let pos = Vec3::new(angle.cos(), 0.0, angle.sin());
                painter.line_segment(
                    pos * radius * 1.1,
                    pos * radius * 1.2,
                    (stroke_width, stroke.1),
                );
            }
        }
    }
}

/// Updates given rotation subgizmo.
/// If the subgizmo is active, returns the rotation result.
pub(crate) fn update_rotation(subgizmo: &SubGizmo, ui: &Ui, _ray: Ray) -> Option<GizmoResult> {
    let state = subgizmo.state::<RotationState>(ui);
    let config = subgizmo.config;

    let mut rotation_angle = rotation_angle(subgizmo, ui)?;
    if config.snapping {
        rotation_angle = round_to_interval(
            rotation_angle - state.start_rotation_angle,
            config.snap_angle,
        ) + state.start_rotation_angle;
    }

    let mut angle_delta = rotation_angle - state.last_rotation_angle;

    // Always take the smallest angle, e.g. -10° instead of 350°
    if angle_delta > PI {
        angle_delta -= TAU;
    } else if angle_delta < -PI {
        angle_delta += TAU;
    }

    subgizmo.update_state_with(ui, |state: &mut RotationState| {
        state.last_rotation_angle = rotation_angle;
        state.current_delta += angle_delta;
    });

    let new_rotation =
        Quat::from_axis_angle(subgizmo.normal(), -angle_delta) * subgizmo.config.rotation;

    Some(GizmoResult {
        scale: subgizmo.config.scale,
        rotation: new_rotation,
        translation: subgizmo.config.translation,
        mode: GizmoMode::Rotate,
        value: (subgizmo.normal() * state.current_delta).to_array(),
    })
}

/// Calculates angle of the rotation axis arc.
/// The arc is a semicircle, which turns into a full circle when viewed
/// directly from the front.
fn arc_angle(subgizmo: &SubGizmo) -> f32 {
    let dot = subgizmo.normal().dot(subgizmo.config.view_forward()).abs();
    let min_dot = 0.990;
    let max_dot = 0.995;

    f32::min(1.0, f32::max(0.0, dot - min_dot) / (max_dot - min_dot)) * FRAC_PI_2 + FRAC_PI_2
}

/// Calculates a matrix used when rendering the rotation axis.
fn rotation_matrix(subgizmo: &SubGizmo) -> Mat4 {
    // First rotate towards the gizmo normal
    let local_normal = subgizmo.local_normal();
    let rotation = rotation_align(Vec3::Y, local_normal);
    let mut rotation = Quat::from_mat3(&rotation);
    let config = subgizmo.config;

    // TODO optimize this. Use same code for all axes if possible.

    if subgizmo.direction != GizmoDirection::Screen {
        if config.local_space() {
            rotation = config.rotation * rotation;
        }

        let tangent = tangent(subgizmo);
        let normal = subgizmo.normal();
        let mut forward = config.view_forward();
        if config.left_handed {
            forward *= -1.0;
        }
        let angle = f32::atan2(tangent.cross(forward).dot(normal), tangent.dot(forward));

        // Rotate towards the camera, along the rotation axis.
        rotation = Quat::from_axis_angle(normal, angle) * rotation;
    } else {
        let angle = f32::atan2(local_normal.x, local_normal.z) + FRAC_PI_2;
        rotation = Quat::from_axis_angle(local_normal, angle) * rotation;
    }

    Mat4::from_rotation_translation(rotation, config.translation)
}

fn rotation_angle(subgizmo: &SubGizmo, ui: &Ui) -> Option<f32> {
    let cursor_pos = ui.input().pointer.hover_pos()?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, Vec3::new(0.0, 0.0, 0.0))?;
    let delta = Vec2::new(cursor_pos.x - gizmo_pos.x, cursor_pos.y - gizmo_pos.y).normalize();

    if delta.is_nan() {
        return None;
    }

    let mut angle = f32::atan2(delta.y, delta.x);
    if subgizmo.config.view_forward().dot(subgizmo.normal()) < 0.0 {
        angle *= -1.0;
    }

    Some(angle)
}

fn tangent(subgizmo: &SubGizmo) -> Vec3 {
    let mut tangent = match subgizmo.direction {
        GizmoDirection::X => Vec3::Z,
        GizmoDirection::Y => Vec3::Z,
        GizmoDirection::Z => -Vec3::Y,
        GizmoDirection::Screen => -subgizmo.config.view_right(),
    };

    if subgizmo.config.local_space() && subgizmo.direction != GizmoDirection::Screen {
        tangent = subgizmo.config.rotation * tangent;
    }

    tangent
}

fn arc_radius(subgizmo: &SubGizmo) -> f32 {
    let mut radius = subgizmo.config.visuals.gizmo_size;

    if subgizmo.direction == GizmoDirection::Screen {
        // Screen axis should be a little bit larger
        radius += subgizmo.config.visuals.stroke_width + 5.0;
    }

    subgizmo.config.scale_factor * radius
}

#[derive(Default, Debug, Copy, Clone)]
struct RotationState {
    start_axis_angle: f32,
    start_rotation_angle: f32,
    last_rotation_angle: f32,
    current_delta: f32,
}

impl WidgetData for RotationState {}
