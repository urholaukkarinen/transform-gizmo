use bevy::{math::DQuat, prelude::*};
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Layout, RichText, Widget},
};
use transform_gizmo_bevy::{config::TransformPivotPoint, prelude::*};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin).add_systems(Update, update_ui);
    }
}

fn update_ui(
    mut contexts: EguiContexts,
    mut gizmo_options: ResMut<GizmoOptions>,
    gizmo_targets: Query<&GizmoTarget>,
) {
    egui::SidePanel::left("options").show(contexts.ctx_mut(), |ui| {
        draw_options(ui, &mut gizmo_options);
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show(contexts.ctx_mut(), |ui| {
            let latest_gizmo_result = gizmo_targets
                .iter()
                .find_map(|target| target.latest_result());

            draw_gizmo_result(ui, latest_gizmo_result);
        });
}

fn draw_gizmo_result(ui: &mut egui::Ui, gizmo_result: Option<GizmoResult>) {
    if let Some(result) = gizmo_result {
        let text = match result {
            GizmoResult::Rotation {
                axis,
                delta: _,
                total,
                is_view_axis: _,
            } => {
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    total.to_degrees()
                )
            }
            GizmoResult::Translation { delta: _, total } => {
                format!(
                    "Translation: ({:.2}, {:.2}, {:.2})",
                    total.x, total.y, total.z,
                )
            }
            GizmoResult::Scale { total } => {
                format!("Scale: ({:.2}, {:.2}, {:.2})", total.x, total.y, total.z,)
            }
            GizmoResult::Arcball { delta: _, total } => {
                let (axis, angle) = DQuat::from(total).to_axis_angle();
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    angle.to_degrees()
                )
            }
        };

        egui::Frame::new()
            .outer_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(text);
            });
    }
}

