use glam::DQuat;

use crate::math::{screen_to_world, Pos2};
use crate::subgizmo::common::{draw_circle, pick_circle};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoKind};
use crate::{config::PreparedGizmoConfig, gizmo::Ray, GizmoDrawData, GizmoMode, GizmoResult};
use ecolor::Color32;

pub(crate) type ArcballSubGizmo = SubGizmoConfig<Arcball>;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ArcballState {
    last_pos: Pos2,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Arcball;

impl SubGizmoKind for Arcball {
    type Params = ();
    type State = ArcballState;
}

impl SubGizmo for ArcballSubGizmo {
    fn pick(&mut self, ray: Ray) -> Option<f64> {
        let pick_result = pick_circle(&self.config, ray, arcball_radius(&self.config), true);
        if !pick_result.picked {
            return None;
        }

        self.state.last_pos = ray.screen_pos;

        Some(pick_result.t)
    }

    fn update(&mut self, ray: Ray) -> Option<GizmoResult> {
        let dir = ray.screen_pos - self.state.last_pos;

        let quat = if dir.length_sq() > f32::EPSILON {
            let mat = self.config.view_projection.inverse();
            let a = screen_to_world(self.config.viewport, mat, ray.screen_pos, 0.0);
            let b = screen_to_world(self.config.viewport, mat, self.state.last_pos, 0.0);

            let origin = self.config.view_forward();
            let a = (a - origin).normalize();
            let b = (b - origin).normalize();

            DQuat::from_axis_angle(a.cross(b).normalize(), a.dot(b).acos() * 10.0)
        } else {
            DQuat::IDENTITY
        };

        self.state.last_pos = ray.screen_pos;

        let new_rotation = quat * self.config.rotation;

        Some(GizmoResult {
            scale: self.config.scale.as_vec3().into(),
            rotation: new_rotation.as_quat().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Rotate,
            value: None,
        })
    }

    fn draw(&self) -> GizmoDrawData {
        draw_circle(
            &self.config,
            Color32::WHITE.gamma_multiply(if self.focused { 0.10 } else { 0.0 }),
            arcball_radius(&self.config),
            true,
        )
    }
}

/// Radius to use for outer circle subgizmos
pub(crate) fn arcball_radius(config: &PreparedGizmoConfig) -> f64 {
    (config.scale_factor * (config.visuals.gizmo_size + config.visuals.stroke_width - 5.0)) as f64
}
