use bevy::{prelude::*, render::camera::Viewport};
use bevy_egui_next::{
    egui::{self, Widget},
    EguiContexts, EguiPlugin,
};
use transform_gizmo_bevy::prelude::*;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin).add_systems(Update, update_ui);
    }
}

fn update_ui(
    mut contexts: EguiContexts,
    mut gizmo_options: ResMut<GizmoOptions>,
    mut camera: Query<&mut Camera>,
) {
    egui::SidePanel::left("options").show(contexts.ctx_mut(), |ui| {
        ui.heading("Options");
        ui.separator();

        egui::Grid::new("options_grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Modes");
                egui::ComboBox::from_id_source("mode_cb")
                    .selected_text(format!("{}", gizmo_options.gizmo_modes.len()))
                    .show_ui(ui, |ui| {
                        for mode in [GizmoMode::Rotate, GizmoMode::Translate, GizmoMode::Scale] {
                            let mut mode_selected = gizmo_options.gizmo_modes.contains(mode);
                            ui.toggle_value(&mut mode_selected, format!("{:?}", mode));
                            if mode_selected {
                                gizmo_options.gizmo_modes.insert(mode);
                            } else {
                                gizmo_options.gizmo_modes.remove(mode);
                            }
                        }
                    });
                ui.end_row();

                ui.label("Orientation");
                egui::ComboBox::from_id_source("orientation_cb")
                    .selected_text(format!("{:?}", gizmo_options.gizmo_orientation))
                    .show_ui(ui, |ui| {
                        for orientation in [GizmoOrientation::Global, GizmoOrientation::Local] {
                            ui.selectable_value(
                                &mut gizmo_options.gizmo_orientation,
                                orientation,
                                format!("{:?}", orientation),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Toggle grouping");
                egui::Checkbox::without_text(&mut gizmo_options.group_targets).ui(ui);
                ui.end_row();
            });
    });

    // Use a transparent panel as the camera viewport
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(contexts.ctx_mut(), |ui| {
            ui.allocate_ui(ui.available_size(), |ui| {
                let clip_rect = ui.clip_rect();

                let mut camera = camera.single_mut();
                camera.viewport = Some(Viewport {
                    physical_position: UVec2::new(clip_rect.left() as _, clip_rect.top() as _),
                    physical_size: UVec2::new(clip_rect.width() as _, clip_rect.height() as _),
                    ..default()
                });
            });
        });
}
