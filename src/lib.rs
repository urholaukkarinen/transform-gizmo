//! Provides a 3d transformation gizmo that can be used to manipulate 4x4
//! transformation matrices. Such gizmos are commonly used in applications
//! such as game engines and 3d modeling software.
//!
//! # Creating a gizmo
//! For a more complete example, see the online demo at <https://urholaukkarinen.github.io/egui-gizmo/>.
//! The demo sources can be found at <https://github.com/urholaukkarinen/egui-gizmo/blob/main/demo/src/main.rs>.
//!
//! ## A basic example
//! ```text
//! let gizmo = Gizmo::new("My gizmo")
//!     .view_matrix(view_matrix)
//!     .projection_matrix(projection_matrix)
//!     .model_matrix(model_matrix)
//!     .mode(GizmoMode::Rotate);
//!
//! if let Some(response) = gizmo.interact(ui) {
//!     model_matrix = response.transform.into();
//! }
//! ```
//! The gizmo can be placed inside a container such as a [`egui::Window`] or an [`egui::Area`].
//! By default, the gizmo will use the ui clip rect as a viewport.
//! The gizmo will apply transformations to the given model matrix.
//! Result of the gizmo interaction includes the transformed 4x4 model matrix as a 2-dimensional array.

#![warn(clippy::all)]

use std::cmp::Ordering;
use std::f32::consts::PI;
use std::hash::Hash;
use std::ops::Sub;

use egui::{Color32, Context, Id, PointerButton, Rect, Sense, Ui};
use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};

use crate::subgizmo::{SubGizmo, SubGizmoKind};

mod math;
mod painter;
mod rotation;
mod scale;
mod subgizmo;
mod translation;

/// The default snapping distance for rotation in radians
pub const DEFAULT_SNAP_ANGLE: f32 = PI / 32.0;
/// The default snapping distance for translation
pub const DEFAULT_SNAP_DISTANCE: f32 = 0.1;
/// The default snapping distance for scale
pub const DEFAULT_SNAP_SCALE: f32 = 0.1;

/// Maximum number of subgizmos in a single gizmo.
/// A subgizmo array of this size is allocated from stack,
/// even if the actual number of subgizmos is less.
const MAX_SUBGIZMOS: usize = 6;

pub struct Gizmo {
    id: Id,
    config: GizmoConfig,
    subgizmos: [Option<SubGizmo>; MAX_SUBGIZMOS],
    subgizmo_count: usize,
}

impl Gizmo {
    pub fn new(id_source: impl Hash) -> Self {
        Self {
            id: Id::new(id_source),
            config: GizmoConfig::default(),
            subgizmos: Default::default(),
            subgizmo_count: 0,
        }
    }

    /// Matrix that specifies translation and rotation of the gizmo in world space
    pub fn model_matrix(mut self, model_matrix: impl Into<[[f32; 4]; 4]>) -> Self {
        self.config.model_matrix = Mat4::from_cols_array_2d(&model_matrix.into());
        self
    }

    /// Matrix that specifies translation and rotation of the viewport camera
    pub fn view_matrix(mut self, view_matrix: impl Into<[[f32; 4]; 4]>) -> Self {
        self.config.view_matrix = Mat4::from_cols_array_2d(&view_matrix.into());
        self
    }

    /// Matrix that specifies projection of the viewport
    pub fn projection_matrix(mut self, projection_matrix: impl Into<[[f32; 4]; 4]>) -> Self {
        self.config.projection_matrix = Mat4::from_cols_array_2d(&projection_matrix.into());
        self
    }

    /// Bounds of the viewport in pixels
    pub fn viewport(mut self, viewport: Rect) -> Self {
        self.config.viewport = viewport;
        self
    }

