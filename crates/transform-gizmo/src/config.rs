use std::ops::{Deref, DerefMut};

pub use ecolor::Color32;

use emath::Rect;
use enumset::{enum_set, EnumSet, EnumSetType};

use crate::math::{
    screen_to_world, world_to_screen, DMat4, DQuat, DVec3, DVec4, Transform, Vec4Swizzles,
};

/// The default snapping distance for rotation in radians
pub const DEFAULT_SNAP_ANGLE: f32 = std::f32::consts::PI / 32.0;
/// The default snapping distance for translation
pub const DEFAULT_SNAP_DISTANCE: f32 = 0.1;
/// The default snapping distance for scale
pub const DEFAULT_SNAP_SCALE: f32 = 0.1;

/// Configuration of a gizmo.
///
/// Defines how the gizmo is drawn to the screen and
/// how it can be interacted with.
#[derive(Debug, Copy, Clone)]
pub struct GizmoConfig {
    /// View matrix for the gizmo, aligning it with the camera's viewpoint.
    pub view_matrix: mint::RowMatrix4<f64>,
    /// Projection matrix for the gizmo, determining how it is projected onto the screen.
    pub projection_matrix: mint::RowMatrix4<f64>,
    /// Screen area where the gizmo is displayed.
    pub viewport: Rect,
    /// The gizmo's operation modes.
    pub modes: EnumSet<GizmoMode>,
    /// If set, this mode is forced active and other modes are disabled
    pub mode_override: Option<GizmoMode>,
    /// Determines the gizmo's orientation relative to global or local axes.
    pub orientation: GizmoOrientation,
    /// Pivot point for transformations
    pub pivot_point: TransformPivotPoint,
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

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            view_matrix: DMat4::IDENTITY.into(),
            projection_matrix: DMat4::IDENTITY.into(),
            viewport: Rect::NOTHING,
            modes: GizmoMode::all(),
            mode_override: None,
            orientation: GizmoOrientation::default(),
            pivot_point: TransformPivotPoint::default(),
            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
            snap_scale: DEFAULT_SNAP_SCALE,
            visuals: GizmoVisuals::default(),
            pixels_per_point: 1.0,
        }
    }
}

impl GizmoConfig {
    /// Forward vector of the view camera
    pub(crate) fn view_forward(&self) -> DVec3 {
        DVec4::from(self.view_matrix.z).xyz()
    }

    /// Up vector of the view camera
    pub(crate) fn view_up(&self) -> DVec3 {
        DVec4::from(self.view_matrix.y).xyz()
    }

    /// Right vector of the view camera
    pub(crate) fn view_right(&self) -> DVec3 {
        DVec4::from(self.view_matrix.x).xyz()
    }

    /// Whether local orientation is used
    pub(crate) fn local_space(&self) -> bool {
        self.orientation() == GizmoOrientation::Local
    }

    /// Transform orientation of the gizmo
    pub(crate) fn orientation(&self) -> GizmoOrientation {
        self.orientation
    }

