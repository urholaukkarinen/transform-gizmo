use egui::Ui;
use glam::DVec3;

use crate::math::{round_to_interval, world_to_screen};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, inner_circle_radius, outer_circle_radius, pick_arrow,
    pick_circle, pick_plane, plane_binormal, plane_tangent,
};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoState, TransformKind};
use crate::{GizmoDirection, GizmoMode, GizmoResult, Ray};

pub(crate) type ScaleSubGizmo = SubGizmoConfig<ScaleState>;

impl SubGizmo for ScaleSubGizmo {
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64> {
        let pick_result = match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => pick_arrow(self, ray),
            (TransformKind::Plane, GizmoDirection::Screen) => {
                let mut result = pick_circle(self, ray, inner_circle_radius(self), true);
                if !result.picked {
                    result = pick_circle(self, ray, outer_circle_radius(self), false);
                }
                result
            }
            (TransformKind::Plane, _) => pick_plane(self, ray),
        };

        let start_delta = distance_from_origin_2d(self, ui)?;

        self.opacity = pick_result.visibility as _;

        self.update_state_with(ui, |state: &mut ScaleState| {
            state.start_scale = self.config.scale;
            state.start_delta = start_delta;
        });

        if pick_result.picked {
            Some(pick_result.t)
        } else {
            None
        }
    }

    fn update(&mut self, ui: &Ui, _ray: Ray) -> Option<GizmoResult> {
        let state = self.state(ui);
        let mut delta = distance_from_origin_2d(self, ui)?;
        delta /= state.start_delta;

        if self.config.snapping {
            delta = round_to_interval(delta, self.config.snap_scale as f64);
        }
        delta = delta.max(1e-4) - 1.0;

        let direction = match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => self.local_normal(),
            (TransformKind::Plane, GizmoDirection::Screen) => DVec3::ONE,
            (TransformKind::Plane, _) => {
                (plane_binormal(self.direction) + plane_tangent(self.direction)).normalize()
            }
        };

        let offset = DVec3::ONE + (direction * delta);
        let new_scale = state.start_scale * offset;

        Some(GizmoResult {
            scale: new_scale.as_vec3().into(),
            rotation: self.config.rotation.as_f32().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Scale,
            value: offset.as_vec3().to_array(),
        })
    }

    fn draw(&self, ui: &Ui) {
        match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => draw_arrow(self, ui),
            (TransformKind::Plane, GizmoDirection::Screen) => {
                draw_circle(self, ui, inner_circle_radius(self));
                draw_circle(self, ui, outer_circle_radius(self));
            }
            (TransformKind::Plane, _) => draw_plane(self, ui),
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ScaleState {
    start_scale: DVec3,
    start_delta: f64,
}

impl SubGizmoState for ScaleState {}

fn distance_from_origin_2d<T: SubGizmoState>(subgizmo: &SubGizmoConfig<T>, ui: &Ui) -> Option<f64> {
    let cursor_pos = ui.input(|i| i.pointer.hover_pos())?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, DVec3::new(0.0, 0.0, 0.0))?;

    Some(cursor_pos.distance(gizmo_pos) as f64)
}
