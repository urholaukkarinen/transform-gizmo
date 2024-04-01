use bevy::prelude::*;
use bevy::render::{Extract, RenderApp};
use bevy::utils::hashbrown::HashSet;
use bevy::utils::{HashMap, Uuid};
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
            .add_systems(Last, (update_gizmos, draw_gizmos, cleanup_old_data).chain())
            .add_plugins(TransformGizmoRenderPlugin);

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(ExtractSchedule, extract_gizmo_data);
    }
}

#[derive(Resource, Copy, Clone, Debug)]
pub struct GizmoOptions {
    pub gizmo_modes: EnumSet<GizmoMode>,
    pub gizmo_orientation: GizmoOrientation,
    pub visuals: GizmoVisuals,
    pub snapping: bool,
    pub snap_angle: f32,
    pub snap_distance: f32,
}

impl Default for GizmoOptions {
    fn default() -> Self {
        Self {
            gizmo_modes: EnumSet::only(GizmoMode::Rotate),
            gizmo_orientation: GizmoOrientation::Global,
            visuals: Default::default(),
            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct GizmoTarget {
    /// Whether any part of the gizmo is currently focused.
    pub is_focused: bool,

    /// Whether the gizmo is currently being interacted with.
    pub is_active: bool,
}

impl Default for GizmoTarget {
    fn default() -> Self {
        Self {
            is_focused: false,
            is_active: false,
        }
    }
}

#[derive(Component)]
pub struct GizmoCamera;

#[derive(Resource, Default)]
struct GizmoStorage {
    target_entities: Vec<Entity>,
    entity_gizmo_map: HashMap<Entity, Uuid>,
    gizmos: HashMap<Uuid, Gizmo>,
}

#[derive(Resource, Default)]
struct DrawDataHandles {
    handles: HashMap<Entity, Handle<render::GizmoDrawData>>,
}

fn extract_gizmo_data(mut commands: Commands, handles: Extract<Res<DrawDataHandles>>) {
    let handle_weak_refs = handles
        .handles
        .values()
        .map(|handle| handle.clone_weak())
        .collect::<HashSet<_>>();

    for handle in handle_weak_refs {
        commands.spawn((handle,));
    }
}

#[allow(clippy::too_many_arguments)]
fn update_gizmos(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_gizmo_camera: Query<(&Camera, &Transform), With<GizmoCamera>>,
    mut q_targets: Query<(Entity, &mut Transform, &mut GizmoTarget), Without<GizmoCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gizmo_options: Res<GizmoOptions>,
    mut gizmo_storage: ResMut<GizmoStorage>,
    mut last_cursor_pos: Local<Vec2>,
) {
    let Ok(window) = q_window.get_single() else {
        // No primary window found.
        return;
    };

    let cursor_pos = window.cursor_position().unwrap_or_else(|| *last_cursor_pos);
    *last_cursor_pos = cursor_pos;

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

    let gizmo_config = GizmoConfig {
        view_matrix: view_matrix.into(),
        projection_matrix: projection_matrix.as_dmat4().into(),
        viewport,
        modes: gizmo_options.gizmo_modes,
        orientation: gizmo_options.gizmo_orientation,
        visuals: gizmo_options.visuals,
        snapping: gizmo_options.snapping,
        snap_angle: gizmo_options.snap_angle,
        snap_distance: gizmo_options.snap_distance,
        snap_scale: gizmo_options.snap_distance,
        pixels_per_point: scale_factor,
    };

    let mut target_entities: Vec<Entity> = vec![];
    let mut target_transforms: Vec<Transform> = vec![];

    for (entity, mut target_transform, mut gizmo_target) in q_targets.iter_mut() {
        target_entities.push(entity);
        target_transforms.push(*target_transform);

        let gizmo_uuid = *gizmo_storage
            .entity_gizmo_map
            .entry(entity)
            .or_insert_with(|| Uuid::new_v4());

        let gizmo = gizmo_storage.gizmos.entry(gizmo_uuid).or_default();

        gizmo.update_config(gizmo_config);

        let gizmo_result = gizmo.update(
            GizmoInteraction {
                cursor_pos: (cursor_pos.x, cursor_pos.y),
                drag_started: mouse.just_pressed(MouseButton::Left),
                dragging: mouse.any_pressed([MouseButton::Left]),
            },
            [target_transform.compute_matrix().as_dmat4().into()].into_iter(), /*
                                                                               target_transforms
                                                                                   .iter()
                                                                                   .map(|transform| transform.compute_matrix().as_dmat4().into()),
                                                                                   */
        );

        let is_focused = gizmo.is_any_focused();

        gizmo_target.is_active = gizmo_result.is_some();
        gizmo_target.is_focused = is_focused;

        if let Some(result) = &gizmo_result {
            let Some(result_transform) = result.targets.get(0) else {
                bevy::log::warn!("No matching transform found in GizmoResult!");
                continue;
            };

            *target_transform =
                Transform::from_matrix(bevy::math::DMat4::from(*result_transform).as_mat4());
        }
    }

    gizmo_storage.target_entities = target_entities;
}

#[allow(clippy::too_many_arguments)]
fn draw_gizmos(
    gizmo_storage: Res<GizmoStorage>,
    mut draw_data_assets: ResMut<Assets<render::GizmoDrawData>>,
    mut draw_data_handles: ResMut<DrawDataHandles>,
) {
    for (entity, gizmo_uuid) in gizmo_storage.entity_gizmo_map.iter() {
        let Some(gizmo) = gizmo_storage.gizmos.get(gizmo_uuid) else {
            continue;
        };

        let draw_data = gizmo.draw();

        let mut bevy_draw_data = render::GizmoDrawData::default();

        let (asset, is_new_asset) = if let Some(handle) = draw_data_handles.handles.get(entity) {
            (draw_data_assets.get_mut(handle).unwrap(), false)
        } else {
            (&mut bevy_draw_data, true)
        };

        let viewport = &gizmo.config().viewport;

        asset.0.vertices.clear();
        asset
            .0
            .vertices
            .extend(draw_data.vertices.into_iter().map(|vert| {
                [
                    ((vert[0] - viewport.left()) / viewport.width()) * 2.0 - 1.0,
                    ((vert[1] - viewport.top()) / viewport.height()) * 2.0 - 1.0,
                ]
            }));
        asset.0.colors = draw_data.colors;
        asset.0.indices = draw_data.indices;

        if is_new_asset {
            let asset = draw_data_assets.add(bevy_draw_data);

            draw_data_handles.handles.insert(*entity, asset.clone());
        }
    }
}

fn cleanup_old_data(
    mut gizmo_storage: ResMut<GizmoStorage>,
    mut draw_data_handles: ResMut<DrawDataHandles>,
) {
    let target_entities = std::mem::take(&mut gizmo_storage.target_entities);

    let mut gizmos_to_remove = vec![];

    gizmo_storage.entity_gizmo_map.retain(|entity, uuid| {
        if !target_entities.contains(entity) {
            gizmos_to_remove.push(*uuid);

            false
        } else {
            true
        }
    });

    for uuid in gizmos_to_remove {
        gizmo_storage.gizmos.remove(&uuid);
    }

    draw_data_handles
        .handles
        .retain(|entity, _| target_entities.contains(entity));
}