    /// Whether the modes have changed, compared to given other config
    pub(crate) fn modes_changed(&self, other: &Self) -> bool {
        (self.modes != other.modes && self.mode_override.is_none())
            || (self.mode_override != other.mode_override)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct PreparedGizmoConfig {
    config: GizmoConfig,
    /// Rotation of the gizmo
    pub(crate) rotation: DQuat,
    /// Translation of the gizmo
    pub(crate) translation: DVec3,
    /// Scale of the gizmo
    pub(crate) scale: DVec3,
    /// Combined view-projection matrix
    pub(crate) view_projection: DMat4,
    /// Model matrix from targets
    pub(crate) model_matrix: DMat4,
    /// Combined model-view-projection matrix
    pub(crate) mvp: DMat4,
    /// Scale factor for the gizmo rendering
    pub(crate) scale_factor: f32,
    /// How close the mouse pointer needs to be to a subgizmo before it is focused
    pub(crate) focus_distance: f32,
    /// Whether left-handed projection is used
    pub(crate) left_handed: bool,
    /// Direction from the camera to the gizmo in world space
    pub(crate) eye_to_model_dir: DVec3,
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

impl PreparedGizmoConfig {
    pub(crate) fn update_for_config(&mut self, config: GizmoConfig) {
        let projection_matrix = DMat4::from(config.projection_matrix);
        let view_matrix = DMat4::from(config.view_matrix);

        let view_projection = projection_matrix * view_matrix;

        let left_handed = if projection_matrix.z_axis.w == 0.0 {
            projection_matrix.z_axis.z > 0.0
        } else {
            projection_matrix.z_axis.w > 0.0
        };

        self.config = config;
        self.view_projection = view_projection;
        self.left_handed = left_handed;

        self.update_transform(Transform {
            scale: self.scale.into(),
            rotation: self.rotation.into(),
            translation: self.translation.into(),
        });
    }

    pub(crate) fn update_for_targets(&mut self, targets: &[Transform]) {
        let mut scale = DVec3::ZERO;
        let mut translation = DVec3::ZERO;
        let mut rotation = DQuat::IDENTITY;

        let mut target_count = 0;
        for target in targets {
            scale += DVec3::from(target.scale);
            translation += DVec3::from(target.translation);
            rotation = DQuat::from(target.rotation);

            target_count += 1;
        }

        if target_count == 0 {
            scale = DVec3::ONE;
        } else {
            translation /= target_count as f64;
            scale /= target_count as f64;
        }

        self.update_transform(Transform {
            scale: scale.into(),
            rotation: rotation.into(),
            translation: translation.into(),
        });
    }

    pub(crate) fn update_transform(&mut self, transform: Transform) {
        self.translation = transform.translation.into();
        self.rotation = transform.rotation.into();
        self.scale = transform.scale.into();
        self.model_matrix =
            DMat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation);
        self.mvp = self.view_projection * self.model_matrix;

        self.scale_factor = self.mvp.as_ref()[15] as f32
            / self.projection_matrix.x.x as f32
            / self.config.viewport.width()
            * 2.0;

        let gizmo_screen_pos =
            world_to_screen(self.config.viewport, self.mvp, self.translation).unwrap_or_default();

        let gizmo_view_near = screen_to_world(
            self.config.viewport,
            self.view_projection.inverse(),
            gizmo_screen_pos,
            -1.0,
        );

        self.focus_distance = self.scale_factor * (self.config.visuals.stroke_width / 2.0 + 5.0);

        self.eye_to_model_dir = (gizmo_view_near - self.translation).normalize_or_zero();
    }

    pub(crate) fn as_transform(&self) -> Transform {
        Transform {
            scale: self.scale.into(),
            rotation: self.rotation.into(),
            translation: self.translation.into(),
        }
    }
}

/// Operation mode of a gizmo.
#[derive(Debug, EnumSetType, Hash)]
pub enum GizmoMode {
    /// Rotate around the X axis
    RotateX,
    /// Rotate around the Y axis
    RotateY,
    /// Rotate around the Z axis
    RotateZ,
    /// Rotate around the view forward axis
    RotateView,
    /// Translate along the X axis
    TranslateX,
    /// Translate along the Y axis
    TranslateY,
    /// Translate along the Z axis
    TranslateZ,
    /// Translate along the XY plane
    TranslateXY,
    /// Translate along the XZ plane
    TranslateXZ,
    /// Translate along the YZ plane
    TranslateYZ,
    /// Translate along the view forward axis
    TranslateView,
    /// Scale along the X axis
    ScaleX,
    /// Scale along the Y axis
    ScaleY,
    /// Scale along the Z axis
    ScaleZ,
    /// Scale along the XY plane
    ScaleXY,
    /// Scale along the XZ plane
    ScaleXZ,
    /// Scale along the YZ plane
    ScaleYZ,
    /// Scale uniformly in all directions
    ScaleUniform,
    /// Rotate using an arcball (trackball)
    Arcball,
}

impl GizmoMode {
    /// All modes
    pub fn all() -> EnumSet<Self> {
        EnumSet::all()
    }

    /// All rotation modes
    pub const fn all_rotate() -> EnumSet<Self> {
        enum_set!(Self::RotateX | Self::RotateY | Self::RotateZ | Self::RotateView)
    }

    /// All translation modes
    pub const fn all_translate() -> EnumSet<Self> {
        enum_set!(
            Self::TranslateX
                | Self::TranslateY
                | Self::TranslateZ
                | Self::TranslateXY
                | Self::TranslateXZ
                | Self::TranslateYZ
                | Self::TranslateView
        )
    }

    /// All scaling modes
    pub const fn all_scale() -> EnumSet<Self> {
        enum_set!(
            Self::ScaleX
                | Self::ScaleY
                | Self::ScaleZ
                | Self::ScaleXY
                | Self::ScaleXZ
                | Self::ScaleYZ
                | Self::ScaleUniform
        )
    }

    /// Is this mode for rotation
    pub fn is_rotate(&self) -> bool {
        self.kind() == GizmoModeKind::Rotate
    }

    /// Is this mode for translation
    pub fn is_translate(&self) -> bool {
        self.kind() == GizmoModeKind::Translate
    }

    /// Is this mode for scaling
    pub fn is_scale(&self) -> bool {
        self.kind() == GizmoModeKind::Scale
    }

    /// Axes this mode acts on
    pub fn axes(&self) -> EnumSet<GizmoDirection> {
        match self {
            Self::RotateX | Self::TranslateX | Self::ScaleX => {
                enum_set!(GizmoDirection::X)
            }
            Self::RotateY | Self::TranslateY | Self::ScaleY => {
                enum_set!(GizmoDirection::Y)
            }
            Self::RotateZ | Self::TranslateZ | Self::ScaleZ => {
                enum_set!(GizmoDirection::Z)
            }
            Self::RotateView | Self::TranslateView => {
                enum_set!(GizmoDirection::View)
            }
            Self::ScaleUniform | Self::Arcball => {
                enum_set!(GizmoDirection::X | GizmoDirection::Y | GizmoDirection::Z)
            }
            Self::TranslateXY | Self::ScaleXY => {
                enum_set!(GizmoDirection::X | GizmoDirection::Y)
            }
            Self::TranslateXZ | Self::ScaleXZ => {
                enum_set!(GizmoDirection::X | GizmoDirection::Z)
            }
            Self::TranslateYZ | Self::ScaleYZ => {
                enum_set!(GizmoDirection::Y | GizmoDirection::Z)
            }
        }
    }

    /// Returns the modes that match to given axes exactly
    pub fn all_from_axes(axes: EnumSet<GizmoDirection>) -> EnumSet<Self> {
        EnumSet::<Self>::all()
            .iter()
            .filter(|mode| mode.axes() == axes)
            .collect()
    }

    pub fn kind(&self) -> GizmoModeKind {
        match self {
            Self::RotateX | Self::RotateY | Self::RotateZ | Self::RotateView => {
                GizmoModeKind::Rotate
            }
            Self::TranslateX
            | Self::TranslateY
            | Self::TranslateZ
            | Self::TranslateXY
            | Self::TranslateXZ
            | Self::TranslateYZ
            | Self::TranslateView => GizmoModeKind::Translate,
            Self::ScaleX
            | Self::ScaleY
            | Self::ScaleZ
            | Self::ScaleXY
            | Self::ScaleXZ
            | Self::ScaleYZ
            | Self::ScaleUniform => GizmoModeKind::Scale,
            Self::Arcball => GizmoModeKind::Arcball,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum GizmoModeKind {
    Rotate,
    Translate,
    Scale,
    Arcball,
}

/// The point in space around which all rotations are centered.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum TransformPivotPoint {
    /// Pivot around the median point of targets
    #[default]
    MedianPoint,
    /// Pivot around each target's own origin
    IndividualOrigins,
}

/// Orientation of a gizmo.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum GizmoOrientation {
    /// Transformation axes are aligned to world space.
    #[default]
    Global,
    /// Transformation axes are aligned to the last target's orientation.
    Local,
}

#[derive(Debug, EnumSetType, Hash)]
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
            x_color: Color32::from_rgb(255, 0, 125),
            y_color: Color32::from_rgb(0, 255, 125),
            z_color: Color32::from_rgb(0, 125, 255),
            s_color: Color32::from_rgb(255, 255, 255),
            inactive_alpha: 0.7,
            highlight_alpha: 1.0,
            highlight_color: None,
            stroke_width: 4.0,
            gizmo_size: 75.0,
        }
    }
}
