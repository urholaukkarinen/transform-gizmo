use emath::Pos2;
use std::ops::{Add, AddAssign, Sub};

use crate::config::{GizmoConfig, GizmoDirection, GizmoMode, PreparedGizmoConfig};
use crate::math::screen_to_world;
use epaint::Mesh;
use glam::{DMat4, DVec3, Mat4, Quat, Vec3};

use crate::subgizmo::rotation::RotationParams;
use crate::subgizmo::scale::ScaleParams;
use crate::subgizmo::translation::TranslationParams;
use crate::subgizmo::{
    common::TransformKind, ArcballSubGizmo, RotationSubGizmo, ScaleSubGizmo, SubGizmo,
    TranslationSubGizmo,
};

pub struct Gizmo {
    config: PreparedGizmoConfig,
    last_mode: Option<GizmoMode>,
    subgizmos: Vec<Box<dyn SubGizmo + 'static>>,
    active_subgizmo_id: Option<u64>,
}

impl Default for Gizmo {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl Gizmo {
    pub fn new(config: GizmoConfig) -> Self {
        Self {
            config: PreparedGizmoConfig::from_config(config),
            last_mode: None,
            subgizmos: Default::default(),
            active_subgizmo_id: None,
        }
    }

    /// Current configuration used by the gizmo.
    pub fn config(&self) -> &GizmoConfig {
        &self.config
    }

    /// Update the configuration used by the gizmo.
    pub fn update_config(&mut self, config: GizmoConfig) {
        self.config = PreparedGizmoConfig::from_config(config);
    }

    /// Were any of the subgizmoes focused after latest [`Gizmo::update`] call.
    pub fn is_any_focused(&self) -> bool {
        self.subgizmos.iter().any(|subgizmo| subgizmo.is_focused())
    }

    /// Updates the gizmo based on given interaction information.
    ///
    /// Returns the result of the interaction with the updated transformation.
    ///
    /// [`Some`] is returned when any of the subgizmos is being dragged, [`None`] otherwise.
    pub fn update(&mut self, interaction: GizmoInteraction) -> Option<GizmoResult> {
        if !self.config.viewport.is_finite() {
            return None;
        }

        // Mode was changed. Update all subgizmos accordingly.
        if Some(self.config.mode) != self.last_mode {
            self.last_mode = Some(self.config.mode);

            self.subgizmos.clear();

            // Choose subgizmos based on the gizmo mode
            match self.config.mode {
                GizmoMode::Rotate => {
                    self.add_subgizmos(self.new_rotation());
                    self.add_subgizmos(self.new_arcball());
                }
                GizmoMode::Translate => self.add_subgizmos(self.new_translation()),
                GizmoMode::Scale => self.add_subgizmos(self.new_scale()),
            };
        }

        for subgizmo in &mut self.subgizmos {
            // Update current configuration to each subgizmo.
            subgizmo.update_config(self.config);
            // All subgizmoes are initially considered unfocused.
            subgizmo.set_focused(false);
        }

        let mut result = None;

        let pointer_ray = self.pointer_ray(Pos2::from(interaction.cursor_pos));

        // If there is no active subgizmo, find which one of them
        // is under the mouse pointer, if any.
        if self.active_subgizmo_id.is_none() {
            if let Some(subgizmo) = self.pick_subgizmo(pointer_ray) {
                subgizmo.set_focused(true);

                // If we started dragging from one of the subgizmos, mark it as active.
                if interaction.drag_started {
                    self.active_subgizmo_id = Some(subgizmo.id());
                }
            }
        }

        let mut active_subgizmo = self.active_subgizmo_id.and_then(|id| {
            self.subgizmos
                .iter_mut()
                .find(|subgizmo| subgizmo.id() == id)
        });

        if let Some(subgizmo) = active_subgizmo.as_mut() {
            if interaction.dragging {
                subgizmo.set_active(true);
                subgizmo.set_focused(true);
                result = subgizmo.update(pointer_ray);
            } else {
                subgizmo.set_active(false);
                subgizmo.set_focused(false);
                self.active_subgizmo_id = None;
            }
        }

        // Update current configuration based on the interaction result.
        if let Some((_, result)) = active_subgizmo.zip(result) {
            self.config.translation = Vec3::from(result.translation).as_dvec3();
            self.config.rotation = Quat::from(result.rotation).as_dquat();
            self.config.scale = Vec3::from(result.scale).as_dvec3();

            self.config.model_matrix = DMat4::from_scale_rotation_translation(
                self.config.scale,
                self.config.rotation,
                self.config.translation,
            );
        }

        result
    }

    /// Add given subgizmos to this gizmo
    fn add_subgizmos<T: SubGizmo + 'static, const N: usize>(&mut self, subgizmos: [T; N]) {
        for subgizmo in subgizmos {
            self.subgizmos.push(Box::new(subgizmo));
        }
    }

    /// Return all the necessary data to draw the latest gizmo interaction.
    ///
    /// The gizmo draw data consists of vertices that are pre-transformed
    /// to normalized device coordinates using the matrices provided in the
    /// gizmo configuration, and are thus easy to draw to the screen.
    pub fn draw(&self) -> GizmoDrawData {
        let mut result = GizmoDrawData::default();
        for subgizmo in &self.subgizmos {
            if self.active_subgizmo_id.is_none() || subgizmo.is_active() {
                result += subgizmo.draw();
            }
        }

        result
    }

