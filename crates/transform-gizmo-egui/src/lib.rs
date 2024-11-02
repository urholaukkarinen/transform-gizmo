//! Provides a 3D transformation gizmo for the Egui library.
//!
//! transform-gizmo-egui provides a feature-rich and configurable gizmo
//! that can be used for 3d transformations (translation, rotation, scale).
//!
//! # Usage
//!
//! Create a new `Gizmo` instance once.
//!
//! ```
//! use transform_gizmo_egui::prelude::*;
//!
//! let gizmo = Gizmo::default();
//! ```
//!
//! Update the gizmo configuration as needed, for example, when the camera moves.
//!
//! ```ignore
//! gizmo.update_config(GizmoConfig {
//!     view_matrix: view_matrix.into(),
//!     projection_matrix: projection_matrix.into(),
//!     modes: GizmoMode::all(),
//!     orientation: GizmoOrientation::Local,
//!     ..Default::default()
//! });
//! ```
//!
//! Finally, interact with the gizmo. The function takes a slice of transforms as an
//! input. The result is [`Some`] if the gizmo was successfully interacted with this frame.
//! In the result you can find the modified transforms, in the same order as was given to the function
//! as arguments.
//!
//! ```ignore
//!  let mut transform = Transform::from_scale_rotation_translation(scale, rotation, translation);
//!
//!  if let Some((result, new_transforms)) = gizmo.interact(ui, &[transform]) {
//!      for (new_transform, transform) in
//!          new_transforms.iter().zip(std::iter::once(&mut transform))
//!      {
//!          // Apply the modified transforms
//!          *transform = *new_transform;
//!      }
//!  }
//! ```
//!
//!
use egui::{epaint::Vertex, Mesh, PointerButton, Pos2, Rgba, Sense, Ui, Vec2};

use transform_gizmo::math::Transform;
pub use transform_gizmo::*;
pub mod prelude;

pub trait GizmoExt {
    /// Interact with the gizmo and draw it to Ui.
    ///
    /// Returns result of the gizmo interaction.
    fn interact(&mut self, ui: &Ui, targets: &[Transform])
        -> Option<(GizmoResult, Vec<Transform>)>;
}

impl GizmoExt for Gizmo {
    fn interact(
        &mut self,
        ui: &Ui,
        targets: &[Transform],
    ) -> Option<(GizmoResult, Vec<Transform>)> {
        let cursor_pos = ui
            .input(|input| input.pointer.hover_pos())
            .unwrap_or_default();

        let mut viewport = self.config().viewport;
        if !viewport.is_finite() {
            viewport = ui.clip_rect();
        }

        let egui_viewport = Rect {
            min: Pos2::new(viewport.min.x, viewport.min.y),
            max: Pos2::new(viewport.max.x, viewport.max.y),
        };

        self.update_config(GizmoConfig {
            viewport,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..*self.config()
        });

        let interaction = ui.interact(
            Rect::from_center_size(cursor_pos, Vec2::splat(1.0)),
            ui.id().with("_interaction"),
            Sense::click_and_drag(),
        );
        let hovered = interaction.hovered();

        let gizmo_result = self.update(
            GizmoInteraction {
                cursor_pos: (cursor_pos.x, cursor_pos.y),
                hovered,
                drag_started: ui
                    .input(|input| input.pointer.button_pressed(PointerButton::Primary)),
                dragging: ui.input(|input| input.pointer.button_down(PointerButton::Primary)),
            },
            targets,
        );

        let draw_data = self.draw();

        egui::Painter::new(ui.ctx().clone(), ui.layer_id(), egui_viewport).add(Mesh {
            indices: draw_data.indices,
            vertices: draw_data
                .vertices
                .into_iter()
                .zip(draw_data.colors)
                .map(|(pos, [r, g, b, a])| Vertex {
                    pos: pos.into(),
                    uv: Pos2::default(),
                    color: Rgba::from_rgba_premultiplied(r, g, b, a).into(),
                })
                .collect(),
            ..Default::default()
        });

        gizmo_result
    }
}
