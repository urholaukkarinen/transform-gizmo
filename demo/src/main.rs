use std::f32::consts::FRAC_PI_4;

use egui::color_picker::Alpha;
use egui::{pos2, Align2, Color32, FontId, LayerId, Ui, Widget};
use macroquad::prelude::*;

use egui_gizmo::{
    Gizmo, GizmoMode, GizmoOrientation, GizmoResult, GizmoVisuals, DEFAULT_SNAP_ANGLE,
    DEFAULT_SNAP_DISTANCE,
};

const SOURCE_URL: &str = "https://github.com/urholaukkarinen/egui-gizmo/blob/main/demo/src/main.rs";

#[macroquad::main("3D")]
async fn main() {
    let texture =
        Texture2D::from_file_with_format(include_bytes!("../crate.png"), Some(ImageFormat::Png));

    let mut camera_angle: f32 = -FRAC_PI_4;
    let mut camera_y = 5.0;

    let target_pos = vec3(0., 0., 0.);

    let mut model_matrix = Mat4::from_scale_rotation_translation(
        vec3(2.0, 2.0, 2.0),
        Quat::from_axis_angle(Vec3::Y, 0.0),
        target_pos,
    );

    let mut gizmo_mode = GizmoMode::Rotate;
    let mut gizmo_orientation = GizmoOrientation::Global;
    let mut last_gizmo_response = None;

    let mut stroke_width = 4.0;
    let mut gizmo_size = 75.0;
    let mut custom_highlight_color = false;
    let mut highlight_color = Color32::GOLD;
    let mut x_color = Color32::from_rgb(255, 0, 148);
    let mut y_color = Color32::from_rgb(148, 255, 0);
    let mut z_color = Color32::from_rgb(0, 148, 255);
    let mut s_color = Color32::WHITE;
    let mut inactive_alpha = 0.5;
    let mut highlight_alpha = 1.0;

    loop {
        // Rotate camera around the object with A/D
        if is_key_down(KeyCode::A) {
            camera_angle += 0.01;
        } else if is_key_down(KeyCode::D) {
            camera_angle -= 0.01;
        }

        // Move camera up/down with W/S
        if is_key_down(KeyCode::W) {
            camera_y += 0.01;
        } else if is_key_down(KeyCode::S) {
            camera_y -= 0.01;
        }

        clear_background(BLACK);

        let camera = Camera3D {
            position: target_pos
                + vec3(camera_angle.cos() * 5.0, camera_y, camera_angle.sin() * 5.0),
            up: vec3(0., 1., 0.),
            target: target_pos,
            ..Default::default()
        };

        set_camera(&camera);

        draw_grid(20, 1., LIGHTGRAY, DARKGRAY);

        let aspect = camera
            .aspect
            .unwrap_or_else(|| screen_width() / screen_height());
        let projection_matrix = Mat4::perspective_rh_gl(camera.fovy, aspect, 0.01, 1000.0);
        let view_matrix = Mat4::look_at_rh(camera.position, camera.target, camera.up);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Settings")
                .resizable(false)
                .show(egui_ctx, |ui| {
                    egui::ComboBox::from_label("Mode")
                        .selected_text(format!("{gizmo_mode:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut gizmo_mode, GizmoMode::Rotate, "Rotate");
                            ui.selectable_value(&mut gizmo_mode, GizmoMode::Translate, "Translate");
                            ui.selectable_value(&mut gizmo_mode, GizmoMode::Scale, "Scale");
                        });
                    ui.end_row();

                    egui::ComboBox::from_label("Orientation")
                        .selected_text(format!("{gizmo_orientation:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut gizmo_orientation,
                                GizmoOrientation::Global,
                                "Global",
                            );
                            ui.selectable_value(
                                &mut gizmo_orientation,
                                GizmoOrientation::Local,
                                "Local",
                            );
                        });
                    ui.end_row();

                    ui.separator();

                    egui::Slider::new(&mut gizmo_size, 10.0..=500.0)
                        .text("Gizmo size")
                        .ui(ui);
                    egui::Slider::new(&mut stroke_width, 0.1..=10.0)
                        .text("Stroke width")
                        .ui(ui);
                    egui::Slider::new(&mut inactive_alpha, 0.0..=1.0)
                        .text("Inactive alpha")
                        .ui(ui);
                    egui::Slider::new(&mut highlight_alpha, 0.0..=1.0)
                        .text("Highlighted alpha")
                        .ui(ui);

                    ui.horizontal(|ui| {
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut highlight_color,
                            Alpha::Opaque,
                        );
                        egui::Checkbox::new(&mut custom_highlight_color, "Custom highlight color")
                            .ui(ui);
                    });

                    ui.horizontal(|ui| {
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut x_color,
                            Alpha::Opaque,
                        );
                        egui::Label::new("X axis color").wrap(false).ui(ui);
                    });

                    ui.horizontal(|ui| {
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut y_color,
                            Alpha::Opaque,
                        );
                        egui::Label::new("Y axis color").wrap(false).ui(ui);
                    });
                    ui.horizontal(|ui| {
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut z_color,
                            Alpha::Opaque,
                        );
                        egui::Label::new("Z axis color").wrap(false).ui(ui);
                    });
                    ui.horizontal(|ui| {
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut s_color,
                            Alpha::Opaque,
                        );
                        egui::Label::new("Screen axis color").wrap(false).ui(ui);
                    });
                    ui.end_row();

                    egui::Hyperlink::from_label_and_url("(source code)", SOURCE_URL).ui(ui);
                });

            egui::Area::new("Viewport")
                .fixed_pos((0.0, 0.0))
                .show(egui_ctx, |ui| {
                    ui.with_layer_id(LayerId::background(), |ui| {
                        // Snapping is enabled with ctrl key.
                        let snapping = is_key_down(KeyCode::LeftControl);
                        let precise_snap = snapping && is_key_down(KeyCode::LeftShift);

                        // Snap angle to use for rotation when snapping is enabled.
                        // Smaller snap angle is used when shift key is pressed.
                        let snap_angle = if precise_snap {
                            DEFAULT_SNAP_ANGLE / 2.0
                        } else {
                            DEFAULT_SNAP_ANGLE
                        };

                        // Snap distance to use for translation when snapping is enabled.
                        // Smaller snap distance is used when shift key is pressed.
                        let snap_distance = if precise_snap {
                            DEFAULT_SNAP_DISTANCE / 2.0
                        } else {
                            DEFAULT_SNAP_DISTANCE
                        };

                        let visuals = GizmoVisuals {
                            stroke_width,
                            x_color,
                            y_color,
                            z_color,
                            s_color,
                            inactive_alpha,
                            highlight_alpha,
                            highlight_color: if custom_highlight_color {
                                Some(highlight_color)
                            } else {
                                None
                            },
                            gizmo_size,
                        };

                        let gizmo = Gizmo::new("My gizmo")
                            .view_matrix(view_matrix.to_cols_array_2d().into())
                            .projection_matrix(projection_matrix.to_cols_array_2d().into())
                            .model_matrix(model_matrix.to_cols_array_2d().into())
                            .mode(gizmo_mode)
                            .orientation(gizmo_orientation)
                            .snapping(snapping)
                            .snap_angle(snap_angle)
                            .snap_distance(snap_distance)
                            .visuals(visuals);

                        last_gizmo_response = gizmo.interact(ui);

                        if let Some(gizmo_response) = last_gizmo_response {
                            // Response contains status of the active gizmo,
                            // including an updated model matrix.

                            model_matrix =
                                Mat4::from_cols_array(gizmo_response.transform().as_ref());

                            show_gizmo_status(ui, gizmo_response);
                        }

                        instructions_text(ui);
                    });
                });
        });

        unsafe { get_internal_gl().quad_gl }.push_model_matrix(model_matrix);

        draw_cube(
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 1.0, 1.0),
            Some(texture),
            GRAY,
        );

        egui_macroquad::draw();

        next_frame().await
    }
}

fn instructions_text(ui: &Ui) {
    let rect = ui.clip_rect();
    ui.painter().text(
        pos2(rect.right() - 10.0, rect.bottom() - 10.0),
        Align2::RIGHT_BOTTOM,
        "Move camera with (A, D, W, S)\n\
         Toggle snapping with Ctrl & Shift",
        FontId::default(),
        Color32::GRAY,
    );
}

fn show_gizmo_status(ui: &Ui, response: GizmoResult) {
    let length = Vec3::from(response.value).length();

    let text = match response.mode {
        GizmoMode::Rotate => format!("{:.1}°, {:.2} rad", length.to_degrees(), length),

        GizmoMode::Translate | GizmoMode::Scale => format!(
            "dX: {:.2}, dY: {:.2}, dZ: {:.2}",
            response.value[0], response.value[1], response.value[2]
        ),
    };

    let rect = ui.clip_rect();
    ui.painter().text(
        pos2(rect.left() + 10.0, rect.bottom() - 10.0),
        Align2::LEFT_BOTTOM,
        text,
        FontId::default(),
        Color32::WHITE,
    );
}
