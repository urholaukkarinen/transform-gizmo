use glam::DVec3;

use crate::math::{round_to_interval, world_to_screen, Pos2};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, gizmo_color, gizmo_local_normal, inner_circle_radius,
    outer_circle_radius, pick_arrow, pick_circle, pick_plane, plane_bitangent, plane_tangent,
    ArrowheadStyle,
};
use crate::subgizmo::{common::TransformKind, SubGizmoConfig, SubGizmoKind};
use crate::{gizmo::Ray, GizmoDirection, GizmoDrawData, GizmoMode, GizmoResult};

pub(crate) type ScaleSubGizmo = SubGizmoConfig<Scale>;

#[derive(Debug, Copy, Clone, Hash)]
pub(crate) struct ScaleParams {
    pub direction: GizmoDirection,
    pub transform_kind: TransformKind,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ScaleState {
    start_scale: DVec3,
    start_delta: f64,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Scale;

impl SubGizmoKind for Scale {
    type Params = ScaleParams;
    type State = ScaleState;

    fn pick(subgizmo: &mut ScaleSubGizmo, ray: Ray) -> Option<f64> {
        let pick_result = match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Plane, GizmoDirection::View) => {
                let mut result = pick_circle(
                    &subgizmo.config,
                    ray,
                    inner_circle_radius(&subgizmo.config),
                    true,
                );
                if !result.picked {
                    result = pick_circle(
                        &subgizmo.config,
                        ray,
                        outer_circle_radius(&subgizmo.config),
                        false,
                    );
                }
                result
            }
            (TransformKind::Plane, _) => pick_plane(&subgizmo.config, ray, subgizmo.direction),
            (TransformKind::Axis, _) => pick_arrow(&subgizmo.config, ray, subgizmo.direction),
        };

        let start_delta = distance_from_origin_2d(subgizmo, ray.screen_pos)?;

        subgizmo.opacity = pick_result.visibility as _;

        subgizmo.state.start_scale = subgizmo.config.scale;
        subgizmo.state.start_delta = start_delta;

        if pick_result.picked {
            Some(pick_result.t)
        } else {
            None
        }
    }

    fn update(subgizmo: &mut ScaleSubGizmo, ray: Ray) -> Option<GizmoResult> {
        let mut delta = distance_from_origin_2d(subgizmo, ray.screen_pos)?;
        delta /= subgizmo.state.start_delta;

        if subgizmo.config.snapping {
            delta = round_to_interval(delta, subgizmo.config.snap_scale as f64);
        }
        delta = delta.max(1e-4) - 1.0;

        let direction = match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Axis, _) => gizmo_local_normal(&subgizmo.config, subgizmo.direction),
            (TransformKind::Plane, GizmoDirection::View) => DVec3::ONE,
            (TransformKind::Plane, _) => (plane_bitangent(subgizmo.direction)
                + plane_tangent(subgizmo.direction))
            .normalize(),
        };

        let offset = DVec3::ONE + (direction * delta);
        let new_scale = subgizmo.state.start_scale * offset;

        Some(GizmoResult {
            scale: new_scale.as_vec3().into(),
            rotation: subgizmo.config.rotation.as_quat().into(),
            translation: subgizmo.config.translation.as_vec3().into(),
            mode: GizmoMode::Scale,
            value: Some(offset.as_vec3().to_array()),
        })
    }

    fn draw(subgizmo: &ScaleSubGizmo) -> GizmoDrawData {
        match (subgizmo.transform_kind, subgizmo.direction) {
            (TransformKind::Axis, _) => draw_arrow(
                &subgizmo.config,
                subgizmo.opacity,
                subgizmo.focused,
                subgizmo.direction,
                ArrowheadStyle::Square,
            ),
            (TransformKind::Plane, GizmoDirection::View) => {
                draw_circle(
                    &subgizmo.config,
                    gizmo_color(&subgizmo.config, subgizmo.focused, subgizmo.direction),
                    inner_circle_radius(&subgizmo.config),
                    false,
                ) + draw_circle(
                    &subgizmo.config,
                    gizmo_color(&subgizmo.config, subgizmo.focused, subgizmo.direction),
                    outer_circle_radius(&subgizmo.config),
                    false,
                )
            }
            (TransformKind::Plane, _) => draw_plane(
                &subgizmo.config,
                subgizmo.opacity,
                subgizmo.focused,
                subgizmo.direction,
            ),
        }
    }
}

fn distance_from_origin_2d<T: SubGizmoKind>(
    subgizmo: &SubGizmoConfig<T>,
    cursor_pos: Pos2,
) -> Option<f64> {
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, DVec3::new(0.0, 0.0, 0.0))?;

    Some(cursor_pos.distance(gizmo_pos) as f64)
}
