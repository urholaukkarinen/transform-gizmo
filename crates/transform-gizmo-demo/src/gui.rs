use bevy::{
    app::{Plugin, Update},
    ecs::system::ResMut,
};
use bevy_egui_next::{egui, EguiContexts, EguiPlugin};
use transform_gizmo_bevy::prelude::*;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin).add_systems(Update, update_ui);
    }
}

fn update_ui(mut contexts: EguiContexts, mut gizmo_options: ResMut<GizmoOptions>) {
    egui::Window::new("Options").show(contexts.ctx_mut(), |ui| {
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
            });
    });
}
