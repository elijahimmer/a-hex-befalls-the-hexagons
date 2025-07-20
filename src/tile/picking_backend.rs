use bevy::app::prelude::*;
use bevy::ecs::prelude::*;
use bevy::math::{FloatExt, prelude::*};
use bevy::picking::backend::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::prelude::*;

/// `bevy_ecs_tilemap` backend for `bevy_mod_picking`
///
/// The plugins provided by those two crates must be added separately.
pub struct TilemapBackend;
impl Plugin for TilemapBackend {
    fn build(&self, app: &mut App) {
        app.init_resource::<TilePickingSettings>()
            .register_type::<TilePickingSettings>()
            .add_systems(PreUpdate, tile_picking.in_set(PickSet::Backend));
    }
}

/// An optional component that marks cameras that should be used in the [`TilePickingPlugin`].
///
/// Only needed if [`TilePickingSettings::require_markers`] is set to `true`, and ignored
/// otherwise.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Debug, Default, Component, Clone)]
pub struct TilePickingCamera;

/// Runtime settings for the [`TilePickingPlugin`].
#[derive(Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct TilePickingSettings {
    /// When set to `true` tile picking will only consider cameras marked with
    /// [`TilePickingCamera`].
    ///
    /// This setting is provided to give you fine-grained control over which cameras and entities
    /// should be used by the tile picking backend at runtime.
    pub require_markers: bool,
}

impl Default for TilePickingSettings {
    fn default() -> Self {
        Self {
            require_markers: false,
        }
    }
}

fn tile_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(
        Entity,
        &Camera,
        &GlobalTransform,
        &Projection,
        Has<TilePickingCamera>,
    )>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    settings: Res<TilePickingSettings>,
    tile_map_query: Query<(
        &GlobalTransform,
        &Pickable,
        &ViewVisibility,
        (
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TilemapType,
            &TilemapAnchor,
        ),
    )>,
    mut output: EventWriter<PointerHits>,
) {
    let mut sorted_tile_maps: Vec<_> = tile_map_query
        .iter()
        .filter_map(|(transform, pickable, vis, params)| {
            if !transform.affine().is_nan() && vis.get() {
                Some((transform, pickable, params))
            } else {
                None
            }
        })
        .collect();

    // radsort is a stable radix sort that performed better than `slice::sort_by_key`
    radsort::sort_by_key(&mut sorted_tile_maps, |(transform, _, _)| {
        -transform.translation().z
    });

    let primary_window = primary_window.single().ok();

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform, Projection::Orthographic(cam_ortho), _)) =
            cameras
                .iter()
                .filter(|(_, camera, _, _, cam_can_pick)| {
                    let marker_requirement = !settings.require_markers || *cam_can_pick;
                    camera.is_active && marker_requirement
                })
                .find(|(_, camera, _, _, _)| {
                    camera
                        .target
                        .normalize(primary_window)
                        .is_some_and(|x| x == location.target)
                })
        else {
            continue;
        };

        let viewport_pos = camera
            .logical_viewport_rect()
            .map(|v| v.min)
            .unwrap_or_default();
        let pos_in_viewport = location.position - viewport_pos;

        let Ok(cursor_ray_world) = camera.viewport_to_world(cam_transform, pos_in_viewport) else {
            continue;
        };
        let cursor_ray_len = cam_ortho.far - cam_ortho.near;
        let cursor_ray_end = cursor_ray_world.origin + cursor_ray_world.direction * cursor_ray_len;

        let picks: Vec<(Entity, HitData)> = sorted_tile_maps
            .iter()
            .copied()
            .filter_map(
                |(
                    map_transform,
                    pickable,
                    (storage, map_size, grid_size, tile_size, map_type, anchor),
                )| {
                    if blocked {
                        return None;
                    }

                    // Transform cursor line segment to tile coordinate system
                    let world_to_tile = map_transform.affine().inverse();
                    let cursor_start_tile = world_to_tile.transform_point3(cursor_ray_world.origin);
                    let cursor_end_tile = world_to_tile.transform_point3(cursor_ray_end);

                    // Find where the cursor segment intersects the plane Z=0 (which is the tile's
                    // plane in tile-local space). It may not intersect if, for example, we're
                    // viewing the tile side-on
                    if cursor_start_tile.z == cursor_end_tile.z {
                        // Cursor ray is parallel to the tile and misses it
                        return None;
                    }
                    let lerp_factor =
                        f32::inverse_lerp(cursor_start_tile.z, cursor_end_tile.z, 0.0);
                    if !(0.0..=1.0).contains(&lerp_factor) {
                        // Lerp factor is out of range, meaning that while an infinite line cast by
                        // the cursor would intersect the tile, the tile is not between the
                        // camera's near and far planes
                        return None;
                    }

                    // Otherwise we can interpolate the xy of the start and end positions by the
                    // lerp factor to get the cursor position in tile space!
                    let cursor_pos_tile = cursor_start_tile.lerp(cursor_end_tile, lerp_factor).xy();

                    let Some(tile_pos) = TilePos::from_world_pos(
                        &cursor_pos_tile,
                        map_size,
                        grid_size,
                        tile_size,
                        map_type,
                        anchor,
                    ) else {
                        return None;
                    };

                    let Some(tile_hovered) = storage.get(&tile_pos) else {
                        return None;
                    };

                    blocked = pickable.should_block_lower;

                    let hit_pos_world = map_transform.transform_point(cursor_pos_tile.extend(0.0));
                    // Transform point from world to camera space to get the Z distance
                    let hit_pos_cam = cam_transform
                        .affine()
                        .inverse()
                        .transform_point3(hit_pos_world);
                    // HitData requires a depth as calculated from the camera's near clipping plane
                    let depth = -cam_ortho.near - hit_pos_cam.z;

                    Some((
                        tile_hovered,
                        HitData::new(
                            cam_entity,
                            depth,
                            Some(hit_pos_world),
                            Some(*map_transform.back()),
                        ),
                    ))
                },
            )
            .collect();

        let order = camera.order as f32;

        output.write(PointerHits::new(*pointer, picks, order));
    }
}
