use std::f64::consts::{FRAC_PI_2, PI, TAU};

use egui::Ui;
use glam::{DMat3, DMat4, DQuat, DVec2, DVec3};

use crate::math::{ray_to_plane_origin, rotation_align, round_to_interval, world_to_screen};
use crate::painter::Painter3d;
use crate::subgizmo::common::{gizmo_color, gizmo_local_normal, gizmo_normal, outer_circle_radius};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoKind};
use crate::{GizmoDirection, GizmoMode, GizmoResult, Ray};

pub(crate) type RotationSubGizmo = SubGizmoConfig<Rotation>;

#[derive(Debug, Copy, Clone)]
pub(crate) struct RotationParams {
    pub direction: GizmoDirection,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct RotationState {
    start_axis_angle: f32,
    start_rotation_angle: f32,
    last_rotation_angle: f32,
    current_delta: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Rotation;

impl SubGizmoKind for Rotation {
    type Params = RotationParams;
    type State = RotationState;
}

impl SubGizmo for RotationSubGizmo {
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64> {
        let radius = arc_radius(self);
        let config = self.config;
        let origin = config.translation;
        let normal = gizmo_normal(&self.config, self.direction);
        let tangent = tangent(self);

        let (t, dist_from_gizmo_origin) =
            ray_to_plane_origin(normal, origin, ray.origin, ray.direction);
        let dist_from_gizmo_edge = (dist_from_gizmo_origin - radius).abs();

        let hit_pos = ray.origin + ray.direction * t;
        let dir_to_origin = (origin - hit_pos).normalize();
        let nearest_circle_pos = hit_pos + dir_to_origin * (dist_from_gizmo_origin - radius);

        let offset = (nearest_circle_pos - origin).normalize();

        let angle = if self.direction == GizmoDirection::View {
            f64::atan2(tangent.cross(normal).dot(offset), tangent.dot(offset))
        } else {
            let mut forward = config.view_forward();
            if config.left_handed {
                forward *= -1.0;
            }
            f64::atan2(offset.cross(forward).dot(normal), offset.dot(forward))
        };

        self.update_state_with(ui, |state: &mut RotationState| {
            let rotation_angle = rotation_angle(self, ui).unwrap_or(0.0);
            state.start_axis_angle = angle as f32;
            state.start_rotation_angle = rotation_angle as f32;
            state.last_rotation_angle = rotation_angle as f32;
            state.current_delta = 0.0;
        });

        if dist_from_gizmo_edge <= config.focus_distance as f64 && angle.abs() < arc_angle(self) {
            Some(t)
        } else {
            None
        }
    }

    fn update(&mut self, ui: &Ui, _ray: Ray) -> Option<GizmoResult> {
        let state = self.state(ui);
        let config = self.config;

        let mut rotation_angle = rotation_angle(self, ui)?;
        if config.snapping {
            rotation_angle = round_to_interval(
                rotation_angle - state.start_rotation_angle as f64,
                config.snap_angle as f64,
            ) + state.start_rotation_angle as f64;
        }

        let mut angle_delta = rotation_angle - state.last_rotation_angle as f64;

        // Always take the smallest angle, e.g. -10° instead of 350°
        if angle_delta > PI {
            angle_delta -= TAU;
        } else if angle_delta < -PI {
            angle_delta += TAU;
        }

        self.update_state_with(ui, |state: &mut RotationState| {
            state.last_rotation_angle = rotation_angle as f32;
            state.current_delta += angle_delta as f32;
        });

        let new_rotation =
            DQuat::from_axis_angle(gizmo_normal(&self.config, self.direction), -angle_delta)
                * self.config.rotation;

        Some(GizmoResult {
            scale: self.config.scale.as_vec3().into(),
            rotation: new_rotation.as_quat().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Rotate,
            value: Some(
                (gizmo_normal(&self.config, self.direction).as_vec3() * state.current_delta)
                    .to_array(),
            ),
        })
    }

    fn draw(&mut self, ui: &Ui) {
        let state = self.state(ui);
        let config = self.config;

        let transform = rotation_matrix(self);
        let painter = Painter3d::new(
            ui.painter().clone(),
            config.view_projection * transform,
            config.viewport,
        );

        let color = gizmo_color(self, self.direction);
        let stroke = (config.visuals.stroke_width, color);

        let radius = arc_radius(self);

        if !self.active {
            let angle = arc_angle(self);
            painter.arc(radius, FRAC_PI_2 - angle, FRAC_PI_2 + angle, stroke);
        } else {
            let start_angle = state.start_axis_angle as f64 + FRAC_PI_2;
            let end_angle = start_angle + state.current_delta as f64;

            // The polyline does not get rendered correctly if
            // the start and end lines are exactly the same
            let end_angle = end_angle + 1e-5;

            painter.polyline(
                &[
                    DVec3::new(start_angle.cos() * radius, 0.0, start_angle.sin() * radius),
                    DVec3::new(0.0, 0.0, 0.0),
                    DVec3::new(end_angle.cos() * radius, 0.0, end_angle.sin() * radius),
                ],
                stroke,
            );

            painter.circle(radius, stroke);

            // Draw snapping ticks
            if config.snapping {
                let stroke_width = stroke.0 / 2.0;
                for i in 0..((TAU / config.snap_angle as f64) as usize + 1) {
                    let angle = (i as f64).mul_add(config.snap_angle as f64, end_angle);
                    let pos = DVec3::new(angle.cos(), 0.0, angle.sin());
                    painter.line_segment(
                        pos * radius * 1.1,
                        pos * radius * 1.2,
                        (stroke_width, stroke.1),
                    );
                }
            }
        }
    }
}

/// Calculates angle of the rotation axis arc.
/// The arc is a semicircle, which turns into a full circle when viewed
/// directly from the front.
fn arc_angle(subgizmo: &SubGizmoConfig<Rotation>) -> f64 {
    let dot = gizmo_normal(&subgizmo.config, subgizmo.direction)
        .dot(subgizmo.config.view_forward())
        .abs();
    let min_dot = 0.990;
    let max_dot = 0.995;

    let mut angle = f64::min(1.0, f64::max(0.0, dot - min_dot) / (max_dot - min_dot))
        .mul_add(FRAC_PI_2, FRAC_PI_2);
    if (angle - PI).abs() < 1e-2 {
        angle = PI;
    }
    angle
}

/// Calculates a matrix used when rendering the rotation axis.
fn rotation_matrix(subgizmo: &SubGizmoConfig<Rotation>) -> DMat4 {
    if subgizmo.direction == GizmoDirection::View {
        let forward = subgizmo.config.view_forward();
        let right = subgizmo.config.view_right();
        let up = subgizmo.config.view_up();

        let rotation = DQuat::from_mat3(&DMat3::from_cols(up, -forward, -right));

        return DMat4::from_rotation_translation(rotation, subgizmo.config.translation);
    }

    // First rotate towards the gizmo normal
    let local_normal = gizmo_local_normal(&subgizmo.config, subgizmo.direction);
    let rotation = rotation_align(DVec3::Y, local_normal);
    let mut rotation = DQuat::from_mat3(&rotation);
    let config = subgizmo.config;

    if config.local_space() {
        rotation = config.rotation * rotation;
    }

    let tangent = tangent(subgizmo);
    let normal = gizmo_normal(&subgizmo.config, subgizmo.direction);
    let mut forward = config.view_forward();
    if config.left_handed {
        forward *= -1.0;
    }
    let angle = f64::atan2(tangent.cross(forward).dot(normal), tangent.dot(forward));

    // Rotate towards the camera, along the rotation axis.
    rotation = DQuat::from_axis_angle(normal, angle) * rotation;

    DMat4::from_rotation_translation(rotation, config.translation)
}

fn rotation_angle(subgizmo: &SubGizmoConfig<Rotation>, ui: &Ui) -> Option<f64> {
    let cursor_pos = ui.input(|i| i.pointer.hover_pos())?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, DVec3::new(0.0, 0.0, 0.0))?;
    let delta = DVec2::new(
        cursor_pos.x as f64 - gizmo_pos.x as f64,
        cursor_pos.y as f64 - gizmo_pos.y as f64,
    )
    .normalize();

    if delta.is_nan() {
        return None;
    }

    let mut angle = f64::atan2(delta.y, delta.x);
    if subgizmo
        .config
        .view_forward()
        .dot(gizmo_normal(&subgizmo.config, subgizmo.direction))
        < 0.0
    {
        angle *= -1.0;
    }

    Some(angle)
}

fn tangent(subgizmo: &SubGizmoConfig<Rotation>) -> DVec3 {
    let mut tangent = match subgizmo.direction {
        GizmoDirection::X | GizmoDirection::Y => DVec3::Z,
        GizmoDirection::Z => -DVec3::Y,
        GizmoDirection::View => -subgizmo.config.view_right(),
    };

    if subgizmo.config.local_space() && subgizmo.direction != GizmoDirection::View {
        tangent = subgizmo.config.rotation * tangent;
    }

    tangent
}

fn arc_radius(subgizmo: &SubGizmoConfig<Rotation>) -> f64 {
    if subgizmo.direction == GizmoDirection::View {
        outer_circle_radius(&subgizmo.config)
    } else {
        (subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size) as f64
    }
}
