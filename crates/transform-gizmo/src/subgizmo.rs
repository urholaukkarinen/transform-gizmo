use std::any::Any;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::Deref;

use enum_dispatch::enum_dispatch;

use crate::{config::PreparedGizmoConfig, gizmo::Ray, GizmoDrawData, GizmoResult};

pub(crate) use arcball::ArcballSubGizmo;
pub(crate) use rotation::RotationSubGizmo;
pub(crate) use scale::ScaleSubGizmo;
pub(crate) use translation::TranslationSubGizmo;

pub(crate) mod arcball;
pub(crate) mod common;
pub(crate) mod rotation;
pub(crate) mod scale;
pub(crate) mod translation;

#[derive(Clone, Debug)]
/// Enumeration of different subgizmo types.
#[enum_dispatch(SubGizmoControl)]
pub(crate) enum SubGizmo {
    Rotate(RotationSubGizmo),
    Translate(TranslationSubGizmo),
    Scale(ScaleSubGizmo),
    Arcball(ArcballSubGizmo),
}

#[enum_dispatch]
pub(crate) trait SubGizmoControl {
    /// Unique identifier for this subgizmo.
    fn id(&self) -> u64;
    /// Update the configuration used by the gizmo.
    fn update_config(&mut self, config: PreparedGizmoConfig);
    /// Sets whether this subgizmo is currently focused.
    fn set_focused(&mut self, focused: bool);
    /// Sets whether this subgizmo is currently active.
    fn set_active(&mut self, active: bool);
    /// Returns true if this subgizmo is currently focused.
    fn is_focused(&self) -> bool;
    /// Returns true if this subgizmo is currently active.
    fn is_active(&self) -> bool;
    /// Pick the subgizmo based on pointer ray. If it is close enough to
    /// the mouse pointer, distance from camera to the subgizmo is returned.
    fn pick(&mut self, ray: Ray) -> Option<f64>;
    /// Update the subgizmo based on pointer ray and interaction.
    fn update(&mut self, ray: Ray) -> Option<GizmoResult>;
    /// Draw the subgizmo.
    fn draw(&self) -> GizmoDrawData;
}

pub(crate) trait SubGizmoKind: 'static {
    type Params: Debug + Copy + Hash;
    type State: Debug + Copy + Clone + Send + Sync + Default + 'static;

    fn pick(subgizmo: &mut SubGizmoConfig<Self>, ray: Ray) -> Option<f64>
    where
        Self: Sized;
    fn update(subgizmo: &mut SubGizmoConfig<Self>, ray: Ray) -> Option<GizmoResult>
    where
        Self: Sized;
    fn draw(subgizmo: &SubGizmoConfig<Self>) -> GizmoDrawData
    where
        Self: Sized;
}

#[derive(Clone, Debug)]
pub(crate) struct SubGizmoConfig<T: SubGizmoKind> {
    id: u64,
    /// Additional parameters depending on the subgizmo kind.
    params: T::Params,

    /// Configuration of the full gizmo
    pub(crate) config: PreparedGizmoConfig,
    /// Whether this subgizmo is focused this frame
    pub(crate) focused: bool,
    /// Whether this subgizmo is active this frame
    pub(crate) active: bool,
    /// Opacity of the subgizmo for this frame.
    /// A fully invisible subgizmo cannot be interacted with.
    pub(crate) opacity: f32,
    /// Implementation-specific state of the subgizmo.
    pub(crate) state: T::State,
}

impl<T: SubGizmoKind> Deref for SubGizmoConfig<T> {
    type Target = T::Params;

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

impl<T> SubGizmoConfig<T>
where
    T: SubGizmoKind,
{
    pub(crate) fn new(config: PreparedGizmoConfig, params: T::Params) -> Self {
        let mut hasher = ahash::RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        params.type_id().hash(&mut hasher);
        params.hash(&mut hasher);
        let id = hasher.finish();

        Self {
            id,
            params,
            config,
            focused: false,
            active: false,
            opacity: 0.0,
            state: Default::default(),
        }
    }
}

impl<T> SubGizmoControl for SubGizmoConfig<T>
where
    T: SubGizmoKind,
{
    fn id(&self) -> u64 {
        self.id
    }
    fn update_config(&mut self, config: PreparedGizmoConfig) {
        self.config = config;
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

    fn pick(&mut self, ray: Ray) -> Option<f64> {
        T::pick(self, ray)
    }

    fn update(&mut self, ray: Ray) -> Option<GizmoResult> {
        T::update(self, ray)
    }

    fn draw(&self) -> GizmoDrawData {
        T::draw(self)
    }
}
