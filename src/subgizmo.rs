use std::hash::Hash;

use egui::{Color32, Id, Ui};
use glam::Vec3;

use crate::rotation::{draw_rotation, pick_rotation, update_rotation};
use crate::scale::{draw_scale, pick_scale, update_scale};
use crate::translation::{
    draw_translation, draw_translation_plane, pick_translation, pick_translation_plane,
    update_translation, update_translation_plane,
};
use crate::{GizmoConfig, GizmoDirection, GizmoResult, Ray, WidgetData};

#[derive(Copy, Clone)]
pub(crate) struct SubGizmo {
    pub(crate) id: Id,
    pub(crate) config: GizmoConfig,
    pub(crate) direction: GizmoDirection,
    pub(crate) kind: SubGizmoKind,
    /// Whether this subgizmo is focused this frame
    pub(crate) focused: bool,
    /// Whether this subgizmo is active this frame
    pub(crate) active: bool,
}

impl SubGizmo {
    pub fn new(
        id_source: impl Hash,
        config: GizmoConfig,
        direction: GizmoDirection,
        kind: SubGizmoKind,
    ) -> Self {
        Self {
            id: Id::new(id_source),
            config,
            direction,
            kind,
            focused: false,
            active: false,
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

    pub fn color(&self) -> Color32 {
        let color = match self.direction {
            GizmoDirection::X => self.config.visuals.x_color,
            GizmoDirection::Y => self.config.visuals.y_color,
            GizmoDirection::Z => self.config.visuals.z_color,
            GizmoDirection::Screen => self.config.visuals.s_color,
        };

        let color = if self.focused {
            self.config.visuals.highlight_color.unwrap_or(color)
        } else {
            color
        };

        let alpha = if self.focused {
            self.config.visuals.highlight_alpha
        } else {
            self.config.visuals.inactive_alpha
        };

        color.linear_multiply(alpha)
    }

    pub fn state<T: WidgetData>(&self, ui: &Ui) -> T {
        T::load(ui.ctx(), self.id)
    }

    pub fn update_state_with<T: WidgetData>(&self, ui: &Ui, fun: impl FnOnce(&mut T)) {
        let mut state = self.state::<T>(ui);
        fun(&mut state);
        state.save(ui.ctx(), self.id);
    }

    pub fn pick(&self, ui: &Ui, ray: Ray) -> Option<f32> {
        match self.kind {
            SubGizmoKind::RotationAxis => pick_rotation(self, ui, ray),
            SubGizmoKind::TranslationVector => pick_translation(self, ui, ray),
            SubGizmoKind::TranslationPlane => pick_translation_plane(self, ui, ray),
            SubGizmoKind::ScaleVector => pick_scale(self, ui, ray),
        }
    }

    /// Update this subgizmo based on pointer ray and interaction.
    pub fn update(&self, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
        match self.kind {
            SubGizmoKind::RotationAxis => update_rotation(self, ui, ray),
            SubGizmoKind::TranslationVector => update_translation(self, ui, ray),
            SubGizmoKind::TranslationPlane => update_translation_plane(self, ui, ray),
            SubGizmoKind::ScaleVector => update_scale(self, ui, ray),
        }
    }

    /// Draw this subgizmo
    pub fn draw(&self, ui: &Ui) {
        match self.kind {
            SubGizmoKind::RotationAxis => draw_rotation(self, ui),
            SubGizmoKind::TranslationVector => draw_translation(self, ui),
            SubGizmoKind::TranslationPlane => draw_translation_plane(self, ui),
            SubGizmoKind::ScaleVector => draw_scale(self, ui),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum SubGizmoKind {
    /// Rotation around an axis
    RotationAxis,
    /// Translation along a vector
    TranslationVector,
    /// Translation along a plane
    TranslationPlane,
    /// Scale along a vector
    ScaleVector,
}