    /// Picks the subgizmo that is closest to the given world space ray.
    fn pick_subgizmo(&mut self, ray: Ray) -> Option<&mut Box<dyn SubGizmo>> {
        self.subgizmos
            .iter_mut()
            .filter_map(|subgizmo| subgizmo.pick(ray).map(|t| (t, subgizmo)))
            .min_by(|(first, _), (second, _)| {
                first
                    .partial_cmp(second)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, subgizmo)| subgizmo)
    }

    /// Create subgizmos for arcball rotation
    fn new_arcball(&self) -> [ArcballSubGizmo; 1] {
        [ArcballSubGizmo::new(self.config, ())]
    }

    /// Create subgizmos for rotation
    fn new_rotation(&self) -> [RotationSubGizmo; 4] {
        [
            RotationSubGizmo::new(
                self.config,
                RotationParams {
                    direction: GizmoDirection::X,
                },
            ),
            RotationSubGizmo::new(
                self.config,
                RotationParams {
                    direction: GizmoDirection::Y,
                },
            ),
            RotationSubGizmo::new(
                self.config,
                RotationParams {
                    direction: GizmoDirection::Z,
                },
            ),
            RotationSubGizmo::new(
                self.config,
                RotationParams {
                    direction: GizmoDirection::View,
                },
            ),
        ]
    }

    /// Create subgizmos for translation
    fn new_translation(&self) -> [TranslationSubGizmo; 7] {
        [
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::View,
                    transform_kind: TransformKind::Plane,
                },
            ),
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::X,
                    transform_kind: TransformKind::Axis,
                },
            ),
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::Y,
                    transform_kind: TransformKind::Axis,
                },
            ),
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::Z,
                    transform_kind: TransformKind::Axis,
                },
            ),
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::X,
                    transform_kind: TransformKind::Plane,
                },
            ),
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::Y,
                    transform_kind: TransformKind::Plane,
                },
            ),
            TranslationSubGizmo::new(
                self.config,
                TranslationParams {
                    direction: GizmoDirection::Z,
                    transform_kind: TransformKind::Plane,
                },
            ),
        ]
    }

    /// Create subgizmos for scale
    fn new_scale(&self) -> [ScaleSubGizmo; 7] {
        [
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::View,
                    transform_kind: TransformKind::Plane,
                },
            ),
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::X,
                    transform_kind: TransformKind::Axis,
                },
            ),
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::Y,
                    transform_kind: TransformKind::Axis,
                },
            ),
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::Z,
                    transform_kind: TransformKind::Axis,
                },
            ),
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::X,
                    transform_kind: TransformKind::Plane,
                },
            ),
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::Y,
                    transform_kind: TransformKind::Plane,
                },
            ),
            ScaleSubGizmo::new(
                self.config,
                ScaleParams {
                    direction: GizmoDirection::Z,
                    transform_kind: TransformKind::Plane,
                },
            ),
        ]
    }

    /// Calculate a world space ray from given screen space position
    fn pointer_ray(&self, screen_pos: Pos2) -> Ray {
        let mat = self.config.view_projection.inverse();
        let origin = screen_to_world(self.config.viewport, mat, screen_pos, -1.0);
        let target = screen_to_world(self.config.viewport, mat, screen_pos, 1.0);

        let direction = target.sub(origin).normalize();

        Ray {
            screen_pos,
            origin,
            direction,
        }
    }
}

/// Information needed for interacting with the gizmo.
#[derive(Default, Clone, Copy, Debug)]
pub struct GizmoInteraction {
    /// Current cursor position in window coordinates.
    pub cursor_pos: (f32, f32),
    /// Whether dragging was started this frame.
    /// Usually this is set to true if the primary mouse
    /// button was just pressed.
    pub drag_started: bool,
    /// Whether the user is currently dragging.
    /// Usually this is set to true whenever the primary mouse
    /// button is being pressed.
    pub dragging: bool,
}

/// Result of a gizmo transformation
#[derive(Debug, Copy, Clone)]
pub struct GizmoResult {
    /// Updated scale
    pub scale: mint::Vector3<f32>,
    /// Updated rotation
    pub rotation: mint::Quaternion<f32>,
    /// Updated translation
    pub translation: mint::Vector3<f32>,
    /// Mode of the active subgizmo
    pub mode: GizmoMode,
    /// Total scale, rotation or translation of the current gizmo activation, depending on mode
    pub value: Option<[f32; 3]>,
}

impl GizmoResult {
    /// Updated transformation matrix in column major order.
    pub fn transform(&self) -> mint::ColumnMatrix4<f32> {
        Mat4::from_scale_rotation_translation(
            self.scale.into(),
            self.rotation.into(),
            self.translation.into(),
        )
        .into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub screen_pos: Pos2,
    pub origin: DVec3,
    pub direction: DVec3,
}

#[derive(Default, Clone, Debug)]
pub struct GizmoDrawData {
    pub vertices: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl From<Mesh> for GizmoDrawData {
    fn from(mesh: Mesh) -> Self {
        let (vertices, colors): (Vec<_>, Vec<_>) = mesh
            .vertices
            .iter()
            .map(|vertex| {
                (
                    [vertex.pos.x, vertex.pos.y],
                    vertex.color.to_normalized_gamma_f32(),
                )
            })
            .unzip();

        Self {
            vertices,
            colors,
            indices: mesh.indices,
        }
    }
}

impl AddAssign for GizmoDrawData {
    fn add_assign(&mut self, rhs: Self) {
        let index_offset = self.vertices.len() as u32;
        self.vertices.extend(rhs.vertices);
        self.colors.extend(rhs.colors);
        self.indices
            .extend(rhs.indices.into_iter().map(|idx| index_offset + idx));
    }
}

impl Add for GizmoDrawData {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}