fn draw_options(ui: &mut egui::Ui, gizmo_options: &mut GizmoOptions) {
    ui.heading("Options");
    ui.separator();

    egui::Grid::new("modes_grid").num_columns(7).show(ui, |ui| {
        ui.label(RichText::new("Mode").strong());
        ui.label(RichText::new("View").strong());
        ui.label(RichText::new("X").strong());
        ui.label(RichText::new("Y").strong());
        ui.label(RichText::new("Z").strong());
        ui.label(RichText::new("XZ").strong());
        ui.label(RichText::new("XY").strong());
        ui.label(RichText::new("YZ").strong());
        ui.end_row();

        ui.label("Rotation");
        draw_mode_picker(ui, GizmoMode::RotateView, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::RotateX, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::RotateY, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::RotateZ, &mut gizmo_options.gizmo_modes);
        ui.end_row();

        ui.label("Translation");
        draw_mode_picker(ui, GizmoMode::TranslateView, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::TranslateX, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::TranslateY, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::TranslateZ, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::TranslateXZ, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::TranslateXY, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::TranslateYZ, &mut gizmo_options.gizmo_modes);
        ui.end_row();

        ui.label("Scale");
        ui.add_enabled_ui(
            !gizmo_options.gizmo_modes.contains(GizmoMode::RotateView),
            |ui| {
                draw_mode_picker(ui, GizmoMode::ScaleUniform, &mut gizmo_options.gizmo_modes);
            },
        );
        draw_mode_picker(ui, GizmoMode::ScaleX, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::ScaleY, &mut gizmo_options.gizmo_modes);
        draw_mode_picker(ui, GizmoMode::ScaleZ, &mut gizmo_options.gizmo_modes);
        ui.add_enabled_ui(
            !gizmo_options.gizmo_modes.contains(GizmoMode::TranslateXZ),
            |ui| {
                draw_mode_picker(ui, GizmoMode::ScaleXZ, &mut gizmo_options.gizmo_modes);
            },
        );
        ui.add_enabled_ui(
            !gizmo_options.gizmo_modes.contains(GizmoMode::TranslateXY),
            |ui| {
                draw_mode_picker(ui, GizmoMode::ScaleXY, &mut gizmo_options.gizmo_modes);
            },
        );
        ui.add_enabled_ui(
            !gizmo_options.gizmo_modes.contains(GizmoMode::TranslateYZ),
            |ui| {
                draw_mode_picker(ui, GizmoMode::ScaleYZ, &mut gizmo_options.gizmo_modes);
            },
        );
        ui.end_row();

        ui.label("Arcball");
        draw_mode_picker(ui, GizmoMode::Arcball, &mut gizmo_options.gizmo_modes);
        ui.end_row();
    });

    ui.separator();

    egui::Grid::new("options_grid")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Orientation");
            egui::ComboBox::from_id_salt("orientation_cb")
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

            ui.label("Pivot point");
            egui::ComboBox::from_id_salt("pivot_cb")
                .selected_text(format!("{:?}", gizmo_options.pivot_point))
                .show_ui(ui, |ui| {
                    for pivot_point in [
                        TransformPivotPoint::MedianPoint,
                        TransformPivotPoint::IndividualOrigins,
                    ] {
                        ui.selectable_value(
                            &mut gizmo_options.pivot_point,
                            pivot_point,
                            format!("{:?}", pivot_point),
                        );
                    }
                });
            ui.end_row();

            ui.label("Group targets");
            egui::Checkbox::without_text(&mut gizmo_options.group_targets).ui(ui);
            ui.end_row();
        });

    ui.separator();
    ui.heading("Visuals");
    ui.separator();

    egui::Grid::new("visuals_grid")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Gizmo size");
            egui::Slider::new(&mut gizmo_options.visuals.gizmo_size, 10.0..=200.0).ui(ui);
            ui.end_row();

            ui.label("Stroke width");
            egui::Slider::new(&mut gizmo_options.visuals.stroke_width, 1.0..=15.0).ui(ui);
            ui.end_row();

            ui.label("Inactive alpha");
            egui::Slider::new(&mut gizmo_options.visuals.inactive_alpha, 0.0..=1.0).ui(ui);
            ui.end_row();

            ui.label("Highlight alpha");
            egui::Slider::new(&mut gizmo_options.visuals.highlight_alpha, 0.0..=1.0).ui(ui);
            ui.end_row();

            ui.label("X axis color");
            draw_color_picker(ui, &mut gizmo_options.visuals.x_color);
            ui.end_row();

            ui.label("Y axis color");
            draw_color_picker(ui, &mut gizmo_options.visuals.y_color);
            ui.end_row();

            ui.label("Z axis color");
            draw_color_picker(ui, &mut gizmo_options.visuals.z_color);
            ui.end_row();

            ui.label("View axis color");
            draw_color_picker(ui, &mut gizmo_options.visuals.s_color);
            ui.end_row();
        });

    ui.separator();

    ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
        egui::Hyperlink::from_label_and_url("(source code)", "https://github.com/urholaukkarinen/transform-gizmo/blob/main/examples/bevy/src/main.rs").ui(ui);

        ui.label(r#"Move and rotate the camera using the middle and right mouse buttons.
Toggle gizmo snapping with left ctrl & shift.
You can enter transform mode for translation, rotation and scale with by pressing G, R or S respectively.
Transform mode can be exited with Esc or by pressing any mouse button."#);
    });
}

fn draw_mode_picker(ui: &mut egui::Ui, mode: GizmoMode, all_modes: &mut EnumSet<GizmoMode>) {
    let mut checked = all_modes.contains(mode);

    egui::Checkbox::without_text(&mut checked).ui(ui);

    if checked {
        all_modes.insert(mode);
    } else {
        all_modes.remove(mode);
    }
}

fn draw_color_picker(ui: &mut egui::Ui, color: &mut Color32) {
    let mut egui_color =
        egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), color.a());

    let color_picker = egui::color_picker::color_edit_button_srgba(
        ui,
        &mut egui_color,
        egui::color_picker::Alpha::Opaque,
    );

    if color_picker.changed() {
        *color = Color32::from_rgba_premultiplied(
            egui_color.r(),
            egui_color.g(),
            egui_color.b(),
            egui_color.a(),
        );
    }
}
