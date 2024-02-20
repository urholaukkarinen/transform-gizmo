use egui::{Color32, Pos2, Ui};
use glam::DQuat;

use crate::math::screen_to_world;
use crate::subgizmo::common::{draw_circle, pick_circle};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoKind};
use crate::{GizmoConfig, GizmoMode, GizmoResult, Ray, WidgetData};

pub(crate) type ArcballSubGizmo = SubGizmoConfig<Arcball>;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ArcballState {
    last_pos: Pos2,
}

impl WidgetData for ArcballState {}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Arcball;

impl SubGizmoKind for Arcball {
    type Params = ();
    type State = ArcballState;
}

impl SubGizmo for ArcballSubGizmo {
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64> {
        let pick_result = pick_circle(self, ray, arcball_radius(&self.config), true);
        if !pick_result.picked {
            return None;
        }

        self.update_state_with(ui, |state: &mut ArcballState| {
            state.last_pos = ray.screen_pos;
        });

        Some(pick_result.t)
    }

    fn update(&mut self, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
        let state = self.state(ui);

        let dir = ray.screen_pos - state.last_pos;

        let quat = if dir.length_sq() > f32::EPSILON {
            let mat = self.config.view_projection.inverse();
            let a = screen_to_world(self.config.viewport, mat, ray.screen_pos, 0.0);
            let b = screen_to_world(self.config.viewport, mat, state.last_pos, 0.0);

            let origin = self.config.view_forward();
            let a = (a - origin).normalize();
            let b = (b - origin).normalize();

            DQuat::from_axis_angle(a.cross(b).normalize(), a.dot(b).acos() * 10.0)
        } else {
            DQuat::IDENTITY
        };

        self.update_state_with(ui, |state: &mut ArcballState| {
            state.last_pos = ray.screen_pos;
        });

        let new_rotation = quat * self.config.rotation;

        Some(GizmoResult {
            scale: self.config.scale.as_vec3().into(),
            rotation: new_rotation.as_quat().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Rotate,
            value: None,
        })
    }

    fn draw(&mut self, ui: &Ui) {
        self.opacity = if self.focused { 0.10 } else { 0.0 };

        draw_circle(self, ui, Color32::WHITE, arcball_radius(&self.config), true);
    }
}

/// Radius to use for outer circle subgizmos
pub(crate) fn arcball_radius(config: &GizmoConfig) -> f64 {
    (config.scale_factor * (config.visuals.gizmo_size + config.visuals.stroke_width - 5.0)) as f64
}
