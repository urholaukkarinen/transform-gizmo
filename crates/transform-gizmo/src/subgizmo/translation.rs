use crate::math::{intersect_plane, ray_to_ray, round_to_interval, DVec3};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, gizmo_color, gizmo_normal, inner_circle_radius,
    pick_arrow, pick_circle, pick_plane, plane_bitangent, plane_global_origin, plane_tangent,
};
use crate::subgizmo::{common::TransformKind, SubGizmoConfig, SubGizmoKind};
use crate::{gizmo::Ray, GizmoDirection, GizmoDrawData, GizmoMode, GizmoOrientation, GizmoResult};

pub(crate) type TranslationSubGizmo = SubGizmoConfig<Translation>;

#[derive(Debug, Copy, Clone, Hash)]
pub(crate) struct TranslationParams {
    pub mode: GizmoMode,
    pub direction: GizmoDirection,
    pub transform_kind: TransformKind,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct TranslationState {
    start_view_dir: DVec3,
    start_point: DVec3,
    last_point: DVec3,
    current_delta: DVec3,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Translation;

impl SubGizmoKind for Translation {
    type Params = TranslationParams;
    type State = TranslationState;

    fn pick(subgizmo: &mut TranslationSubGizmo, ray: Ray) -> Option<f64> {
        let pick_result = match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Plane, GizmoDirection::View) => pick_circle(
                &subgizmo.config,
                ray,
                inner_circle_radius(&subgizmo.config),
                true,
            ),
            (TransformKind::Plane, _) => pick_plane(&subgizmo.config, ray, subgizmo.direction),
            (TransformKind::Axis, _) => {
                pick_arrow(&subgizmo.config, ray, subgizmo.direction, subgizmo.mode)
            }
        };

        subgizmo.opacity = pick_result.visibility as _;

        subgizmo.state.start_view_dir = subgizmo.config.view_forward();
        subgizmo.state.start_point = pick_result.subgizmo_point;
        subgizmo.state.last_point = pick_result.subgizmo_point;
        subgizmo.state.current_delta = DVec3::ZERO;

        if pick_result.picked {
            Some(pick_result.t)
        } else {
            None
        }
    }

    fn update(subgizmo: &mut TranslationSubGizmo, ray: Ray) -> Option<GizmoResult> {
        if subgizmo.config.view_forward() != subgizmo.state.start_view_dir {
            // If the view_forward direction has changed, i.e. camera has rotated,
            // refresh the subgizmo state by calling pick. Feels a bit hacky, but
            // fixes the issue where the target starts flying away if camera is rotated
            // while view plane translation is active.
            Self::pick(subgizmo, ray);
        }

        let mut new_point = if subgizmo.transform_kind == TransformKind::Axis {
            point_on_axis(subgizmo, ray)
        } else {
            point_on_plane(
                gizmo_normal(&subgizmo.config, subgizmo.direction),
                plane_global_origin(&subgizmo.config, subgizmo.direction),
                ray,
            )?
        };

        let mut new_delta = new_point - subgizmo.state.start_point;

        if subgizmo.config.snapping {
            new_delta = if subgizmo.transform_kind == TransformKind::Axis {
                snap_translation_vector(subgizmo, new_delta)
            } else {
                snap_translation_plane(subgizmo, new_delta)
            };
            new_point = subgizmo.state.start_point + new_delta;
        }

        let mut translation_delta = new_point - subgizmo.state.last_point;
        let mut total_translation = new_point - subgizmo.state.start_point;

        if subgizmo.config.orientation() == GizmoOrientation::Local {
            let inverse_rotation = subgizmo.config.rotation.inverse();
            translation_delta = inverse_rotation * translation_delta;
            total_translation = inverse_rotation * total_translation;
        }

        subgizmo.state.last_point = new_point;
        subgizmo.state.current_delta = new_delta;

        Some(GizmoResult::Translation {
            delta: translation_delta.into(),
            total: total_translation.into(),
        })
    }

    fn draw(subgizmo: &TranslationSubGizmo) -> GizmoDrawData {
        match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Axis, _) => draw_arrow(
                &subgizmo.config,
                subgizmo.opacity,
                subgizmo.focused,
                subgizmo.direction,
                subgizmo.mode,
            ),
            (TransformKind::Plane, GizmoDirection::View) => draw_circle(
                &subgizmo.config,
                gizmo_color(&subgizmo.config, subgizmo.focused, subgizmo.direction),
                inner_circle_radius(&subgizmo.config),
                false,
            ),
            (TransformKind::Plane, _) => draw_plane(
                &subgizmo.config,
                subgizmo.opacity,
                subgizmo.focused,
                subgizmo.direction,
            ),
        }
    }
}

/// Finds the nearest point on line that points in translation subgizmo direction
fn point_on_axis(subgizmo: &SubGizmoConfig<Translation>, ray: Ray) -> DVec3 {
    let origin = subgizmo.config.translation;
    let direction = gizmo_normal(&subgizmo.config, subgizmo.direction);

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

fn snap_translation_vector(subgizmo: &SubGizmoConfig<Translation>, new_delta: DVec3) -> DVec3 {
    let delta_length = new_delta.length();
    if delta_length > 1e-5 {
        new_delta / delta_length
            * round_to_interval(delta_length, subgizmo.config.snap_distance as f64)
    } else {
        new_delta
    }
}

fn snap_translation_plane(subgizmo: &SubGizmoConfig<Translation>, new_delta: DVec3) -> DVec3 {
    let mut bitangent = plane_bitangent(subgizmo.direction);
    let mut tangent = plane_tangent(subgizmo.direction);
    if subgizmo.config.local_space() {
        bitangent = subgizmo.config.rotation * bitangent;
        tangent = subgizmo.config.rotation * tangent;
    }
    let cb = new_delta.cross(-bitangent);
    let ct = new_delta.cross(tangent);
    let lb = cb.length();
    let lt = ct.length();
    let n = gizmo_normal(&subgizmo.config, subgizmo.direction);

    if lb > 1e-5 && lt > 1e-5 {
        bitangent * round_to_interval(lt, subgizmo.config.snap_distance as f64) * (ct / lt).dot(n)
            + tangent
                * round_to_interval(lb, subgizmo.config.snap_distance as f64)
                * (cb / lb).dot(n)
    } else {
        new_delta
    }
}
