use egui::{Pos2, Ui};
use glam::DQuat;

use crate::math::screen_to_world;
use crate::subgizmo::common::{draw_circle, pick_circle};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoState};
use crate::{GizmoMode, GizmoResult, Ray};

pub(crate) type ArcballSubGizmo = SubGizmoConfig<ArcballState>;

impl SubGizmo for ArcballSubGizmo {
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64> {
        let pick_result = pick_circle(self, ray, arcball_radius(self), true);
        if !pick_result.picked {
            return None;
        }

        self.update_state_with(ui, |state: &mut ArcballState| {
            state.last = ray.screen_pos;
        });

        Some(pick_result.t)
    }

    fn update(&mut self, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
        let state = self.state(ui);

        let dir = ray.screen_pos - state.last;

        // Not a typical ArcBall rotation, but instead always rotates the object in the direction of mouse movement

        let quat = if dir.length_sq() > f32::EPSILON {
            let mat = self.config.view_projection.inverse();
            let a = screen_to_world(self.config.viewport, mat, ray.screen_pos, 0.0);
            let b = screen_to_world(self.config.viewport, mat, state.last, 0.0);
            let origin = self.config.view_forward();
            let a = (a - origin).normalize();
            let b = (b - origin).normalize();

            DQuat::from_axis_angle(a.cross(b).normalize(), a.dot(b).acos() * 10.0)
        } else {
            DQuat::IDENTITY
        };

        self.update_state_with(ui, |state: &mut ArcballState| {
            state.last = ray.screen_pos;
        });

        let new_rotation = quat * self.config.rotation;

        Some(GizmoResult {
            scale: self.config.scale.as_vec3().into(),
            rotation: new_rotation.as_f32().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Rotate,
            value: self.normal().as_vec3().to_array(),
        })
    }

    fn draw(&mut self, ui: &Ui) {
        self.opacity = if self.focused { 0.10 } else { 0.0 };

        draw_circle(self, ui, arcball_radius(self), true);
    }
}

/// Radius to use for outer circle subgizmos
pub(crate) fn arcball_radius<T: SubGizmoState>(subgizmo: &SubGizmoConfig<T>) -> f64 {
    (subgizmo.config.scale_factor
        * (subgizmo.config.visuals.gizmo_size + subgizmo.config.visuals.stroke_width - 5.0))
        as f64
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ArcballState {
    last: Pos2,
}

impl SubGizmoState for ArcballState {}
