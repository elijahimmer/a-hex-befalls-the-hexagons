use crate::prelude::*;
use bevy::prelude::*;
use std::{thread, time};

use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexNeighbors;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};
use crate::tiles::spawn_tile_labels;

pub struct NewGamePlugin;

const ROOM_SIZE: TilemapSize = TilemapSize { x: 21, y: 21 };
const ROOM_TILE_LAYER: f32 = 0.0;
const RADIUS: u32 = 10;

impl Plugin for NewGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileRand(RandomSource::from_os_rng()))
            .add_systems(OnEnter(GameState::Game),
                (spawn_room,
                change_tile)
                //spawn_tile_labels::<RoomTileMap, RoomTile>
            .chain()
            );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTile;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTileMap;

#[derive(Resource)]
struct TileRand(pub RandomSource);

#[derive(Component)]
pub struct ValidTiles {
    gray: bool,
    red: bool,
    yellow: bool,
    green: bool,
    lblue: bool,
    rblue: bool,
    entropy: u32,
}

fn spawn_room(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    mut rng: ResMut<TileRand>
) {
    let texture_handle: Handle<Image> = asset_server.load(TILE_ASSET_LOAD_PATH);

    let tilemap_entity = commands.spawn_empty().id();
    
    let mut tile_storage = TileStorage::empty(ROOM_SIZE);
    let origin = TilePos { x: 10, y: 10 };

    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&origin, HEX_COORD_SYSTEM),
        RADIUS,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(HEX_COORD_SYSTEM))
    .collect::<Vec<TilePos>>();

    commands.entity(tilemap_entity).with_children(|parent| {
        for tile_pos in tile_positions {

            let x_pos = tile_pos.x;
            let y_pos = tile_pos.y;

            let id = parent.spawn((
                RoomTile,
                ValidTiles {
                    gray: true,
                    red: true,
                    yellow: true,
                    green: true,
                    lblue: true,
                    rblue: true,
                    entropy: 6,
                },

/*
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                },
*/ 
            ))
            .id();
            tile_storage.checked_set(&tile_pos, id);
        }
    });

    commands.entity(tilemap_entity).insert((
        RoomTileMap,
        TilemapBundle { grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: ROOM_SIZE,
            storage: tile_storage, texture: TilemapTexture::Single(texture_handle),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
            ..Default::default()
        },
    ));
}

fn change_tile(
    mut commands: Commands,
    tilestorage_q: Query<(Entity, &mut TileStorage),  With<RoomTileMap>>,
) {

    let origin = TilePos { x: 10, y: 10 };

    for (tilemap_entity, mut tile_storage) in &tilestorage_q {
        let tile = tile_storage.checked_get(&origin).unwrap();

        commands.entity(tile).insert((
            RoomTile,
            TileBundle {
                position: origin,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(0),
                ..Default::default()
            }
        ));
    }

    update_neighbors(
        commands,
        tilestorage_q,
        HexNeighbors::<bevy_ecs_tilemap::tiles::TilePos>::
            get_neighboring_positions_standard(&origin, &ROOM_SIZE)
    );
}

fn update_neighbors(
    mut commands: Commands,
    tilestorage_q: Query<(Entity, &mut TileStorage), With<RoomTileMap>>,
    neighbors: HexNeighbors::<bevy_ecs_tilemap::tiles::TilePos>,
) {
    for (entity, mut tile_storage) in tilestorage_q {
        for loc in neighbors.iter() {
            let tile = tile_storage.checked_get(&loc);
//            println!("{}", tile.);
        }
    }
}


/*

fn print_neighbors_pos(
    tile_pos: &TilePos,
) {
    let neighbors = HexNeighbors::<bevy_ecs_tilemap::tiles::TilePos>::get_neighboring_positions_standard(tile_pos, &ROOM_SIZE);

    for tile in neighbors.iter() {
        println!("tile x: {}, tile y: {}", tile.x, tile.y); 
    }
}

fn valid_tiles(tile_pos: &TilePos) {
    let neighbors = HexNeighbors::get_neighboring_positions_standard(tile_pos, &ROOM_SIZE);
}


fn valid_index() {
    let valid: Vec<u32> = Vec::new();

    for i in 0..6 {
        
    }
}

fn update_random_tile(
) {
    
}

*/

/*
fn spawn_room( 
    center: TilePos,
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    mut rng: ResMut<TileRand>   
) {
    let texture_handle: Handle<Image> = asset_server.load(TILE_ASSET_LOAD_PATH);
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(ROOM_SIZE);

    for ring_radius in 0..RADIUS {
        
    }
}
*/
