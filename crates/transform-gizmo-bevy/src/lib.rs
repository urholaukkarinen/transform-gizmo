use bevy::prelude::*;
use bevy::render::{Extract, RenderApp};
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use render::TransformGizmoRenderPlugin;
use transform_gizmo::config::{DEFAULT_SNAP_ANGLE, DEFAULT_SNAP_DISTANCE};

pub use transform_gizmo::{GizmoConfig, *};

pub mod prelude;

mod render;

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<render::GizmoDrawData>()
            .init_resource::<DrawDataHandles>()
            .init_resource::<GizmoOptions>()
            .init_resource::<GizmoStorage>()
            .add_systems(Last, update_gizmos)
            .add_plugins(TransformGizmoRenderPlugin);

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(ExtractSchedule, extract_gizmo_data);
    }
}

#[derive(Resource, Debug)]
pub struct GizmoOptions {
    pub gizmo_mode: GizmoMode,
    pub gizmo_orientation: GizmoOrientation,
    pub visuals: GizmoVisuals,
    pub snapping: bool,
    pub snap_angle: f32,
    pub snap_distance: f32,
}

impl Default for GizmoOptions {
    fn default() -> Self {
        Self {
            gizmo_mode: GizmoMode::Rotate,
            gizmo_orientation: GizmoOrientation::Global,
            visuals: Default::default(),
            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
        }
    }
}

#[derive(Component, Debug)]
pub struct GizmoTarget {
    /// Whether the gizmo target is enabled.
    /// When `false`, the gizmo is not drawn and cannot be interacted with.
    pub is_enabled: bool,

    /// Whether any part of the gizmo is currently focused.
    pub is_focused: bool,

    /// Whether the gizmo is currently being interacted with.
    pub is_active: bool,

    /// Result of the latest interaction with this target.
    pub latest_result: Option<GizmoResult>,
}

impl Default for GizmoTarget {
    fn default() -> Self {
        Self {
            is_enabled: true,
            is_focused: false,
            is_active: false,
            latest_result: None,
        }
    }
}

#[derive(Component)]
pub struct GizmoCamera;

#[derive(Resource, Default)]
struct DrawDataHandles {
    handles: HashMap<Entity, Handle<render::GizmoDrawData>>,
}

#[derive(Resource, Default)]
pub struct GizmoStorage {
    pub gizmos: HashMap<Entity, Gizmo>,
    pub results: HashMap<Entity, GizmoResult>,
}

fn extract_gizmo_data(mut commands: Commands, handles: Extract<Res<DrawDataHandles>>) {
    for (_, handle) in &handles.handles {
        commands.spawn((handle.clone_weak(),));
    }
}

#[allow(clippy::too_many_arguments)]
fn update_gizmos(
    q_gizmo_camera: Query<(&Camera, &Transform), With<GizmoCamera>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_targets: Query<(Entity, &mut Transform, &mut GizmoTarget), Without<GizmoCamera>>,

    mouse: Res<ButtonInput<MouseButton>>,
    gizmo_options: Res<GizmoOptions>,

    mut gizmo_storage: ResMut<GizmoStorage>,
    mut draw_data_assets: ResMut<Assets<render::GizmoDrawData>>,
    mut draw_data_handles: ResMut<DrawDataHandles>,
) {
    let Ok(window) = q_window.get_single() else {
        // No primary window found.
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        // Cannot update gizmos, because the cursor is not inside the window.
        return;
    };

    let scale_factor = window.scale_factor();

    let Ok((camera, camera_transform)) = q_gizmo_camera.get_single() else {
        bevy::log::warn!("Only one camera with a GizmoCamera component is supported.");
        return;
    };

    let Some(viewport) = camera.physical_viewport_rect() else {
        return;
    };

    let viewport = transform_gizmo::math::Rect::from_min_max(
        transform_gizmo::math::Pos2::new(viewport.min.x as _, viewport.min.y as _),
        transform_gizmo::math::Pos2::new(viewport.max.x as _, viewport.max.y as _),
    );

    let projection_matrix = camera.projection_matrix();

    let view_matrix = bevy::math::DMat4::from_scale_rotation_translation(
        camera_transform.scale.as_dvec3(),
        camera_transform.rotation.as_dquat(),
        camera_transform.translation.as_dvec3(),
    )
    .inverse();

    let mut targets_this_frame = vec![];

    for (entity, mut target_transform, mut gizmo_target) in &mut q_targets {
        if !gizmo_target.is_enabled {
            continue;
        }

        targets_this_frame.push(entity);

        let model_matrix = bevy::math::DMat4::from_scale_rotation_translation(
            target_transform.scale.as_dvec3(),
            target_transform.rotation.as_dquat(),
            target_transform.translation.as_dvec3(),
        );

        let gizmo_config = GizmoConfig {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.as_dmat4().into(),
            model_matrix: model_matrix.into(),
            viewport,
            modes: EnumSet::only(gizmo_options.gizmo_mode),
            orientation: gizmo_options.gizmo_orientation,
            visuals: gizmo_options.visuals,
            snapping: gizmo_options.snapping,
            snap_angle: gizmo_options.snap_angle,
            snap_distance: gizmo_options.snap_distance,
            snap_scale: gizmo_options.snap_distance,
            pixels_per_point: scale_factor,
        };

        let gizmo = gizmo_storage.gizmos.entry(entity).or_default();

        gizmo.update_config(gizmo_config);

        let gizmo_result = gizmo.update(GizmoInteraction {
            cursor_pos: (cursor_pos.x, cursor_pos.y),
            drag_started: mouse.just_pressed(MouseButton::Left),
            dragging: mouse.any_pressed([MouseButton::Left]),
        });

        let draw_data = gizmo.draw();

        gizmo_target.is_active = gizmo_result.is_some();
        gizmo_target.is_focused = gizmo.is_any_focused();
        gizmo_target.latest_result = gizmo_result;

        if let Some(result) = gizmo_result {
            target_transform.translation = bevy::math::DVec3::from(result.translation).as_vec3();
            target_transform.rotation = bevy::math::DQuat::from(result.rotation).as_quat();
            target_transform.scale = bevy::math::DVec3::from(result.scale).as_vec3();

            gizmo_storage.results.entry(entity).or_insert(result);
        }

        let mut bevy_draw_data = render::GizmoDrawData::default();

        let (asset, is_new) = if let Some(handle) = draw_data_handles.handles.get(&entity) {
            (draw_data_assets.get_mut(handle).unwrap(), false)
        } else {
            (&mut bevy_draw_data, true)
        };

        asset.0.vertices.clear();
        asset
            .0
            .vertices
            .extend(draw_data.vertices.into_iter().map(|vert| {
                [
                    (vert[0] / viewport.width()) * 2.0 - 1.0,
                    (vert[1] / viewport.height()) * 2.0 - 1.0,
                ]
            }));
        asset.0.colors = draw_data.colors;
        asset.0.indices = draw_data.indices;

        if is_new {
            draw_data_handles
                .handles
                .insert(entity, draw_data_assets.add(bevy_draw_data));
        }
    }

    draw_data_handles
        .handles
        .retain(|entity, _| targets_this_frame.contains(entity));
    gizmo_storage
        .gizmos
        .retain(|entity, _| targets_this_frame.contains(entity));
    gizmo_storage
        .results
        .retain(|entity, _| targets_this_frame.contains(entity));
}
