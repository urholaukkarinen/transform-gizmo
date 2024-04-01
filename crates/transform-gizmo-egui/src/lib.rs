use egui::{epaint::Vertex, Color32, Mesh, PointerButton, Pos2, Ui};

use transform_gizmo::prelude::*;

pub trait GizmoExt {
    /// Interact with the gizmo and draw it to Ui.
    ///
    /// Returns result of the gizmo interaction.
    fn interact(
        &mut self,
        ui: &Ui,
        targets: impl Iterator<Item = mint::RowMatrix4<f64>>,
    ) -> Option<GizmoResult>;
}

impl GizmoExt for Gizmo {
    fn interact(
        &mut self,
        ui: &Ui,
        targets: impl Iterator<Item = mint::RowMatrix4<f64>>,
    ) -> Option<GizmoResult> {
        let config = self.config();

        let egui_viewport = egui::Rect {
            min: Pos2::new(config.viewport.min.x, config.viewport.min.y),
            max: Pos2::new(config.viewport.max.x, config.viewport.max.y),
        };

        let cursor_pos = ui
            .input(|input| input.pointer.hover_pos())
            .unwrap_or_default();

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
                    color: Color32::from_rgba_premultiplied(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                        (a * 255.0) as u8,
                    ),
                })
                .collect(),
            ..Default::default()
        });

        gizmo_result
    }
}
