use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use egui::{Color32, Id, Ui};
use glam::DVec3;

use self::rotation::{draw_rotation, pick_rotation, update_rotation};
use self::scale::{pick_scale, update_scale};
use self::translation::{pick_translation, update_translation};
use crate::subgizmo::common::{draw_arrow, draw_plane, pick_arrow, pick_plane};
use crate::{GizmoConfig, GizmoDirection, GizmoResult, Ray, WidgetData};

mod common;
mod rotation;
mod scale;
mod translation;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct SubGizmoState<T: Default + Copy + Clone> {
    inner: T,
    visibility: f32,
}

impl<T: Default + Copy + Clone> Deref for SubGizmoState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Default + Copy + Clone> DerefMut for SubGizmoState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> WidgetData for SubGizmoState<T> where T: Default + Copy + Clone + Send + Sync + 'static {}

#[derive(Copy, Clone, Debug)]
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

    pub fn local_normal(&self) -> DVec3 {
        match self.direction {
            GizmoDirection::X => DVec3::X,
            GizmoDirection::Y => DVec3::Y,
            GizmoDirection::Z => DVec3::Z,
            GizmoDirection::Screen => -self.config.view_forward(),
        }
    }

    pub fn normal(&self) -> DVec3 {
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

    pub fn state<T: Default + Copy + Clone + Send + Sync + 'static>(
        &self,
        ui: &Ui,
    ) -> SubGizmoState<T> {
        <_ as WidgetData>::load(ui.ctx(), self.id)
    }

    pub fn update_state_with<T: Default + Copy + Clone + Send + Sync + 'static>(
        &self,
        ui: &Ui,
        fun: impl FnOnce(&mut SubGizmoState<T>),
    ) {
        let mut state = self.state::<T>(ui);
        fun(&mut state);
        state.save(ui.ctx(), self.id);
    }

    pub fn pick(&self, ui: &Ui, ray: Ray) -> Option<f64> {
        match self.kind {
            SubGizmoKind::RotationAxis => pick_rotation(self, ui, ray),
            SubGizmoKind::TranslationVector => pick_translation(self, ui, ray, pick_arrow),
            SubGizmoKind::TranslationPlane => pick_translation(self, ui, ray, pick_plane),
            SubGizmoKind::ScaleVector => pick_scale(self, ui, ray, pick_arrow),
            SubGizmoKind::ScalePlane => pick_scale(self, ui, ray, pick_plane),
        }
    }

    /// Update this subgizmo based on pointer ray and interaction.
    pub fn update(&self, ui: &Ui, ray: Ray) -> Option<GizmoResult> {
        match self.kind {
            SubGizmoKind::RotationAxis => update_rotation(self, ui, ray),
            SubGizmoKind::TranslationVector | SubGizmoKind::TranslationPlane => {
                update_translation(self, ui, ray)
            }
            SubGizmoKind::ScaleVector | SubGizmoKind::ScalePlane => update_scale(self, ui),
        }
    }

    /// Draw this subgizmo
    pub fn draw(&self, ui: &Ui) {
        match self.kind {
            SubGizmoKind::RotationAxis => draw_rotation(self, ui),
            SubGizmoKind::TranslationVector | SubGizmoKind::ScaleVector => draw_arrow(self, ui),
            SubGizmoKind::TranslationPlane | SubGizmoKind::ScalePlane => draw_plane(self, ui),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum SubGizmoKind {
    /// Rotation around an axis
    RotationAxis,
    /// Translation along a vector
    TranslationVector,
    /// Translation along a plane
    TranslationPlane,
    /// Scale along a vector
    ScaleVector,
    /// Scale along a plane
    ScalePlane,
}
