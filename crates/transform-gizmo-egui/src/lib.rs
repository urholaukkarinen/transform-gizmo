//! Provides a 3D transformation gizmo for the Egui library.
//!
//! transform-gizmo-egui provides a feature-rich and configurable 3D transformation
//! gizmo that can be used to manipulate 4x4 transformation matrices (position, rotation, scale)
//! visually.
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
//! When drawing the gui, update the gizmo configuration.
//!
//! ```ignore
//! gizmo.update_config(GizmoConfig {
//!     view_matrix: view_matrix.into(),
//!     projection_matrix: projection_matrix.into(),
//!     modes: enum_set!(GizmoMode::Rotate | GizmoMode::Translate | GizmoMode::Scale),
//!     orientation: GizmoOrientation::Local,
//!     ..Default::default()
//! });
//! ```
//!
//! Finally, interact with the gizmo. The function takes a slice of matrices as an
//! input. The result is [`Some`] if the gizmo was successfully interacted with this frame.
//! In the result you can find the modified matrices, in the same order as was given to the function
//! as arguments.
//!
//! ```ignore
//!  if let Some(result) = gizmo.interact(ui, &[model_matrix.into()]) {
//!      model_matrix = result.targets.first().copied().unwrap().into();
//!  }
//! ```
//!
//!
use egui::{epaint::Vertex, Mesh, PointerButton, Pos2, Rgba, Ui};

pub use transform_gizmo::*;
pub mod prelude;

pub trait GizmoExt {
    /// Interact with the gizmo and draw it to Ui.
    ///
    /// Returns result of the gizmo interaction.
    fn interact(
        &mut self,
        ui: &Ui,
        targets: &[mint::RowMatrix4<f64>],
    ) -> Option<(GizmoResult, Vec<mint::RowMatrix4<f64>>)>;
}

impl GizmoExt for Gizmo {
    fn interact(
        &mut self,
        ui: &Ui,
        targets: &[mint::RowMatrix4<f64>],
    ) -> Option<(GizmoResult, Vec<mint::RowMatrix4<f64>>)> {
        let config = self.config();

        let egui_viewport = egui::Rect {
            min: Pos2::new(config.viewport.min.x, config.viewport.min.y),
            max: Pos2::new(config.viewport.max.x, config.viewport.max.y),
        };

        let cursor_pos = ui
            .input(|input| input.pointer.hover_pos())
            .unwrap_or_default();

        let mut viewport = self.config().viewport;
        if !viewport.is_finite() {
            viewport = ui.clip_rect();
        }

        self.update_config(GizmoConfig {
            viewport,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..*self.config()
        });

        let gizmo_result = self.update(
            GizmoInteraction {
                cursor_pos: (cursor_pos.x, cursor_pos.y),
                drag_started: ui
                    .input(|input| input.pointer.button_pressed(PointerButton::Primary)),
                dragging: ui.input(|input| input.pointer.button_down(PointerButton::Primary)),
            },
            targets,
        );

        let draw_data = self.draw();

        ui.painter().with_clip_rect(egui_viewport).add(Mesh {
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
