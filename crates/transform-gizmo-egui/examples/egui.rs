use eframe::{egui, NativeOptions};
use transform_gizmo::math::{DMat4, DVec3};
use transform_gizmo::{enum_set, prelude::*, EnumSet};
use transform_gizmo_egui::GizmoExt;

struct ExampleApp {
    gizmo: Gizmo,

    gizmo_modes: EnumSet<GizmoMode>,
    gizmo_orientation: GizmoOrientation,

    model_matrix: DMat4,
}

impl ExampleApp {
    fn new() -> Self {
        Self {
            gizmo: Gizmo::default(),
            gizmo_modes: enum_set!(GizmoMode::Rotate),
            gizmo_orientation: GizmoOrientation::Global,
            model_matrix: DMat4::IDENTITY,
        }
    }

    fn draw_gizmo(&mut self, ui: &mut egui::Ui) {
        // The whole clipping area of the UI is used as viewport
        let viewport = ui.clip_rect();

        let projection_matrix = DMat4::perspective_infinite_reverse_lh(
            std::f64::consts::PI / 4.0,
            (viewport.width() / viewport.height()).into(),
            0.1,
        );

        // Fixed camera position
        let view_matrix = DMat4::look_at_lh(DVec3::splat(5.0), DVec3::ZERO, DVec3::Y);

        // Ctrl toggles snapping
        let snapping = ui.input(|input| input.modifiers.ctrl);

        self.gizmo.update_config(GizmoConfig {
            model_matrix: self.model_matrix,
            view_matrix,
            projection_matrix,
            viewport,
            modes: self.gizmo_modes,
            orientation: self.gizmo_orientation,
            snapping,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..Default::default()
        });

        if let Some(result) = self.gizmo.interact(ui) {
            println!("{result:#?}");
        }

        self.model_matrix = self.gizmo.config().model_matrix;
    }

    fn draw_options(&mut self, ui: &mut egui::Ui) {
        ui.heading("Options");
        ui.separator();

        egui::Grid::new("options_grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Modes");
                egui::ComboBox::from_id_source("mode_cb")
                    .selected_text(format!("{}", self.gizmo_modes.len()))
                    .show_ui(ui, |ui| {
                        for mode in [GizmoMode::Rotate, GizmoMode::Translate, GizmoMode::Scale] {
                            let mut mode_selected = self.gizmo_modes.contains(mode);
                            ui.toggle_value(&mut mode_selected, format!("{:?}", mode));
                            if mode_selected {
                                self.gizmo_modes.insert(mode);
                            } else {
                                self.gizmo_modes.remove(mode);
                            }
                        }
                    });
                ui.end_row();

                ui.label("Orientation");
                egui::ComboBox::from_id_source("orientation_cb")
                    .selected_text(format!("{:?}", self.gizmo_orientation))
                    .show_ui(ui, |ui| {
                        for orientation in [GizmoOrientation::Global, GizmoOrientation::Local] {
                            ui.selectable_value(
                                &mut self.gizmo_orientation,
                                orientation,
                                format!("{:?}", orientation),
                            );
                        }
                    });
                ui.end_row();
            });
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("options_panel").show(ctx, |ui| {
            self.draw_options(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_gizmo(ui);
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "transform_gizmo_egui example",
        NativeOptions::default(),
        Box::new(|_| Box::new(ExampleApp::new())),
    )
}