    /// Gizmo mode to use
    pub fn mode(mut self, mode: GizmoMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Gizmo orientation to use
    pub fn orientation(mut self, orientation: GizmoOrientation) -> Self {
        self.config.orientation = orientation;
        self
    }

    /// Whether snapping is enabled
    pub fn snapping(mut self, snapping: bool) -> Self {
        self.config.snapping = snapping;
        self
    }

    /// Snap angle to use for rotation when snapping is enabled
    pub fn snap_angle(mut self, snap_angle: f32) -> Self {
        self.config.snap_angle = snap_angle;
        self
    }

    /// Snap distance to use for translation when snapping is enabled
    pub fn snap_distance(mut self, snap_distance: f32) -> Self {
        self.config.snap_distance = snap_distance;
        self
    }

    /// Snap distance to use for scaling when snapping is enabled
    pub fn snap_scale(mut self, snap_scale: f32) -> Self {
        self.config.snap_scale = snap_scale;
        self
    }

    /// Visual configuration of the gizmo, such as colors and size
    pub fn visuals(mut self, visuals: GizmoVisuals) -> Self {
        self.config.visuals = visuals;
        self
    }

    /// Draw and interact with the gizmo. This consumes the gizmo.
    ///
    /// Returns the result of the interaction, which includes a transformed model matrix.
    /// [`None`] is returned when the gizmo is not active.
    pub fn interact(mut self, ui: &mut Ui) -> Option<GizmoResult> {
        self.config.prepare(ui);

        // Choose subgizmos based on the gizmo mode
        match self.config.mode {
            GizmoMode::Rotate => self.add_subgizmos(self.new_rotation()),
            GizmoMode::Translate => self.add_subgizmos(self.new_translation()),
            GizmoMode::Scale => self.add_subgizmos(self.new_scale()),
        };

        let mut result = None;
        let mut state = GizmoState::load(ui.ctx(), self.id);

        if let Some(pointer_ray) = self.pointer_ray(ui) {
            let interaction = ui.interact(self.config.viewport, self.id, Sense::click_and_drag());
            let dragging = interaction.dragged_by(PointerButton::Primary);

            // If there is no active subgizmo, find which one of them
            // is under the mouse pointer, if any.
            if state.active_subgizmo_id.is_none() {
                if let Some(subgizmo) = self.pick_subgizmo(ui, pointer_ray) {
                    subgizmo.focused = true;

                    if interaction.drag_started() && dragging {
                        state.active_subgizmo_id = Some(subgizmo.id);
                    }
                }
            }

            let active_subgizmo = state
                .active_subgizmo_id
                .and_then(|id| self.subgizmos_mut().find(|subgizmo| subgizmo.id == id));

            if let Some(subgizmo) = active_subgizmo {
                if dragging {
                    subgizmo.active = true;
                    subgizmo.focused = true;
                    result = subgizmo.update(ui, pointer_ray);
                } else {
                    state.active_subgizmo_id = None;
                }
            }
        }

        state.save(ui.ctx(), self.id);

        for subgizmo in self.subgizmos() {
            if state.active_subgizmo_id.is_none() || subgizmo.active {
                subgizmo.draw(ui);
            }
        }

        result
    }

    /// Picks the subgizmo that is closest to the mouse pointer
    fn pick_subgizmo(&mut self, ui: &Ui, ray: Ray) -> Option<&mut SubGizmo> {
        self.subgizmos_mut()
            .filter_map(|subgizmo| subgizmo.pick(ui, ray).map(|t| (t, subgizmo)))
            .min_by(|(first, _), (second, _)| first.partial_cmp(second).unwrap_or(Ordering::Equal))
            .map(|(_, subgizmo)| subgizmo)
    }

    /// Iterator to the subgizmos
    fn subgizmos(&self) -> impl Iterator<Item = &SubGizmo> {
        self.subgizmos.iter().flatten()
    }

    /// Mutable iterator to the subgizmos
    fn subgizmos_mut(&mut self) -> impl Iterator<Item = &mut SubGizmo> {
        self.subgizmos.iter_mut().flatten()
    }

    /// Create subgizmos for rotation
    fn new_rotation(&self) -> [SubGizmo; 4] {
        [
            SubGizmo::new(
                self.id.with("rx"),
                self.config,
                GizmoDirection::X,
                SubGizmoKind::RotationAxis,
            ),
            SubGizmo::new(
                self.id.with("ry"),
                self.config,
                GizmoDirection::Y,
                SubGizmoKind::RotationAxis,
            ),
            SubGizmo::new(
                self.id.with("rz"),
                self.config,
                GizmoDirection::Z,
                SubGizmoKind::RotationAxis,
            ),
            SubGizmo::new(
                self.id.with("rs"),
                self.config,
                GizmoDirection::Screen,
                SubGizmoKind::RotationAxis,
            ),
        ]
    }

    /// Create subgizmos for translation
    fn new_translation(&self) -> [SubGizmo; 6] {
        [
            SubGizmo::new(
                self.id.with("tx"),
                self.config,
                GizmoDirection::X,
                SubGizmoKind::TranslationVector,
            ),
            SubGizmo::new(
                self.id.with("ty"),
                self.config,
                GizmoDirection::Y,
                SubGizmoKind::TranslationVector,
            ),
            SubGizmo::new(
                self.id.with("tz"),
                self.config,
                GizmoDirection::Z,
                SubGizmoKind::TranslationVector,
            ),
            SubGizmo::new(
                self.id.with("tyz"),
                self.config,
                GizmoDirection::X,
                SubGizmoKind::TranslationPlane,
            ),
            SubGizmo::new(
                self.id.with("txz"),
                self.config,
                GizmoDirection::Y,
                SubGizmoKind::TranslationPlane,
            ),
            SubGizmo::new(
                self.id.with("txy"),
                self.config,
                GizmoDirection::Z,
                SubGizmoKind::TranslationPlane,
            ),
        ]
    }

    /// Create subgizmos for scale
    fn new_scale(&self) -> [SubGizmo; 6] {
        [
            SubGizmo::new(
                self.id.with("sx"),
                self.config,
                GizmoDirection::X,
                SubGizmoKind::ScaleVector,
            ),
            SubGizmo::new(
                self.id.with("sy"),
                self.config,
                GizmoDirection::Y,
                SubGizmoKind::ScaleVector,
            ),
            SubGizmo::new(
                self.id.with("sz"),
                self.config,
                GizmoDirection::Z,
                SubGizmoKind::ScaleVector,
            ),
            SubGizmo::new(
                self.id.with("syz"),
                self.config,
                GizmoDirection::X,
                SubGizmoKind::ScalePlane,
            ),
            SubGizmo::new(
                self.id.with("sxz"),
                self.config,
                GizmoDirection::Y,
                SubGizmoKind::ScalePlane,
            ),
            SubGizmo::new(
                self.id.with("sxy"),
                self.config,
                GizmoDirection::Z,
                SubGizmoKind::ScalePlane,
            ),
        ]
    }

    /// Add given subgizmos to this gizmo
    fn add_subgizmos<const N: usize>(&mut self, subgizmos: [SubGizmo; N]) {
        let mut i = self.subgizmo_count;
        for subgizmo in subgizmos.into_iter() {
            self.subgizmos[i] = Some(subgizmo);
            i += 1;
        }

        self.subgizmo_count = i;
    }

    /// Calculate a world space ray from current mouse position
    fn pointer_ray(&self, ui: &Ui) -> Option<Ray> {
        let hover = ui.input().pointer.hover_pos()?;
        let viewport = self.config.viewport;

        let x = ((hover.x - viewport.min.x) / viewport.width()) * 2.0 - 1.0;
        let y = ((hover.y - viewport.min.y) / viewport.height()) * 2.0 - 1.0;

        let screen_to_world = self.config.view_projection.inverse();
        let mut origin = screen_to_world * Vec4::new(x, -y, -1.0, 1.0);
        origin /= origin.w;
        let mut target = screen_to_world * Vec4::new(x, -y, 1.0, 1.0);

        // w is zero when far plane is set to infinity
        if target.w.abs() < 1e-7 {
            target.w = 1e-7;
        }

        target /= target.w;

        let direction = target.sub(origin).xyz().normalize();

        Some(Ray {
            origin: origin.xyz(),
            direction,
        })
    }
}

/// Result of an active transformation
#[derive(Debug, Copy, Clone)]
pub struct GizmoResult {
    /// Transformed model matrix
    pub transform: [[f32; 4]; 4],
    /// Mode of the active subgizmo
    pub mode: GizmoMode,
    /// Current rotation or translation for each axis, depending on the mode.
    pub value: [f32; 3],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GizmoMode {
    /// Only rotation
    Rotate,
    /// Only translation
    Translate,
    /// Only scale
    Scale,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GizmoOrientation {
    /// Transformation axes are aligned to world space. Rotation of the
    /// gizmo does not change.
    Global,
    /// Transformation axes are aligned to local space. Rotation of the
    /// gizmo matches the rotation represented by the model matrix.
    Local,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GizmoDirection {
    /// Gizmo points in the X-direction
    X,
    /// Gizmo points in the Y-direction
    Y,
    /// Gizmo points in the Z-direction
    Z,
    /// Gizmo points towards the screen
    Screen,
}

/// Controls the visual style of the gizmo
#[derive(Debug, Copy, Clone)]
pub struct GizmoVisuals {
    /// Color of the x axis
    pub x_color: Color32,
    /// Color of the y axis
    pub y_color: Color32,
    /// Color of the z axis
    pub z_color: Color32,
    /// Color of the screen direction axis
    pub s_color: Color32,
    /// Alpha of the gizmo color when inactive
    pub inactive_alpha: f32,
    /// Alpha of the gizmo color when highlighted/active
    pub highlight_alpha: f32,
    /// Color to use for highlighted and active axes. By default the axis color is used with `highlight_alpha`
    pub highlight_color: Option<Color32>,
    /// Width (thickness) of the gizmo strokes
    pub stroke_width: f32,
    /// Gizmo size in pixels
    pub gizmo_size: f32,
}

impl Default for GizmoVisuals {
    fn default() -> Self {
        Self {
            x_color: Color32::from_rgb(255, 50, 0),
            y_color: Color32::from_rgb(50, 255, 0),
            z_color: Color32::from_rgb(0, 50, 255),
            s_color: Color32::from_rgb(255, 255, 255),
            inactive_alpha: 0.5,
            highlight_alpha: 0.9,
            highlight_color: None,
            stroke_width: 4.0,
            gizmo_size: 75.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct GizmoConfig {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub model_matrix: Mat4,
    pub viewport: Rect,
    pub mode: GizmoMode,
    pub orientation: GizmoOrientation,
    pub snapping: bool,
    pub snap_angle: f32,
    pub snap_distance: f32,
    pub snap_scale: f32,
    pub visuals: GizmoVisuals,
    //----------------------------------//
    pub rotation: Quat,
    pub translation: Vec3,
    pub scale: Vec3,
    pub view_projection: Mat4,
    pub mvp: Mat4,
    pub scale_factor: f32,
    /// How close the mouse pointer needs to be to a subgizmo before it is focused
    pub focus_distance: f32,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            model_matrix: Mat4::IDENTITY,
            viewport: Rect::NOTHING,
            mode: GizmoMode::Rotate,
            orientation: GizmoOrientation::Global,
            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
            snap_scale: DEFAULT_SNAP_SCALE,
            visuals: GizmoVisuals::default(),
            //----------------------------------//
            rotation: Quat::IDENTITY,
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
            view_projection: Mat4::IDENTITY,
            mvp: Mat4::IDENTITY,
            scale_factor: 0.0,
            focus_distance: 0.0,
        }
    }
}

impl GizmoConfig {
    /// Prepare the gizmo configuration for interaction and rendering.
    /// Some values are precalculated for better performance at the cost of memory usage.
    fn prepare(&mut self, ui: &Ui) {
        // Use ui clip rect if the user has not specified a viewport
        if self.viewport.is_negative() {
            self.viewport = ui.clip_rect();
        }

        let (scale, rotation, translation) = self.model_matrix.to_scale_rotation_translation();
        self.rotation = rotation;
        self.translation = translation;
        self.scale = scale;
        self.view_projection = self.projection_matrix * self.view_matrix;
        self.mvp = self.projection_matrix * self.view_matrix * self.model_matrix;

        self.scale_factor =
            self.mvp.as_ref()[15] / self.projection_matrix.as_ref()[0] / self.viewport.width()
                * 2.0;

        self.focus_distance = self.scale_factor * (self.visuals.stroke_width / 2.0 + 5.0);
    }

    /// Forward vector of the view camera
    pub(crate) fn view_forward(&self) -> Vec3 {
        self.view_matrix.row(2).xyz()
    }

    /// Right vector of the view camera
    pub(crate) fn view_right(&self) -> Vec3 {
        self.view_matrix.row(0).xyz()
    }

    /// Whether local orientation is used
    pub(crate) fn local_space(&self) -> bool {
        self.orientation == GizmoOrientation::Local
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Ray {
    origin: Vec3,
    direction: Vec3,
}

/// Gizmo state that is saved between frames
#[derive(Default, Debug, Copy, Clone)]
struct GizmoState {
    active_subgizmo_id: Option<Id>,
}

pub(crate) trait WidgetData: Sized + Default + Copy + Clone + Send + Sync + 'static {
    fn load(ctx: &Context, gizmo_id: Id) -> Self {
        *ctx.memory().data.get_temp_mut_or_default(gizmo_id)
    }

    fn save(self, ctx: &Context, gizmo_id: Id) {
        ctx.memory().data.insert_temp(gizmo_id, self);
    }
}

impl WidgetData for GizmoState {}
