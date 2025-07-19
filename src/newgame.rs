use crate::menu::new_game::NewGameState;
use crate::prelude::*;
use bevy::prelude::*;

//use crate::tiles::spawn_tile_labels;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexNeighbors;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};

pub struct NewGamePlugin;

const ROOM_SIZE: TilemapSize = TilemapSize { x: 21, y: 21 };
const ROOM_TILE_LAYER: f32 = 0.0;
const RADIUS: u32 = 10;

const GENERATING_STATE: NewGameState = NewGameState::GeneratingWorld;

impl Plugin for NewGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileRand(RandomSource::from_os_rng()))
            .add_systems(
                OnEnter(GENERATING_STATE),
                (spawn_room, change_tile)
                    //spawn_tile_labels::<RoomTileMap, RoomTile>
                    .chain(),
            )
            .add_systems(Update, update_neighbors.run_if(in_state(GENERATING_STATE)));
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
    dblue: bool,
    entropy: u32,
}

#[derive(Component)]
pub enum Collapsed {
    Gray,
    Red,
    Yellow,
    Green,
    LBlue,
    DBlue,
}

fn spawn_room(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<TileRand>) {
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
            let id = parent
                .spawn((
                    RoomTile,
                    tile_pos,
                    ValidTiles {
                        gray: true,
                        red: true,
                        yellow: true,
                        green: true,
                        lblue: true,
                        dblue: true,
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
        TilemapBundle {
            grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: ROOM_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
            ..Default::default()
        },
    ));
}

fn change_tile(mut commands: Commands, tilestorage_q: Query<&mut TileStorage, With<RoomTileMap>>) {
    let origin = TilePos { x: 10, y: 10 };

    for tile_storage in &tilestorage_q {
        let tile = tile_storage.checked_get(&origin).unwrap();

        commands.entity(tile).insert(Collapsed::Gray);

        /*
        commands.entity(tile).insert((

            TileBundle {
            position: origin,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(0),
            ..Default::default()
        },));
        */
    }
}

fn update_neighbors(
    changed_tile_q: Query<(&Collapsed, &TilePos), Changed<Collapsed>>,
    tilestorage_q: Single<&TileStorage, With<RoomTileMap>>,
    mut valid_tile_q: Query<&mut ValidTiles>,
) {
    let tile_storage = *tilestorage_q;
    for (collapsed, tile_pos) in changed_tile_q {
        let neighbors =
            HexNeighbors::<TilePos>::get_neighboring_positions_standard(&tile_pos, &ROOM_SIZE);
        for loc in neighbors.iter() {
            if let Some(entity) = tile_storage.checked_get(&loc) {
                let mut valid_tile = valid_tile_q.get_mut(entity).unwrap();

                valid_tile.gray = false;
            }
        }
    }
}
