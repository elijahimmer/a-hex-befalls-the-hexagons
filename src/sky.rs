//! The infinite sky implementation
use crate::prelude::*;
//use crate::tiles::spawn_tile_labels;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};

const SKY_MAP_SIZE: TilemapSize = TilemapSize { x: 80, y: 50 };
const SKY_TILE_SIZE_LOOP_THRESHOLD: Vec2 = Vec2 {
    x: TILE_SIZE.x,
    y: TILE_SIZE.y * 1.5,
};
const SKY_TILE_LAYER: f32 = -2.0;
const AXIAL_TRANSLATION_MATRIX: Mat2 =
    Mat2::from_cols_array(&[SQRT_3_2, 1.0 / 3.0, 0.0, 2.0 / 3.0]);

/// The plugin to
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SkyRand(RandomSource::from_os_rng()))
            .add_systems(Startup, spawn_sky)
            .add_systems(Update, sky_movement);
    }
}

/// A marker to mark the Sky Tiles in the Sky TileMap
#[derive(Component)]
pub struct SkyTile;

#[derive(Resource)]
pub struct SkyTileMap(Entity);

/// A marker to mark the Sky TileMap
#[derive(Component)]
pub struct SkyTileMapMarker;

#[derive(Resource)]
struct SkyRand(pub RandomSource);

#[derive(Resource)]
pub struct SkySettings {
    /// The speed of movement in tiles per second, in axial coordinates.
    pub speed: Vec2,
}

/// Spawns the sky fitting the screen (to an extent).
fn spawn_sky(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<SkyRand>) {
    let texture_handle: Handle<Image> = asset_server.load(TILE_ASSET_LOAD_PATH);

    let tilemap_entity = commands.spawn_empty().id();
    commands.insert_resource(SkyTileMap(tilemap_entity));
    let mut tile_storage = TileStorage::empty(SKY_MAP_SIZE);

    commands.entity(tilemap_entity).with_children(|parent| {
        for x in 0..SKY_MAP_SIZE.x {
            for y in 0..SKY_MAP_SIZE.y {
                let tile_pos = TilePos { x, y };
                let id = parent
                    .spawn((
                        SkyTile,
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                            texture_index: TileTextureIndex(rng.0.random_range(SKY_TILE_VARIENTS)),
                            ..Default::default()
                        },
                    ))
                    .id();
                tile_storage.set(&tile_pos, id);
            }
        }
    });

    commands.entity(tilemap_entity).insert((
        SkyTileMapMarker,
        TilemapBundle {
            grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: SKY_MAP_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., SKY_TILE_LAYER),
            ..Default::default()
        },
    ));

    commands.insert_resource(SkySettings {
        speed: Vec2::new(-5.0, -2.0),
    });
}

/// Moves the sky with an illusion that it is indefinite.
///
/// This system
///
fn sky_movement(
    time: Res<Time>,
    sky_movement: ResMut<SkySettings>,
    mut rng: ResMut<SkyRand>,
    tilemap_id: Res<SkyTileMap>,
    mut tilemap: Query<(&TileStorage, &TilemapSize, &mut Transform), With<SkyTileMapMarker>>,
    mut tile_query: Query<&mut TileTextureIndex, With<SkyTile>>,
) {
    let (tile_storage, map_size, mut transform) = tilemap
        .get_mut(tilemap_id.0)
        .expect("The sky should exist.");

    let map_size: IVec2 = IVec2::new(map_size.x as i32, map_size.y as i32);

    let new_translation = AXIAL_TRANSLATION_MATRIX * sky_movement.speed * time.delta_secs()
        + transform.translation.xy();

    let tile_diff = (new_translation / SKY_TILE_SIZE_LOOP_THRESHOLD)
        .trunc()
        .as_ivec2();

    // only translate by the sky by the amount that was less than a whole tile.
    let new_translation = new_translation - tile_diff.as_vec2() * SKY_TILE_SIZE_LOOP_THRESHOLD;

    transform.translation = new_translation.extend(transform.translation.z);

    if tile_diff == IVec2::ZERO {
        return;
    }

    let flip_x = tile_diff.x > 0;
    let flip_y = tile_diff.y > 0;

    for y in 0..map_size.y {
        let y = flip_y.then_some(map_size.y - y - 1).unwrap_or(y);
        for x in 0..map_size.x {
            let x = flip_x.then_some(map_size.x - x - 1).unwrap_or(x);

            let old_pos = IVec2 { x, y };

            // for the hexagons to align with where you started, they have
            // to move 1.5 hexes up or 1 hex to the right.
            // This does the 1.5 hexes up adjustment to turn the
            // hex distance into square distance used by the position.
            let adjusted_diff =
                (Mat2::from_cols_array(&[1., 0., -1., 2.]) * tile_diff.as_vec2()).as_ivec2();

            let replace_pos = old_pos + adjusted_diff;
            let new_pos = old_pos - adjusted_diff;

            let Some(curr_tile_entity) = tile_storage.get(&old_pos.as_uvec2().into()) else {
                warn!("Failed to find sky tile entity at position ({x}, {y})");
                continue;
            };

            if replace_pos.cmpge(IVec2::ZERO).all() && replace_pos.cmplt(map_size).all() {
                // move the texture along the `tile_diff` vector

                let Some(new_tile_entity) = tile_storage.get(&replace_pos.as_uvec2().into()) else {
                    warn!("Failed to find new tile at pos {replace_pos}");
                    continue;
                };

                let curr_tile_texture = match tile_query.get(curr_tile_entity).and_then(|t| Ok(*t))
                {
                    Ok(curr_tile_texture) => curr_tile_texture,
                    Err(err) => {
                        warn!("Failed to find base sky tile at {old_pos} with {err}");
                        continue;
                    }
                };

                match tile_query.get_mut(new_tile_entity) {
                    Ok(mut new_tile_texture) => *new_tile_texture = curr_tile_texture,
                    Err(err) => {
                        warn!("Failed to find to be replaced sky tile at {replace_pos} with {err}");
                        continue;
                    }
                }
            }

            if new_pos.cmplt(IVec2::ZERO).any() || new_pos.cmpge(map_size).any() {
                match tile_query.get_mut(curr_tile_entity) {
                    Ok(mut curr_tile_texture) => {
                        let tile_idx = rng.0.random_range(SKY_TILE_VARIENTS);
                        *curr_tile_texture = TileTextureIndex(tile_idx);
                    }
                    Err(err) => warn!("Failed to get current tile at {new_pos} with {err}"),
                };
            }
        }
    }
}
