use crate::math::{ray_to_plane_origin, segment_to_segment};
use egui::{Color32, Stroke, Ui};
use std::ops::RangeInclusive;

use crate::painter::Painter3d;
use crate::subgizmo::{SubGizmoConfig, SubGizmoKind};
use crate::{GizmoConfig, GizmoDirection, Ray};
use glam::{DMat3, DMat4, DQuat, DVec3};

const ARROW_FADE: RangeInclusive<f64> = 0.95..=0.99;
const PLANE_FADE: RangeInclusive<f64> = 0.70..=0.86;

#[derive(Debug, Copy, Clone)]
pub(crate) struct PickResult {
    pub subgizmo_point: DVec3,
    pub visibility: f64,
    pub picked: bool,
    pub t: f64,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum ArrowheadStyle {
    Cone,
    Square,
}

pub(crate) fn pick_arrow<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    ray: Ray,
    direction: GizmoDirection,
) -> PickResult {
    let width = (subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width) as f64;

    let dir = gizmo_normal(&subgizmo.config, direction);
    let start = subgizmo.config.translation
        + (dir * (width.mul_add(0.5, inner_circle_radius(&subgizmo.config))));

    let length = (subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size) as f64;

    let ray_length = 1e+14;

    let (ray_t, subgizmo_t) = segment_to_segment(
        ray.origin,
        ray.origin + ray.direction * ray_length,
        start,
        start + dir * length,
    );

    let ray_point = ray.origin + ray.direction * ray_length * ray_t;
    let subgizmo_point = start + dir * length * subgizmo_t;
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

pub(crate) fn pick_plane<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    ray: Ray,
    direction: GizmoDirection,
) -> PickResult {
    let origin = plane_global_origin(&subgizmo.config, direction);

    let normal = gizmo_normal(&subgizmo.config, direction);

    let (t, dist_from_origin) = ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let ray_point = ray.origin + ray.direction * t;

    let dot = subgizmo
        .config
        .gizmo_view_forward
        .dot(gizmo_normal(&subgizmo.config, direction))
        .abs();
    let visibility = (1.0
        - ((1.0 - dot) - *PLANE_FADE.start()) / (*PLANE_FADE.end() - *PLANE_FADE.start()))
    .min(1.0);

    let picked = visibility > 0.0 && dist_from_origin <= plane_size(&subgizmo.config);

    PickResult {
        subgizmo_point: ray_point,
        visibility,
        picked,
        t,
    }
}

pub(crate) fn pick_circle<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    ray: Ray,
    radius: f64,
    filled: bool,
) -> PickResult {
    let config = &subgizmo.config;
    let origin = config.translation;
    let normal = -subgizmo.config.view_forward();

    let (t, dist_from_gizmo_origin) =
        ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let hit_pos = ray.origin + ray.direction * t;

    let picked = if filled {
        dist_from_gizmo_origin <= radius + config.focus_distance as f64
    } else {
        (dist_from_gizmo_origin - radius).abs() <= config.focus_distance as f64
    };

    PickResult {
        subgizmo_point: hit_pos,
        visibility: 1.0,
        picked,
        t,
    }
}

pub(crate) fn draw_arrow<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    ui: &Ui,
    direction: GizmoDirection,
    arrowhead_style: ArrowheadStyle,
) {
    if subgizmo.opacity <= 1e-4 {
        return;
    }

    let color = gizmo_color(subgizmo, direction).gamma_multiply(subgizmo.opacity);

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

    let direction = gizmo_local_normal(&subgizmo.config, direction);
    let width = (subgizmo.config.scale_factor * subgizmo.config.visuals.stroke_width) as f64;
    let length = (subgizmo.config.scale_factor * subgizmo.config.visuals.gizmo_size) as f64;

    let start = direction * width.mul_add(0.5, inner_circle_radius(&subgizmo.config));
    let end = direction * length;
    painter.line_segment(start, end, (subgizmo.config.visuals.stroke_width, color));

    match arrowhead_style {
        ArrowheadStyle::Square => {
            let end_stroke_width = subgizmo.config.visuals.stroke_width * 2.5;
            let end_length = subgizmo.config.scale_factor * end_stroke_width;

            painter.line_segment(
                end,
                end + direction * end_length as f64,
                (end_stroke_width, color),
            );
        }
        ArrowheadStyle::Cone => {
            let arrow_length = width * 2.4;

            painter.arrow(
                end,
                end + direction * arrow_length,
                (subgizmo.config.visuals.stroke_width * 1.2, color),
            );
        }
    }
}

