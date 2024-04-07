//! A 3D transformation Gizmo for the Bevy game engine.
//!
//! transform-gizmo-bevy provides a feature-rich and configurable 3D transformation
//! gizmo that can be used to manipulate entities' transforms (position, rotation, scale)
//! visually.
//!
//! # Usage
//!
//! Add `TransformGizmoPlugin` to your App.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use transform_gizmo_bevy::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(TransformGizmoPlugin)
//!     .run();
//! ```
//!
//! Add [`GizmoCamera`] component to your Camera entity.
//!
//! Add [`GizmoTarget`] component to any of your entities that you would like to manipulate the [`Transform`] of.
//!
//! # Configuration
//!
//! You can configure the gizmo by modifying the [`GizmoOptions`] resource.
//!
//! You can either set it up with [`App::insert_resource`] when creating your App, or at any point in a system with [`ResMut<GizmoOptions>`].

use bevy::prelude::*;
use bevy::utils::{HashMap, Uuid};
use bevy::window::PrimaryWindow;
use bevy_math::{DQuat, DVec3};
use render::{DrawDataHandles, TransformGizmoRenderPlugin};
use transform_gizmo::config::{
    TransformPivotPoint, DEFAULT_SNAP_ANGLE, DEFAULT_SNAP_DISTANCE, DEFAULT_SNAP_SCALE,
};

pub use transform_gizmo::{
    math::{Pos2, Rect},
    GizmoConfig, *,
};

pub mod prelude;

mod render;

const GIZMO_GROUP_UUID: Uuid = Uuid::from_u128(0x_1c90_3d44_0152_45e1_b1c9_889a_0203_e90c);

/// Adds transform gizmos to the App.
///
/// Gizmos are interactive tools that appear in the scene, allowing users to manipulate
/// entities' transforms (position, rotation, scale) visually.
pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<render::GizmoDrawData>()
            .init_resource::<GizmoOptions>()
            .init_resource::<GizmoStorage>()
            .add_plugins(TransformGizmoRenderPlugin)
            .add_systems(Last, (update_gizmos, draw_gizmos, cleanup_old_data).chain());
    }
}

/// Various options for configuring the transform gizmos.
/// Many of these options are
#[derive(Resource, Copy, Clone, Debug)]
pub struct GizmoOptions {
    /// Modes to use in the gizmos
    pub gizmo_modes: EnumSet<GizmoMode>,
    /// Orientation of the gizmo. This affects the behaviour of transformations.
    pub gizmo_orientation: GizmoOrientation,
    /// Orientation of the gizmo. This affects the behaviour of transformations.
    pub pivot_point: TransformPivotPoint,
    /// Look and feel of the gizmo.
    pub visuals: GizmoVisuals,
    /// Whether snapping is enabled in the gizmo transformations.
    pub snapping: bool,
    /// Angle increment for snapping rotations, in radians.
    pub snap_angle: f32,
    /// Distance increment for snapping translations.
    pub snap_distance: f32,
    /// Scale increment for snapping scalings.
    pub snap_scale: f32,
    /// If `true`, all [`GizmoTarget`]s are transformed
    /// using a single gizmo. If `false`, each target
    /// has its own gizmo.
    pub group_targets: bool,
}

impl Default for GizmoOptions {
    fn default() -> Self {
        Self {
            gizmo_modes: EnumSet::only(GizmoMode::Rotate),
            gizmo_orientation: GizmoOrientation::default(),
            pivot_point: TransformPivotPoint::default(),
            visuals: Default::default(),
            snapping: false,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
            snap_scale: DEFAULT_SNAP_SCALE,
            group_targets: true,
        }
    }
}

/// Marks an entity as a gizmo target.
///
/// When an entity has this component and a [`Transform`],
/// a gizmo is shown, which can be used to manipulate the
/// transform component.
///
/// If target grouping is enabled in [`GizmoOptions`],
/// a single gizmo is used for all targets. Otherwise
/// a separate gizmo is used for each target entity.
#[derive(Component, Copy, Clone, Debug, Default)]
pub struct GizmoTarget {
    /// Whether any part of the gizmo is currently focused.
    pub(crate) is_focused: bool,

    /// Whether the gizmo is currently being interacted with.
    pub(crate) is_active: bool,

    /// This gets replaced with the result of the most recent
    /// gizmo interaction that affected this entity.
    pub(crate) latest_result: Option<GizmoResult>,
}

impl GizmoTarget {
    /// Whether any part of the gizmo is currently focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Whether the gizmo is currently being interacted with.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// This gets replaced with the result of the most recent
    /// gizmo interaction that affected this entity.
    pub fn latest_result(&self) -> Option<GizmoResult> {
        self.latest_result
    }
}

/// Marker used to specify which camera to use for gizmos.
#[derive(Component)]
pub struct GizmoCamera;

#[derive(Resource, Default)]
struct GizmoStorage {
    target_entities: Vec<Entity>,
    entity_gizmo_map: HashMap<Entity, Uuid>,
    gizmos: HashMap<Uuid, Gizmo>,
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

    let Some(viewport) = camera.logical_viewport_rect() else {
        return;
    };

    let viewport = Rect::from_min_max(
        Pos2::new(viewport.min.x, viewport.min.y),
        Pos2::new(viewport.max.x, viewport.max.y),
    );

    let projection_matrix = camera.projection_matrix();

    let view_matrix = camera_transform.compute_matrix().inverse();

