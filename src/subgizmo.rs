use std::hash::Hash;
use std::marker::PhantomData;

use egui::{Color32, Id, Ui};
use glam::DVec3;

use crate::{GizmoConfig, GizmoDirection, GizmoResult, Ray, WidgetData};

pub(crate) use arcball::ArcballSubGizmo;
pub(crate) use rotation::RotationSubGizmo;
pub(crate) use scale::ScaleSubGizmo;
pub(crate) use translation::TranslationSubGizmo;

mod arcball;
mod common;
mod rotation;
mod scale;
mod translation;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransformKind {
    Axis,
    Plane,
}

pub(crate) trait SubGizmoState: Default + Copy + Clone + Send + Sync + 'static {}
impl<T> WidgetData for T where T: SubGizmoState {}

pub(crate) struct SubGizmoConfig<T> {
    id: Id,
    pub(crate) config: GizmoConfig,
    pub(crate) direction: GizmoDirection,
    pub(crate) transform_kind: TransformKind,
    /// Whether this subgizmo is focused this frame
    pub(crate) focused: bool,
    /// Whether this subgizmo is active this frame
    pub(crate) active: bool,
    /// Opacity of the subgizmo for this frame.
    /// A fully invisible subgizmo cannot be interacted with.
    pub(crate) opacity: f32,

    _phantom: PhantomData<T>,
}

pub(crate) trait SubGizmoBase: 'static {
    /// Identifier for this subgizmo. It should be unique across all subgizmos.
    fn id(&self) -> Id;
    /// Sets whether this subgizmo is currently focused
    fn set_focused(&mut self, focused: bool);
    /// Sets whether this subgizmo is currently active
    fn set_active(&mut self, active: bool);
    /// Returns true if this subgizmo is currently focused
    fn is_focused(&self) -> bool;
    /// Returns true if this subgizmo is currently active
    fn is_active(&self) -> bool;
}

impl<T: SubGizmoState> SubGizmoBase for SubGizmoConfig<T> {
    fn id(&self) -> Id {
        self.id
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

pub(crate) trait SubGizmo: SubGizmoBase {
    /// Pick the subgizmo based on pointer ray. If it is close enough to
    /// the mouse pointer, distance from camera to the subgizmo is returned.
    fn pick(&mut self, ui: &Ui, ray: Ray) -> Option<f64>;
    /// Update the subgizmo based on pointer ray and interaction.
    fn update(&mut self, ui: &Ui, ray: Ray) -> Option<GizmoResult>;
    /// Draw the subgizmo
    fn draw(&mut self, ui: &Ui);
}

impl<T> SubGizmoConfig<T>
where
    T: SubGizmoState,
{
    pub fn new(
        id_source: impl Hash,
        config: GizmoConfig,
        direction: GizmoDirection,
        transform_kind: TransformKind,
    ) -> Self {
        Self {
            id: Id::new(id_source),
            config,
            direction,
            transform_kind,
            focused: false,
            active: false,
            opacity: 0.0,
            _phantom: Default::default(),
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

    pub fn state(&self, ui: &Ui) -> T {
        <_ as WidgetData>::load(ui.ctx(), self.id)
    }

    pub fn update_state_with(&self, ui: &Ui, fun: impl FnOnce(&mut T)) {
        let mut state = self.state(ui);
        fun(&mut state);
        state.save(ui.ctx(), self.id);
    }
}
