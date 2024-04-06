use eframe::{egui, NativeOptions};
use transform_gizmo_egui::math::{DQuat, Transform};
use transform_gizmo_egui::{
    math::{DMat4, DVec3},
    *,
};

struct ExampleApp {
    gizmo: Gizmo,

    gizmo_modes: EnumSet<GizmoMode>,
    gizmo_orientation: GizmoOrientation,

    scale: DVec3,
    rotation: DQuat,
    translation: DVec3,
}

impl ExampleApp {
    fn new() -> Self {
        Self {
            gizmo: Gizmo::default(),
            gizmo_modes: enum_set!(GizmoMode::Rotate | GizmoMode::Translate),
            gizmo_orientation: GizmoOrientation::Local,
            scale: DVec3::ONE,
            rotation: DQuat::IDENTITY,
            translation: DVec3::ZERO,
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
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            viewport,
            modes: self.gizmo_modes,
            orientation: self.gizmo_orientation,
            snapping,
            ..Default::default()
        });

        let mut transform =
            Transform::from_scale_rotation_translation(self.scale, self.rotation, self.translation);

        if let Some((result, new_transforms)) = self.gizmo.interact(ui, &[transform]) {
            for (new_transform, transform) in
                new_transforms.iter().zip(std::iter::once(&mut transform))
            {
                *transform = *new_transform;
            }

            self.scale = transform.scale.into();
            self.rotation = transform.rotation.into();
            self.translation = transform.translation.into();

            match result {
                GizmoResult::Rotation { delta: _, total } => {
                    let (axis, angle) = DQuat::from(total).to_axis_angle();
                    ui.label(format!(
                        "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                        axis.x,
                        axis.y,
                        axis.z,
                        angle.to_degrees()
                    ));
                }
                GizmoResult::Translation { delta: _, total } => {
                    ui.label(format!(
                        "Translation: ({:.2}, {:.2}, {:.2})",
                        total.x, total.y, total.z,
                    ));
                }
                GizmoResult::Scale { total } => {
                    ui.label(format!(
                        "Scale: ({:.2}, {:.2}, {:.2})",
                        total.x, total.y, total.z,
                    ));
                }
            }
        }
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