pub(crate) fn draw_plane<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    ui: &Ui,
    direction: GizmoDirection,
) {
    if subgizmo.opacity <= 1e-4 {
        return;
    }

    let color = gizmo_color(subgizmo, direction).gamma_multiply(subgizmo.opacity);

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

    let scale = plane_size(&subgizmo.config) * 0.5;
    let a = plane_bitangent(direction) * scale;
    let b = plane_tangent(direction) * scale;
    let origin = plane_local_origin(&subgizmo.config, direction);

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

pub(crate) fn draw_circle<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    ui: &Ui,
    color: Color32,
    radius: f64,
    filled: bool,
) {
    if subgizmo.opacity <= 1e-4 {
        return;
    }

    let color = color.gamma_multiply(subgizmo.opacity);

    let rotation = {
        let forward = subgizmo.config.view_forward();
        let right = subgizmo.config.view_right();
        let up = subgizmo.config.view_up();

        DQuat::from_mat3(&DMat3::from_cols(up, -forward, -right))
    };

    let transform = DMat4::from_rotation_translation(rotation, subgizmo.config.translation);

    let painter = Painter3d::new(
        ui.painter().clone(),
        subgizmo.config.view_projection * transform,
        subgizmo.config.viewport,
    );

    if filled {
        painter.filled_circle(radius, color);
    } else {
        painter.circle(radius, (subgizmo.config.visuals.stroke_width, color));
    }
}

pub(crate) const fn plane_bitangent(direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::Y,
        GizmoDirection::Y => DVec3::Z,
        GizmoDirection::Z => DVec3::X,
        GizmoDirection::View => DVec3::ZERO, // Unused
    }
}

pub(crate) const fn plane_tangent(direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::Z,
        GizmoDirection::Y => DVec3::X,
        GizmoDirection::Z => DVec3::Y,
        GizmoDirection::View => DVec3::ZERO, // Unused
    }
}

pub(crate) fn plane_size(config: &GizmoConfig) -> f64 {
    (config.scale_factor
        * config
            .visuals
            .gizmo_size
            .mul_add(0.1, config.visuals.stroke_width * 2.0)) as f64
}

pub(crate) fn plane_local_origin(config: &GizmoConfig, direction: GizmoDirection) -> DVec3 {
    let offset = config.scale_factor * config.visuals.gizmo_size * 0.5;

    let a = plane_bitangent(direction);
    let b = plane_tangent(direction);
    (a + b) * offset as f64
}

pub(crate) fn plane_global_origin(config: &GizmoConfig, direction: GizmoDirection) -> DVec3 {
    let mut origin = plane_local_origin(config, direction);
    if config.local_space() {
        origin = config.rotation * origin;
    }
    origin + config.translation
}

/// Radius to use for inner circle subgizmos
pub(crate) fn inner_circle_radius(config: &GizmoConfig) -> f64 {
    (config.scale_factor * config.visuals.gizmo_size) as f64 * 0.2
}

/// Radius to use for outer circle subgizmos
pub(crate) fn outer_circle_radius(config: &GizmoConfig) -> f64 {
    (config.scale_factor * (config.visuals.gizmo_size + config.visuals.stroke_width + 5.0)) as f64
}

pub(crate) fn gizmo_local_normal(config: &GizmoConfig, direction: GizmoDirection) -> DVec3 {
    match direction {
        GizmoDirection::X => DVec3::X,
        GizmoDirection::Y => DVec3::Y,
        GizmoDirection::Z => DVec3::Z,
        GizmoDirection::View => -config.view_forward(),
    }
}

pub(crate) fn gizmo_normal(config: &GizmoConfig, direction: GizmoDirection) -> DVec3 {
    let mut normal = gizmo_local_normal(config, direction);

    if config.local_space() && direction != GizmoDirection::View {
        normal = config.rotation * normal;
    }

    normal
}

pub(crate) fn gizmo_color<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    direction: GizmoDirection,
) -> Color32 {
    let color = match direction {
        GizmoDirection::X => subgizmo.config.visuals.x_color,
        GizmoDirection::Y => subgizmo.config.visuals.y_color,
        GizmoDirection::Z => subgizmo.config.visuals.z_color,
        GizmoDirection::View => subgizmo.config.visuals.s_color,
    };

    let color = if subgizmo.focused {
        subgizmo.config.visuals.highlight_color.unwrap_or(color)
    } else {
        color
    };

    let alpha = if subgizmo.focused {
        subgizmo.config.visuals.highlight_alpha
    } else {
        subgizmo.config.visuals.inactive_alpha
    };

    color.linear_multiply(alpha)
}
