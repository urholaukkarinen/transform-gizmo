use bevy::prelude::*;
use camera::PanOrbitCameraPlugin;
use gui::GuiPlugin;
use picking::PickingPlugin;
use scene::ScenePlugin;

use transform_gizmo_bevy::prelude::*;

mod camera;
mod grid;
mod gui;
mod picking;
mod scene;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(20, 20, 20)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "transform-gizmo-demo".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GuiPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(ScenePlugin)
        .add_plugins(TransformGizmoPlugin)
        .add_plugins(PickingPlugin)
        .insert_resource(GizmoOptions {
            gizmo_modes: enum_set!(GizmoMode::Rotate | GizmoMode::Translate | GizmoMode::Scale),
            gizmo_orientation: GizmoOrientation::Global,
            ..default()
        })
        .run();
}
