use bevy::{math::DQuat, prelude::*, render::camera::Viewport};
use bevy_egui::{
    egui::{self, Layout, Widget},
    EguiContexts, EguiPlugin,
};
use transform_gizmo_bevy::{
    config::{DEFAULT_SNAP_ANGLE, DEFAULT_SNAP_DISTANCE, DEFAULT_SNAP_SCALE},
    prelude::*,
};

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
    keyboard_input: Res<ButtonInput<KeyCode>>,

    gizmo_targets: Query<&GizmoTarget>,
) {
    // Snapping is enabled when CTRL is pressed.
    let snapping = keyboard_input.pressed(KeyCode::ControlLeft);
    // Accurate snapping is enabled when both CTRL and SHIFT are pressed
    let accurate_snapping = snapping && keyboard_input.pressed(KeyCode::ShiftLeft);

    gizmo_options.snapping = snapping;

    gizmo_options.snap_angle = DEFAULT_SNAP_ANGLE;
    gizmo_options.snap_distance = DEFAULT_SNAP_DISTANCE;
    gizmo_options.snap_scale = DEFAULT_SNAP_SCALE;

    if accurate_snapping {
        gizmo_options.snap_angle /= 2.0;
        gizmo_options.snap_distance /= 2.0;
        gizmo_options.snap_scale /= 2.0;
    }

    egui::SidePanel::left("options").show(contexts.ctx_mut(), |ui| {
        draw_options(ui, &mut gizmo_options);
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

                let latest_gizmo_result =
                    gizmo_targets.iter().find_map(|target| target.latest_result);

                draw_gizmo_result(ui, latest_gizmo_result);
            });
        });
}

fn draw_gizmo_result(ui: &mut egui::Ui, gizmo_result: Option<GizmoResult>) {
    if let Some(result) = gizmo_result {
        let text = match result {
            GizmoResult::Rotation { delta: _, total } => {
                let (axis, angle) = DQuat::from(total).to_axis_angle();
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    angle.to_degrees()
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
        };

        egui::Frame::none()
            .outer_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(text);
            });
    }
}

fn draw_options(ui: &mut egui::Ui, gizmo_options: &mut GizmoOptions) {
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

    ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
        egui::Hyperlink::from_label_and_url("(source code)", "https://github.com/urholaukkarinen/transform-gizmo/blob/main/crates/transform-gizmo-demo/src/main.rs").ui(ui);

        ui.label("Move and rotate the camera using the middle and right mouse buttons");
        ui.label("Toggle gizmo snapping with left ctrl & shift");
    });
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
