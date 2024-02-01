use egui::Ui;
use glam::DVec3;

use crate::math::{round_to_interval, world_to_screen};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, gizmo_color, gizmo_local_normal, inner_circle_radius,
    outer_circle_radius, pick_arrow, pick_circle, pick_plane, plane_bitangent, plane_tangent,
    ArrowheadStyle,
};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoKind, TransformKind};
use crate::{GizmoDirection, GizmoMode, GizmoResult, Ray};

pub(crate) type ScaleSubGizmo = SubGizmoConfig<Scale>;

#[derive(Debug, Copy, Clone)]
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
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64> {
        let pick_result = match (self.transform_kind, self.direction) {
            (TransformKind::Plane, GizmoDirection::View) => {
                let mut result = pick_circle(self, ray, inner_circle_radius(&self.config), true);
                if !result.picked {
                    result = pick_circle(self, ray, outer_circle_radius(&self.config), false);
                }
                result
            }
            (TransformKind::Plane, _) => pick_plane(self, ray, self.direction),
            (TransformKind::Axis, _) => pick_arrow(self, ray, self.direction),
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
            (TransformKind::Axis, _) => gizmo_local_normal(&self.config, self.direction),
            (TransformKind::Plane, GizmoDirection::View) => DVec3::ONE,
            (TransformKind::Plane, _) => {
                (plane_bitangent(self.direction) + plane_tangent(self.direction)).normalize()
            }
        };

        let offset = DVec3::ONE + (direction * delta);
        let new_scale = state.start_scale * offset;

        Some(GizmoResult {
            scale: new_scale.as_vec3().into(),
            rotation: self.config.rotation.as_quat().into(),
            translation: self.config.translation.as_vec3().into(),
            mode: GizmoMode::Scale,
            value: Some(offset.as_vec3().to_array()),
        })
    }

    fn draw(&mut self, ui: &Ui) {
        match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => {
                draw_arrow(self, ui, self.direction, ArrowheadStyle::Square);
            }
            (TransformKind::Plane, GizmoDirection::View) => {
                draw_circle(
                    self,
                    ui,
                    gizmo_color(self, self.direction),
                    inner_circle_radius(&self.config),
                    false,
                );
                draw_circle(
                    self,
                    ui,
                    gizmo_color(self, self.direction),
                    outer_circle_radius(&self.config),
                    false,
                );
            }
            (TransformKind::Plane, _) => draw_plane(self, ui, self.direction),
        }
    }
}

fn distance_from_origin_2d<T: SubGizmoKind>(subgizmo: &SubGizmoConfig<T>, ui: &Ui) -> Option<f64> {
    let cursor_pos = ui.input(|i| i.pointer.hover_pos())?;
    let viewport = subgizmo.config.viewport;
    let gizmo_pos = world_to_screen(viewport, subgizmo.config.mvp, DVec3::new(0.0, 0.0, 0.0))?;

    Some(cursor_pos.distance(gizmo_pos) as f64)
}
