use std::f64::consts::{FRAC_PI_2, PI, TAU};

use glam::{DMat3, DMat4, DQuat, DVec2, DVec3};

use crate::math::{ray_to_plane_origin, rotation_align, round_to_interval, world_to_screen, Pos2};
use crate::shape::ShapeBuidler;
use crate::subgizmo::common::{gizmo_color, gizmo_local_normal, gizmo_normal, outer_circle_radius};
use crate::subgizmo::{SubGizmoConfig, SubGizmoKind};
use crate::{gizmo::Ray, GizmoDirection, GizmoDrawData, GizmoMode, GizmoResult};

pub(crate) type RotationSubGizmo = SubGizmoConfig<Rotation>;

#[derive(Debug, Copy, Clone, Hash)]
pub(crate) struct RotationParams {
    pub direction: GizmoDirection,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct RotationState {
    start_axis_angle: f64,
    start_rotation_angle: f64,
    last_rotation_angle: f64,
    current_delta: f64,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Rotation;

impl SubGizmoKind for Rotation {
    type Params = RotationParams;
    type State = RotationState;

    fn pick(subgizmo: &mut RotationSubGizmo, ray: Ray) -> Option<f64> {
        let radius = arc_radius(subgizmo);
        let config = subgizmo.config;
        let origin = config.translation;
        let normal = gizmo_normal(&subgizmo.config, subgizmo.direction);
        let tangent = tangent(subgizmo);

        let (t, dist_from_gizmo_origin) =
            ray_to_plane_origin(normal, origin, ray.origin, ray.direction);
        let dist_from_gizmo_edge = (dist_from_gizmo_origin - radius).abs();

        let hit_pos = ray.origin + ray.direction * t;
        let dir_to_origin = (origin - hit_pos).normalize();
        let nearest_circle_pos = hit_pos + dir_to_origin * (dist_from_gizmo_origin - radius);

        let offset = (nearest_circle_pos - origin).normalize();

        let angle = if subgizmo.direction == GizmoDirection::View {
            f64::atan2(tangent.cross(normal).dot(offset), tangent.dot(offset))
        } else {
            let mut forward = config.view_forward();
            if config.left_handed {
                forward *= -1.0;
            }
            f64::atan2(offset.cross(forward).dot(normal), offset.dot(forward))
        };

        let rotation_angle = rotation_angle(subgizmo, ray.screen_pos).unwrap_or(0.0);
        subgizmo.state.start_axis_angle = angle;
        subgizmo.state.start_rotation_angle = rotation_angle;
        subgizmo.state.last_rotation_angle = rotation_angle;
        subgizmo.state.current_delta = 0.0;

        if dist_from_gizmo_edge <= config.focus_distance as f64 && angle.abs() < arc_angle(subgizmo)
        {
            Some(t)
        } else {
            None
        }
    }

    fn update(subgizmo: &mut RotationSubGizmo, ray: Ray) -> Option<GizmoResult> {
        let config = subgizmo.config;

        let mut rotation_angle = rotation_angle(subgizmo, ray.screen_pos)?;
        if config.snapping {
            rotation_angle = round_to_interval(
                rotation_angle - subgizmo.state.start_rotation_angle,
                config.snap_angle as f64,
            ) + subgizmo.state.start_rotation_angle;
        }

        let mut angle_delta = rotation_angle - subgizmo.state.last_rotation_angle;

        // Always take the smallest angle, e.g. -10° instead of 350°
        if angle_delta > PI {
            angle_delta -= TAU;
        } else if angle_delta < -PI {
            angle_delta += TAU;
        }

        subgizmo.state.last_rotation_angle = rotation_angle;
        subgizmo.state.current_delta += angle_delta;

        let new_rotation = DQuat::from_axis_angle(
            gizmo_normal(&subgizmo.config, subgizmo.direction),
            -angle_delta,
        ) * subgizmo.config.rotation;

        Some(GizmoResult {
            scale: subgizmo.config.scale.into(),
            rotation: new_rotation.into(),
            translation: subgizmo.config.translation.into(),
            mode: GizmoMode::Rotate,
            value: Some(
                (gizmo_normal(&subgizmo.config, subgizmo.direction) * subgizmo.state.current_delta)
                    .to_array(),
            ),
        })
    }

    fn draw(subgizmo: &RotationSubGizmo) -> GizmoDrawData {
        let config = subgizmo.config;

        let transform = rotation_matrix(subgizmo);
        let shape_builder = ShapeBuidler::new(
            config.view_projection * transform,
            config.viewport,
            config.pixels_per_point,
        );

        let color = gizmo_color(&subgizmo.config, subgizmo.focused, subgizmo.direction);
        let stroke = (config.visuals.stroke_width, color);

        let radius = arc_radius(subgizmo);

        let mut draw_data = GizmoDrawData::default();

        if !subgizmo.active {
            let angle = arc_angle(subgizmo);
            draw_data += shape_builder
                .arc(radius, FRAC_PI_2 - angle, FRAC_PI_2 + angle, stroke)
                .into();
        } else {
            let start_angle = subgizmo.state.start_axis_angle + FRAC_PI_2;
            let end_angle = start_angle + subgizmo.state.current_delta;

            // The polyline does not get rendered correctly if
            // the start and end lines are exactly the same
            let end_angle = end_angle + 1e-5;

            draw_data += shape_builder
                .polyline(
                    &[
                        DVec3::new(start_angle.cos() * radius, 0.0, start_angle.sin() * radius),
                        DVec3::new(0.0, 0.0, 0.0),
                        DVec3::new(end_angle.cos() * radius, 0.0, end_angle.sin() * radius),
                    ],
                    stroke,
                )
                .into();

            draw_data += shape_builder.circle(radius, stroke).into();

            // Draw snapping ticks
            if config.snapping {
                let stroke_width = stroke.0 / 2.0;
                for i in 0..((TAU / config.snap_angle as f64) as usize + 1) {
                    let angle = i as f64 * config.snap_angle as f64 + end_angle;
                    let pos = DVec3::new(angle.cos(), 0.0, angle.sin());
                    draw_data += shape_builder
                        .line_segment(
                            pos * radius * 1.1,
                            pos * radius * 1.2,
                            (stroke_width, stroke.1),
                        )
                        .into();
                }
            }
        }

        draw_data
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

    let mut angle =
        f64::min(1.0, f64::max(0.0, dot - min_dot) / (max_dot - min_dot)) * FRAC_PI_2 + FRAC_PI_2;
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

fn rotation_angle(subgizmo: &SubGizmoConfig<Rotation>, cursor_pos: Pos2) -> Option<f64> {
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
