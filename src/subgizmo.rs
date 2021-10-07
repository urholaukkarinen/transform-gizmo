use std::hash::Hash;

use egui::{Color32, Id, Response, Ui};
use glam::Vec3;

use crate::rotation::{draw_rotation, pick_rotation, update_rotation, RotationState};
use crate::translation::{
    draw_translation, pick_translation, update_translation, TranslationState,
};
use crate::{GizmoConfig, GizmoDirection, GizmoMode, GizmoResult, Ray};

#[derive(Copy, Clone)]
pub(crate) struct SubGizmo<'a> {
    pub(crate) ui: &'a Ui,
    pub(crate) id: Id,
    pub(crate) config: GizmoConfig,
    pub(crate) direction: GizmoDirection,
    pub(crate) mode: GizmoMode,
}

impl<'a> SubGizmo<'a> {
    pub fn new(
        ui: &'a Ui,
        id_source: impl Hash,
        config: GizmoConfig,
        direction: GizmoDirection,
        mode: GizmoMode,
    ) -> Self {
        Self {
            ui,
            id: Id::new(id_source),
            config,
            direction,
            mode,
        }
    }

    pub fn local_normal(&self) -> Vec3 {
        match self.direction {
            GizmoDirection::X => Vec3::X,
            GizmoDirection::Y => Vec3::Y,
            GizmoDirection::Z => Vec3::Z,
            GizmoDirection::Screen => -self.config.view_forward(),
        }
    }

    pub fn normal(&self) -> Vec3 {
        let mut normal = self.local_normal();

        if self.config.local_space() && self.direction != GizmoDirection::Screen {
            normal = self.config.rotation * normal;
        }

        normal
    }

    pub fn local_tangent(&self) -> Vec3 {
        match self.direction {
            GizmoDirection::X => Vec3::Z,
            GizmoDirection::Y => Vec3::Z,
            GizmoDirection::Z => -Vec3::Y,
            GizmoDirection::Screen => -self.config.view_right(),
        }
    }

    pub fn tangent(&self) -> Vec3 {
        let mut tangent = self.local_tangent();

        if self.config.local_space() && self.direction != GizmoDirection::Screen {
            tangent = self.config.rotation * tangent;
        }

        tangent
    }

    pub fn color(&self) -> Color32 {
        match self.direction {
            GizmoDirection::X => self.config.visuals.x_color,
            GizmoDirection::Y => self.config.visuals.y_color,
            GizmoDirection::Z => self.config.visuals.z_color,
            GizmoDirection::Screen => self.config.visuals.s_color,
        }
    }

    pub fn radius(&self) -> f32 {
        let mut radius = self.config.visuals.gizmo_size;

        if self.direction == GizmoDirection::Screen {
            // Screen axis should be a little bit larger
            radius += self.config.visuals.stroke_width + 5.0;
        }

        self.config.scale_factor * radius
    }

    pub fn state(&self) -> SubGizmoState {
        *self
            .ui
            .ctx()
            .memory()
            .id_data_temp
            .get_or_default::<SubGizmoState>(self.id)
    }

    pub fn update_state_with(&self, fun: impl FnOnce(&mut SubGizmoState)) {
        fun(self
            .ui
            .ctx()
            .memory()
            .id_data_temp
            .get_mut_or_default::<SubGizmoState>(self.id))
    }

    pub fn active(&self) -> bool {
        self.state().active
    }

    pub fn pick(&self, ray: Ray) -> Option<f32> {
        match self.mode {
            GizmoMode::Rotate => pick_rotation(self, ray),
            GizmoMode::Translate => pick_translation(self, ray),
        }
    }

    /// Update this subgizmo based on pointer ray and interaction.
    ///
    pub fn update(&self, ray: Ray, interaction: &Response) -> Option<GizmoResult> {
        match self.mode {
            GizmoMode::Rotate => update_rotation(self, ray, interaction),
            GizmoMode::Translate => update_translation(self, ray, interaction),
        }
    }

    /// Draw this subgizmo
    pub fn draw(&self) {
        match self.mode {
            GizmoMode::Rotate => draw_rotation(self),
            GizmoMode::Translate => draw_translation(self),
        }
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct SubGizmoState {
    /// Whether this subgizmo is focused
    pub focused: bool,
    /// Whether this subgizmo is active
    pub active: bool,
    /// State used for rotation
    pub rotation: RotationState,
    /// State used for translation
    pub translation: TranslationState,
}