    let gizmo_config = GizmoConfig {
        view_matrix: view_matrix.as_dmat4().into(),
        projection_matrix: projection_matrix.as_dmat4().into(),
        viewport,
        modes: gizmo_options.gizmo_modes,
        orientation: gizmo_options.gizmo_orientation,
        pivot_point: gizmo_options.pivot_point,
        visuals: gizmo_options.visuals,
        snapping: gizmo_options.snapping,
        snap_angle: gizmo_options.snap_angle,
        snap_distance: gizmo_options.snap_distance,
        snap_scale: gizmo_options.snap_scale,
        pixels_per_point: scale_factor,
    };

    let gizmo_interaction = GizmoInteraction {
        cursor_pos: (cursor_pos.x, cursor_pos.y),
        drag_started: mouse.just_pressed(MouseButton::Left),
        dragging: mouse.any_pressed([MouseButton::Left]),
    };

    let mut target_entities: Vec<Entity> = vec![];
    let mut target_transforms: Vec<Transform> = vec![];

    for (entity, mut target_transform, mut gizmo_target) in &mut q_targets {
        target_entities.push(entity);
        target_transforms.push(*target_transform);

        if gizmo_options.group_targets {
            gizmo_storage
                .entity_gizmo_map
                .insert(entity, GIZMO_GROUP_UUID);
            continue;
        }

        let mut gizmo_uuid = *gizmo_storage
            .entity_gizmo_map
            .entry(entity)
            .or_insert_with(Uuid::new_v4);

        // Group gizmo was used previously
        if gizmo_uuid == GIZMO_GROUP_UUID {
            gizmo_uuid = Uuid::new_v4();
            gizmo_storage.entity_gizmo_map.insert(entity, gizmo_uuid);
        }

        let gizmo = gizmo_storage.gizmos.entry(gizmo_uuid).or_default();
        gizmo.update_config(gizmo_config);

        let gizmo_result = gizmo.update(
            gizmo_interaction,
            &[transform_gizmo::math::Transform {
                translation: target_transform.translation.as_dvec3().into(),
                rotation: target_transform.rotation.as_dquat().into(),
                scale: target_transform.scale.as_dvec3().into(),
            }],
        );

        let is_focused = gizmo.is_focused();

        gizmo_target.is_active = gizmo_result.is_some();
        gizmo_target.is_focused = is_focused;

        if let Some((_, updated_targets)) = &gizmo_result {
            let Some(result_transform) = updated_targets.first() else {
                bevy::log::warn!("No transform found in GizmoResult!");
                continue;
            };

            target_transform.translation = DVec3::from(result_transform.translation).as_vec3();
            target_transform.rotation = DQuat::from(result_transform.rotation).as_quat();
            target_transform.scale = DVec3::from(result_transform.scale).as_vec3();
        }

        gizmo_target.latest_result = gizmo_result.map(|(result, _)| result);
    }

    if gizmo_options.group_targets {
        let gizmo = gizmo_storage.gizmos.entry(GIZMO_GROUP_UUID).or_default();
        gizmo.update_config(gizmo_config);

        let gizmo_result = gizmo.update(
            gizmo_interaction,
            target_transforms
                .iter()
                .map(|transform| transform_gizmo::math::Transform {
                    translation: transform.translation.as_dvec3().into(),
                    rotation: transform.rotation.as_dquat().into(),
                    scale: transform.scale.as_dvec3().into(),
                })
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let is_focused = gizmo.is_focused();

        for (i, (_, mut target_transform, mut gizmo_target)) in q_targets.iter_mut().enumerate() {
            gizmo_target.is_active = gizmo_result.is_some();
            gizmo_target.is_focused = is_focused;

            if let Some((_, updated_targets)) = &gizmo_result {
                let Some(result_transform) = updated_targets.get(i) else {
                    bevy::log::warn!("No transform {i} found in GizmoResult!");
                    continue;
                };

                target_transform.translation = DVec3::from(result_transform.translation).as_vec3();
                target_transform.rotation = DQuat::from(result_transform.rotation).as_quat();
                target_transform.scale = DVec3::from(result_transform.scale).as_vec3();
            }

            gizmo_target.latest_result = gizmo_result.as_ref().map(|(result, _)| *result);
        }
    }

    gizmo_storage.target_entities = target_entities;
}

fn draw_gizmos(
    gizmo_storage: Res<GizmoStorage>,
    mut draw_data_assets: ResMut<Assets<render::GizmoDrawData>>,
    mut draw_data_handles: ResMut<DrawDataHandles>,
) {
    for (gizmo_uuid, gizmo) in &gizmo_storage.gizmos {
        let draw_data = gizmo.draw();

        let mut bevy_draw_data = render::GizmoDrawData::default();

        let (asset, is_new_asset) = if let Some(handle) = draw_data_handles.handles.get(gizmo_uuid)
        {
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

            draw_data_handles.handles.insert(*gizmo_uuid, asset.clone());
        }
    }
}

fn cleanup_old_data(
    gizmo_options: Res<GizmoOptions>,
    mut gizmo_storage: ResMut<GizmoStorage>,
    mut draw_data_handles: ResMut<DrawDataHandles>,
) {
    let target_entities = std::mem::take(&mut gizmo_storage.target_entities);

    let mut gizmos_to_keep = vec![];

    if gizmo_options.group_targets && !target_entities.is_empty() {
        gizmos_to_keep.push(GIZMO_GROUP_UUID);
    }

    gizmo_storage.entity_gizmo_map.retain(|entity, uuid| {
        if !target_entities.contains(entity) {
            false
        } else {
            gizmos_to_keep.push(*uuid);

            true
        }
    });

    gizmo_storage
        .gizmos
        .retain(|uuid, _| gizmos_to_keep.contains(uuid));

    draw_data_handles
        .handles
        .retain(|uuid, _| gizmos_to_keep.contains(uuid));
}
