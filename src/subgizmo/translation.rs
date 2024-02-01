use egui::Ui;
use glam::DVec3;

use crate::math::{intersect_plane, ray_to_ray, round_to_interval};

use crate::subgizmo::common::{
    draw_arrow, draw_circle, draw_plane, gizmo_color, gizmo_normal, inner_circle_radius,
    pick_arrow, pick_circle, pick_plane, plane_bitangent, plane_global_origin, plane_tangent,
    ArrowheadStyle,
};
use crate::subgizmo::{SubGizmo, SubGizmoConfig, SubGizmoKind, TransformKind};
use crate::{GizmoDirection, GizmoMode, GizmoResult, Ray};

pub(crate) type TranslationSubGizmo = SubGizmoConfig<Translation>;

#[derive(Debug, Copy, Clone)]
pub(crate) struct TranslationParams {
    pub direction: GizmoDirection,
    pub transform_kind: TransformKind,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct TranslationState {
    start_point: DVec3,
    last_point: DVec3,
    current_delta: DVec3,
}

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Translation;

impl SubGizmoKind for Translation {
    type Params = TranslationParams;
    type State = TranslationState;
}

impl SubGizmo for TranslationSubGizmo {
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64> {
        let pick_result = match (self.transform_kind, self.direction) {
            (TransformKind::Plane, GizmoDirection::View) => {
                pick_circle(self, ray, inner_circle_radius(&self.config), true)
            }
            (TransformKind::Plane, _) => pick_plane(self, ray, self.direction),
            (TransformKind::Axis, _) => pick_arrow(self, ray, self.direction),
        };

        self.opacity = pick_result.visibility as _;

        self.update_state_with(ui, |state: &mut TranslationState| {
            state.start_point = pick_result.subgizmo_point;
            state.last_point = pick_result.subgizmo_point;
            state.current_delta = DVec3::ZERO;
        });

        if pick_result.picked {
            Some(pick_result.t)
        } else {
            None
        }
    }

    fn update(&mut self, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
        let state = self.state(ui);

        let mut new_point = if self.transform_kind == TransformKind::Axis {
            point_on_axis(self, ray)
        } else {
            point_on_plane(
                gizmo_normal(&self.config, self.direction),
                plane_global_origin(&self.config, self.direction),
                ray,
            )?
        };

        let mut new_delta = new_point - state.start_point;

        if self.config.snapping {
            new_delta = if self.transform_kind == TransformKind::Axis {
                snap_translation_vector(self, new_delta)
            } else {
                snap_translation_plane(self, new_delta)
            };
            new_point = state.start_point + new_delta;
        }

        self.update_state_with(ui, |state: &mut TranslationState| {
            state.last_point = new_point;
            state.current_delta = new_delta;
        });

        let new_translation = self.config.translation + new_point - state.last_point;

        Some(GizmoResult {
            scale: self.config.scale.as_vec3().into(),
            rotation: self.config.rotation.as_quat().into(),
            translation: new_translation.as_vec3().into(),
            mode: GizmoMode::Translate,
            value: Some(state.current_delta.as_vec3().to_array()),
        })
    }

    fn draw(&mut self, ui: &Ui) {
        match (self.transform_kind, self.direction) {
            (TransformKind::Axis, _) => draw_arrow(self, ui, self.direction, ArrowheadStyle::Cone),
            (TransformKind::Plane, GizmoDirection::View) => {
                draw_circle(
                    self,
                    ui,
                    gizmo_color(self, self.direction),
                    inner_circle_radius(&self.config),
                    false,
                );
            }
            (TransformKind::Plane, _) => draw_plane(self, ui, self.direction),
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
