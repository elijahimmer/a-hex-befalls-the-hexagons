use crate::menu::new_game::NewGameState;
use crate::prelude::*;
use crate::tiles::spawn_tile_labels;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexNeighbors;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;

pub struct GenerateMapPlugin;

const ROOM_SIZE: TilemapSize = TilemapSize { x: 21, y: 21 };
const ROOM_TILE_LAYER: f32 = 0.0;
const RADIUS: u32 = 10;

const GENERATING_STATE: NewGameState = NewGameState::GeneratingWorld;

impl Plugin for GenerateMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileRand(RandomSource::from_os_rng()))
            .add_systems(
                OnEnter(GENERATING_STATE),
                (
                    spawn_room,
                    create_origin,
                    spawn_tile_labels::<RoomTileMap, RoomTile>,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (update_neighbors, collapse_tile)
                    .chain()
                    .run_if(in_state(GENERATING_STATE)),
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
    dblue: bool,
}



impl ValidTiles {
    pub fn entropy(&self) -> u8 {
        self.gray as u8
            + self.red as u8
            + self.yellow as u8
            + self.green as u8
            + self.lblue as u8
            + self.dblue as u8
    }

    pub fn collapse(&self, mut rng: &mut RandomSource) -> Option<Collapsed> {
        let mut options = Vec::with_capacity(6);
        if self.gray {
            options.push(Collapsed::Gray);
        }
        if self.red {
            options.push(Collapsed::Red);
        }
        if self.yellow {
            options.push(Collapsed::Yellow);
        }
        if self.green {
            options.push(Collapsed::Green);
        }
        if self.lblue {
            options.push(Collapsed::LBlue);
        }
        if self.dblue {
            options.push(Collapsed::DBlue);
        }

        if options.len() == 0 {
            return None;
        }

        Some(options[rng.random_range(0..options.len())]) 
    }
}

#[derive(Component, Clone, Copy)]
pub enum Collapsed {
    Gray,
    Red,
    Yellow,
    Green,
    LBlue,
    DBlue,
}

impl Collapsed {
    pub fn to_texture(&self) -> TileTextureIndex {
        TileTextureIndex(match self {
            Collapsed::Gray => 0,
            Collapsed::Red => 1,
            Collapsed::Yellow => 2,
            Collapsed::Green => 3,
            Collapsed::LBlue => 4,
            Collapsed::DBlue => 5,
        })
    }
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
                    ValidTiles {
                        gray: false,
                        red: true,
                        yellow: true,
                        green: true,
                        lblue: true,
                        dblue: true,
                    },
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(0),
                        ..Default::default()
                    },
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

fn create_origin(
    mut commands: Commands,
    tilestorage_q: Query<&mut TileStorage, With<RoomTileMap>>,
) {
    let origin = TilePos { x: 10, y: 10 };

    for tile_storage in &tilestorage_q {
        let tile = tile_storage.checked_get(&origin).unwrap();

        commands.entity(tile).insert(Collapsed::Red);
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
                let Ok(mut valid_tile) = valid_tile_q.get_mut(entity) else {
                    continue;
                };

                match collapsed {
                    Collapsed::Gray => {
                        valid_tile.gray = false;
                    }

                    Collapsed::Red => {
                        valid_tile.red = false;
                    }

                    Collapsed::Yellow => {
                        valid_tile.yellow = false;
                    }
                    Collapsed::Green => {
                        valid_tile.green = false;
                    }

                    Collapsed::LBlue => {
                        valid_tile.lblue = false;
                    }

                    Collapsed::DBlue => {
                        valid_tile.dblue = false;
                    }
                    _ => println!("ERROR"),
                }
            }
        }
    }
}

fn collapse_tile(
    mut commands: Commands,
    tile_storage_q: Query<&TileStorage, With<RoomTileMap>>,
    valid_tile_q: Query<&ValidTiles>,
    mut tile_text_q: Query<&mut TileTextureIndex>,
    mut tile_rand: ResMut<TileRand>,
) {
    let mut entity_vec: Vec<Entity> = Vec::new();
    for tile_storage in tile_storage_q {
        let mut lowest = 7;

        for tile in tile_storage.iter().filter_map(|t| *t) {
            let Ok(valid_tile) = valid_tile_q.get(tile) else {
                continue;
            };

            let entropy = valid_tile.entropy();

            match lowest.cmp(&entropy) {
                Ordering::Less => {}
                Ordering::Equal => {
                    entity_vec.push(tile);
                }
                Ordering::Greater => {
                    entity_vec.clear();
                    entity_vec.push(tile);
                    lowest = entropy;
                }
            }
        }
    }

    if entity_vec.len() == 0 {
        return;
    }

    let selected_entity = entity_vec[tile_rand.0.random_range(0..entity_vec.len())];
    let mut tile_texture = tile_text_q.get_mut(selected_entity).unwrap();
    let Some(collapsed) = valid_tile_q
        .get(selected_entity)
        .unwrap()
        .collapse(&mut tile_rand.0) else {
            commands.entity(selected_entity).remove::<ValidTiles>();
            return;
        };

    *tile_texture = collapsed.to_texture();
    commands
        .entity(selected_entity)
        .insert(collapsed)
        .remove::<ValidTiles>();
}
