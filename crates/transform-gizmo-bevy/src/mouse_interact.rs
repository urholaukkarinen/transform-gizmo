use bevy_app::{App, Plugin, Update};
use bevy_ecs::{event::EventWriter, system::Res};
use bevy_input::{mouse::MouseButton, ButtonInput};

use crate::{GizmoDragStarted, GizmoDragging};

pub struct MouseGizmoInteractionPlugin;
impl Plugin for MouseGizmoInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_interact_gizmo);
    }
}

fn mouse_interact_gizmo(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag_started: EventWriter<GizmoDragStarted>,
    mut dragging: EventWriter<GizmoDragging>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        drag_started.send_default();
    }

    if mouse.pressed(MouseButton::Left) {
        dragging.send_default();
    }
}
