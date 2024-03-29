use glam::DVec3;

use crate::math::{round_to_interval, world_to_screen, Pos2};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, gizmo_color, gizmo_local_normal, inner_circle_radius,
    outer_circle_radius, pick_arrow, pick_circle, pick_plane, plane_bitangent, plane_tangent,
    ArrowheadStyle,
};
use crate::subgizmo::{common::TransformKind, SubGizmo, SubGizmoConfig, SubGizmoKind};
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
}

impl SubGizmo for ScaleSubGizmo {
    fn pick(&mut self, ray: Ray) -> Option<f64> {
        let pick_result = match (self.transform_kind, self.direction) {
            (TransformKind::Plane, GizmoDirection::View) => {
                let mut result =
                    pick_circle(&self.config, ray, inner_circle_radius(&self.config), true);
                if !result.picked {
                    result =
                        pick_circle(&self.config, ray, outer_circle_radius(&self.config), false);
                }
                result
            }
            (TransformKind::Plane, _) => pick_plane(&self.config, ray, self.direction),
            (TransformKind::Axis, _) => pick_arrow(&self.config, ray, self.direction),
        };

        let start_delta = distance_from_origin_2d(self, ray.screen_pos)?;

        self.opacity = pick_result.visibility as _;

        self.state.start_scale = self.config.scale;
        self.state.start_delta = start_delta;

        if pick_result.picked {
            Some(pick_result.t)
        } else {
            None
        }
    }

    fn update(&mut self, ray: Ray) -> Option<GizmoResult> {
        let mut delta = distance_from_origin_2d(self, ray.screen_pos)?;
        delta /= self.state.start_delta;

        if self.config.snapping {
            delta = round_to_interval(delta, self.config.snap_scale as f64);
        }
        delta = delta.max(1e-4) - 1.0;

        let direction = match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => gizmo_local_normal(&self.config, self.direction),
            (TransformKind::Plane, GizmoDirection::View) => DVec3::ONE,
            (TransformKind::Plane, _) => {
                (plane_bitangent(self.direction) + plane_tangent(self.direction)).normalize()
            }
        };

        let offset = DVec3::ONE + (direction * delta);
        let new_scale = self.state.start_scale * offset;

        Some(GizmoResult {
            scale: new_scale.as_vec3().into(),
            rotation: self.config.rotation.as_quat().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Scale,
            value: Some(offset.as_vec3().to_array()),
        })
    }

    fn draw(&self) -> GizmoDrawData {
        match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => draw_arrow(
                &self.config,
                self.opacity,
                self.focused,
                self.direction,
                ArrowheadStyle::Square,
            ),
            (TransformKind::Plane, GizmoDirection::View) => {
                draw_circle(
                    &self.config,
                    gizmo_color(&self.config, self.focused, self.direction),
                    inner_circle_radius(&self.config),
                    false,
                ) + draw_circle(
                    &self.config,
                    gizmo_color(&self.config, self.focused, self.direction),
                    outer_circle_radius(&self.config),
                    false,
                )
            }
            (TransformKind::Plane, _) => {
                draw_plane(&self.config, self.opacity, self.focused, self.direction)
            }
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
