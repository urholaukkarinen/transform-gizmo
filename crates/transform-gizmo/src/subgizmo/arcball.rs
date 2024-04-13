use crate::math::{screen_to_world, DQuat, Pos2};
use crate::subgizmo::common::{draw_circle, pick_circle};
use crate::subgizmo::{SubGizmoConfig, SubGizmoKind};
use crate::{config::PreparedGizmoConfig, gizmo::Ray, GizmoDrawData, GizmoResult};
use ecolor::Color32;

pub(crate) type ArcballSubGizmo = SubGizmoConfig<Arcball>;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ArcballState {
    last_pos: Pos2,
    total_rotation: DQuat,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Arcball;

impl SubGizmoKind for Arcball {
    type Params = ();
    type State = ArcballState;

    fn pick(subgizmo: &mut ArcballSubGizmo, ray: Ray) -> Option<f64> {
        let pick_result = pick_circle(
            &subgizmo.config,
            ray,
            arcball_radius(&subgizmo.config),
            true,
        );

        subgizmo.state.last_pos = ray.screen_pos;

        if !pick_result.picked {
            return None;
        }

        Some(f64::MAX)
    }

    fn update(subgizmo: &mut ArcballSubGizmo, ray: Ray) -> Option<GizmoResult> {
        let dir = ray.screen_pos - subgizmo.state.last_pos;

        let rotation_delta = if dir.length_sq() > f32::EPSILON {
            let mat = subgizmo.config.view_projection.inverse();
            let a = screen_to_world(subgizmo.config.viewport, mat, ray.screen_pos, 0.0);
            let b = screen_to_world(subgizmo.config.viewport, mat, subgizmo.state.last_pos, 0.0);

            let origin = subgizmo.config.view_forward();
            let a = (a - origin).normalize();
            let b = (b - origin).normalize();

            DQuat::from_axis_angle(a.cross(b).normalize(), a.dot(b).acos() * 10.0)
        } else {
            DQuat::IDENTITY
        };

        subgizmo.state.last_pos = ray.screen_pos;
        subgizmo.state.total_rotation = rotation_delta.mul_quat(subgizmo.state.total_rotation);

        Some(GizmoResult::Arcball {
            delta: rotation_delta.into(),
            total: subgizmo.state.total_rotation.into(),
        })
    }

    fn draw(subgizmo: &ArcballSubGizmo) -> GizmoDrawData {
        draw_circle(
            &subgizmo.config,
            Color32::WHITE.gamma_multiply(if subgizmo.focused { 0.10 } else { 0.0 }),
            arcball_radius(&subgizmo.config),
            true,
        )
    }
}

/// Radius to use for outer circle subgizmos
pub(crate) fn arcball_radius(config: &PreparedGizmoConfig) -> f64 {
    (config.scale_factor * (config.visuals.gizmo_size + config.visuals.stroke_width - 5.0)) as f64
}
