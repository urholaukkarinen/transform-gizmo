use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{CompressedImageFormats, ImageFormat, ImageSampler, ImageType};
use bevy::window::PresentMode;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use egui::color_picker::Alpha;
use egui::{pos2, Align2, Color32, FontId, LayerId, Ui, Widget};

use egui_gizmo::{
    Gizmo, GizmoMode, GizmoOrientation, GizmoResult, GizmoVisuals, DEFAULT_SNAP_ANGLE,
    DEFAULT_SNAP_DISTANCE,
};

use crate::camera::{setup_camera, update_camera};

mod camera;

const SOURCE_URL: &str = "https://github.com/urholaukkarinen/egui-gizmo/blob/main/demo/src/main.rs";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "egui-gizmo demo".into(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(EguiPlugin)
        .add_plugins(InfiniteGridPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, update)
        .add_systems(Update, update_camera)
        .run();
}

#[derive(Resource)]
struct GizmoOptions {
    gizmo_mode: GizmoMode,
    gizmo_orientation: GizmoOrientation,
    last_result: Option<GizmoResult>,
    custom_highlight_color: bool,
    visuals: GizmoVisuals,
}

#[derive(Component)]
struct Target;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GizmoOptions {
        gizmo_mode: GizmoMode::Rotate,
        gizmo_orientation: GizmoOrientation::Global,
        last_result: None,
        custom_highlight_color: false,
        visuals: GizmoVisuals {
            x_color: Color32::from_rgb(255, 0, 148),
            y_color: Color32::from_rgb(148, 255, 0),
            z_color: Color32::from_rgb(0, 148, 255),
            s_color: Color32::WHITE,
            ..default()
        },
    });

    let texture_handle = asset_server.add(
        Image::from_buffer(
            include_bytes!("../crate.png"),
            ImageType::Format(ImageFormat::Png),
            CompressedImageFormats::all(),
            true,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        )
        .unwrap(),
    );

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.,
    });
    let cube_handle = meshes.add(Cuboid {
        half_size: Vec3::ONE,
    });
    let cube_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        // unlit: true,
        ..default()
    });

    commands.spawn((
        Target,
        PbrBundle {
            mesh: cube_handle,
            material: cube_material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
    ));

    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            shadow_color: None,
            ..default()
        },
        ..default()
    });
}

