use std::ops::{Deref, DerefMut};

use ecolor::Color32;
use emath::Rect;
pub use glam::{DMat4, DQuat, DVec3, Mat4, Vec4Swizzles};

use crate::math::{screen_to_world, world_to_screen};

/// The default snapping distance for rotation in radians
pub const DEFAULT_SNAP_ANGLE: f32 = std::f32::consts::PI / 32.0;
/// The default snapping distance for translation
pub const DEFAULT_SNAP_DISTANCE: f32 = 0.1;
/// The default snapping distance for scale
pub const DEFAULT_SNAP_SCALE: f32 = 0.1;

#[derive(Debug, Copy, Clone)]
pub struct GizmoConfig {
    /// View matrix for the gizmo, aligning it with the camera's viewpoint.
    pub view_matrix: DMat4,

    /// Projection matrix for the gizmo, determining how it is projected onto the screen.
    pub projection_matrix: DMat4,

    /// Model matrix for positioning the gizmo in the world space.
    pub model_matrix: DMat4,

    /// Screen area where the gizmo is displayed.
    pub viewport: Rect,

    /// The gizmo's operation mode.
    pub mode: GizmoMode,

    /// Determines the gizmo's orientation relative to global or local axes.
    pub orientation: GizmoOrientation,

    /// Toggles snapping to predefined increments during transformations for precision.
    pub snapping: bool,

    /// Angle increment for snapping rotations, in radians.
    pub snap_angle: f32,

    /// Distance increment for snapping translations.
    pub snap_distance: f32,

    /// Scale increment for snapping scalings.
    pub snap_scale: f32,

    /// Visual settings for the gizmo, affecting appearance and visibility.
    pub visuals: GizmoVisuals,

    /// Ratio of window's physical size to logical size.
    pub pixels_per_point: f32,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct PreparedGizmoConfig {
    config: GizmoConfig,
    //----------------------------------//
    pub rotation: DQuat,
    pub translation: DVec3,
    pub scale: DVec3,
    pub view_projection: DMat4,
    pub mvp: DMat4,
    pub scale_factor: f32,
    /// How close the mouse pointer needs to be to a subgizmo before it is focused
    pub focus_distance: f32,
    /// Whether left-handed projection is used.
    pub left_handed: bool,
    pub eye_to_model_dir: DVec3,
}

impl Deref for PreparedGizmoConfig {
    type Target = GizmoConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for PreparedGizmoConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            view_matrix: DMat4::IDENTITY,
            projection_matrix: DMat4::IDENTITY,
            model_matrix: DMat4::IDENTITY,
            viewport: Rect::NOTHING,
            mode: GizmoMode::Rotate,
            orientation: GizmoOrientation::Global,
            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
            snap_scale: DEFAULT_SNAP_SCALE,
            visuals: GizmoVisuals::default(),
            pixels_per_point: 1.0,
        }
    }
}

impl PreparedGizmoConfig {
    pub fn from_config(config: GizmoConfig) -> Self {
        let (scale, rotation, translation) = config.model_matrix.to_scale_rotation_translation();
        let view_projection = config.projection_matrix * config.view_matrix;
        let mvp = config.projection_matrix * config.view_matrix * config.model_matrix;

        let scale_factor = mvp.as_ref()[15] as f32
            / config.projection_matrix.as_ref()[0] as f32
            / config.viewport.width()
            * 2.0;

        let focus_distance = scale_factor * (config.visuals.stroke_width / 2.0 + 5.0);

        let left_handed = if config.projection_matrix.z_axis.w == 0.0 {
            config.projection_matrix.z_axis.z > 0.0
        } else {
            config.projection_matrix.z_axis.w > 0.0
        };

        let gizmo_screen_pos =
            world_to_screen(config.viewport, mvp, translation).unwrap_or_default();

        let gizmo_view_near = screen_to_world(
            config.viewport,
            view_projection.inverse(),
            gizmo_screen_pos,
            -1.0,
        );

        let eye_to_model_dir = (gizmo_view_near - translation).normalize_or_zero();

        Self {
            config,
            rotation,
            translation,
            scale,
            view_projection,
            mvp,
            eye_to_model_dir,
            scale_factor,
            focus_distance,
            left_handed,
        }
    }
}

impl GizmoConfig {
    /// Forward vector of the view camera
    pub(crate) fn view_forward(&self) -> DVec3 {
        self.view_matrix.row(2).xyz()
    }

    /// Up vector of the view camera
    pub(crate) fn view_up(&self) -> DVec3 {
        self.view_matrix.row(1).xyz()
    }

    /// Right vector of the view camera
    pub(crate) fn view_right(&self) -> DVec3 {
        self.view_matrix.row(0).xyz()
    }

    /// Whether local orientation is used
    pub(crate) fn local_space(&self) -> bool {
        // Scale mode only works in local space
        self.orientation == GizmoOrientation::Local || self.mode == GizmoMode::Scale
    }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GizmoDirection {
    /// Gizmo points in the X-direction
    X,
    /// Gizmo points in the Y-direction
    Y,
    /// Gizmo points in the Z-direction
    Z,
    /// Gizmo points in the view direction
    View,
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
    /// Color of the forward axis
    pub s_color: Color32,
    /// Alpha of the gizmo color when inactive
    pub inactive_alpha: f32,
    /// Alpha of the gizmo color when highlighted/active
    pub highlight_alpha: f32,
    /// Color to use for highlighted and active axes. By default, the axis color is used with `highlight_alpha`
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