fn update(
    mut contexts: EguiContexts,
    camera_q: Query<(&Camera, &Transform), Without<Target>>,
    mut target_q: Query<&mut Transform, With<Target>>,
    mut gizmo_options: ResMut<GizmoOptions>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (projection_matrix, view_matrix) = {
        let (camera, transform) = camera_q.single();
        (
            camera.projection_matrix(),
            transform.compute_matrix().inverse(),
        )
    };

    egui::Window::new("Settings")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            egui::ComboBox::from_label("Mode")
                .selected_text(format!("{:?}", gizmo_options.gizmo_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut gizmo_options.gizmo_mode, GizmoMode::Rotate, "Rotate");
                    ui.selectable_value(
                        &mut gizmo_options.gizmo_mode,
                        GizmoMode::Translate,
                        "Translate",
                    );
                    ui.selectable_value(&mut gizmo_options.gizmo_mode, GizmoMode::Scale, "Scale");
                });
            ui.end_row();

            egui::ComboBox::from_label("Orientation")
                .selected_text(format!("{:?}", gizmo_options.gizmo_orientation))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut gizmo_options.gizmo_orientation,
                        GizmoOrientation::Global,
                        "Global",
                    );
                    ui.selectable_value(
                        &mut gizmo_options.gizmo_orientation,
                        GizmoOrientation::Local,
                        "Local",
                    );
                });
            ui.end_row();

            ui.separator();

            egui::Slider::new(&mut gizmo_options.visuals.gizmo_size, 10.0..=500.0)
                .text("Gizmo size")
                .ui(ui);
            egui::Slider::new(&mut gizmo_options.visuals.stroke_width, 0.1..=10.0)
                .text("Stroke width")
                .ui(ui);
            egui::Slider::new(&mut gizmo_options.visuals.inactive_alpha, 0.0..=1.0)
                .text("Inactive alpha")
                .ui(ui);
            egui::Slider::new(&mut gizmo_options.visuals.highlight_alpha, 0.0..=1.0)
                .text("Highlighted alpha")
                .ui(ui);

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    gizmo_options
                        .visuals
                        .highlight_color
                        .get_or_insert(Color32::GOLD),
                    Alpha::Opaque,
                );
                egui::Checkbox::new(
                    &mut gizmo_options.custom_highlight_color,
                    "Custom highlight color",
                )
                .ui(ui);
            });

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut gizmo_options.visuals.x_color,
                    Alpha::Opaque,
                );
                egui::Label::new("X axis color").wrap(false).ui(ui);
            });

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut gizmo_options.visuals.y_color,
                    Alpha::Opaque,
                );
                egui::Label::new("Y axis color").wrap(false).ui(ui);
            });
            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut gizmo_options.visuals.z_color,
                    Alpha::Opaque,
                );
                egui::Label::new("Z axis color").wrap(false).ui(ui);
            });
            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut gizmo_options.visuals.s_color,
                    Alpha::Opaque,
                );
                egui::Label::new("Screen axis color").wrap(false).ui(ui);
            });
            ui.end_row();

            egui::Hyperlink::from_label_and_url("(source code)", SOURCE_URL).ui(ui);
        });

    egui::Area::new("Viewport".into())
        .fixed_pos((0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.with_layer_id(LayerId::background(), |ui| {
                // Snapping is enabled with ctrl key.
                let snapping = keys.pressed(KeyCode::ControlLeft);
                let precise_snap = snapping && keys.pressed(KeyCode::ShiftLeft);

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
                    highlight_color: if gizmo_options.custom_highlight_color {
                        gizmo_options.visuals.highlight_color
                    } else {
                        None
                    },
                    ..gizmo_options.visuals
                };

                let model_matrix = target_q.single_mut().compute_matrix();

                let gizmo = Gizmo::new("My gizmo")
                    .view_matrix(view_matrix.to_cols_array_2d().into())
                    .projection_matrix(projection_matrix.to_cols_array_2d().into())
                    .model_matrix(model_matrix.to_cols_array_2d().into())
                    .mode(gizmo_options.gizmo_mode)
                    .orientation(gizmo_options.gizmo_orientation)
                    .snapping(snapping)
                    .snap_angle(snap_angle)
                    .snap_distance(snap_distance)
                    .visuals(visuals);

                gizmo_options.last_result = gizmo.interact(ui);

                if let Some(gizmo_response) = gizmo_options.last_result {
                    let mut target_transform = target_q.single_mut();

                    target_transform.translation = gizmo_response.translation.into();
                    target_transform.rotation = gizmo_response.rotation.into();
                    target_transform.scale = gizmo_response.scale.into();

                    show_gizmo_status(ui, gizmo_response);
                }

                instructions_text(ui);
            });
        });
}

fn instructions_text(ui: &Ui) {
    let rect = ui.clip_rect();
    ui.painter().text(
        pos2(rect.left() + 10.0, rect.bottom() - 10.0),
        Align2::LEFT_BOTTOM,
        "Move and rotate the camera using the middle and right mouse buttons\n\
         Toggle gizmo snapping with left ctrl & shift",
        FontId::default(),
        Color32::GRAY,
    );
}

fn show_gizmo_status(ui: &Ui, response: GizmoResult) {
    let text = if let Some(value) = response.value {
        match response.mode {
            GizmoMode::Rotate => {
                let length = Vec3::from(value).length();
                format!("{:.1}°, {:.2} rad", length.to_degrees(), length)
            }

            GizmoMode::Translate | GizmoMode::Scale => format!(
                "dX: {:.2}, dY: {:.2}, dZ: {:.2}",
                value[0], value[1], value[2]
            ),
        }
    } else {
        String::new()
    };

    let rect = ui.clip_rect();
    ui.painter().text(
        pos2(rect.right() - 10.0, rect.bottom() - 10.0),
        Align2::RIGHT_BOTTOM,
        text,
        FontId::default(),
        Color32::WHITE,
    );
}
