// TODO: Make this run not limited to the game loop

use crate::menu::new_game::GenerationProgress;
use crate::menu::new_game::NewGameState;
use crate::prelude::*;
use crate::tile::spawn_tile_labels;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexNeighbors;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;

pub struct GenerateMapPlugin;

pub const WORLD_MAP_ORIGIN: Vec3 = Vec3::new(1000.0, 0.0, MAP_TILE_LAYER);
pub const MAP_RADIUS: u32 = 3;
pub const MAP_SIZE: TilemapSize = TilemapSize {
    x: MAP_RADIUS * 2 + 1,
    y: MAP_RADIUS * 2 + 1,
};
pub const MAP_ORIGIN: TilePos = TilePos {
    x: MAP_RADIUS,
    y: MAP_RADIUS,
};
pub const MAP_TILE_LAYER: f32 = 0.0;

const GENERATION_SCHEDULE_FREQUENCY: f64 = 10000.0;
const GENERATING_STATE: NewGameState = NewGameState::GeneratingWorld;

impl Plugin for GenerateMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GENERATING_STATE),
            set_fixed_update_time(GENERATION_SCHEDULE_FREQUENCY),
        )
        .add_systems(
            OnEnter(GENERATING_STATE),
            (
                setup,
                spawn_map,
                (
                    create_origin,
                    spawn_tile_labels::<With<MapTilemap>, With<MapTile>>,
                ),
            )
                .chain(),
        )
        .add_systems(
            OnExit(GENERATING_STATE),
            (
                restore_fixed_update_time,
                despawn_tile_labels::<With<MapTilemap>>,
                remove_component::<Collapsed>,
            ),
        )
        .add_systems(
            FixedUpdate,
            (update_neighbors, collapse_tile)
                .chain()
                .run_if(in_state(GENERATING_STATE)),
        );
    }
}

/// Settings set by the UI before world generation to
/// give generation parameters.
#[derive(Resource)]
pub struct GenerationSettings {
    pub seed: u64,
}

/// Seedable Rand Resource
#[derive(Resource)]
struct GenerationRand(pub RandomSource);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MapTile;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MapTilemap;

/// List of tiles a tile in the tilemap can be next to.
#[derive(Component, Clone)]
pub struct ValidTiles {
    gray: bool,
    red: bool,
    yellow: bool,
    green: bool,
    lblue: bool,
    dblue: bool,
}



impl ValidTiles {
    /// Method to get entropy of a tile
    pub fn entropy(&self) -> u8 {
        self.gray as u8
            + self.red as u8
            + self.yellow as u8
            + self.green as u8
            + self.lblue as u8
            + self.dblue as u8
    }

    /// Method to collapse a tile to a single state
    pub fn collapse(&self, rng: &mut RandomSource) -> Option<Collapsed> {
        let possibilities = [
            (self.gray, Collapsed::Gray),
            (self.red, Collapsed::Red),
            (self.yellow, Collapsed::Yellow),
            (self.green, Collapsed::Green),
            (self.lblue, Collapsed::LBlue),
            (self.dblue, Collapsed::DBlue),
        ]
        .into_iter()
        .filter_map(|(enable, c)| enable.then_some(c))
        .collect::<Vec<_>>();

        if possibilities.len() == 0 {
            return None;
        }

        Some(possibilities[rng.random_range(0..possibilities.len())])
    }
}


impl Default for ValidTiles {
    fn default() -> Self {
        Self {
            gray: true,
            red: true,
            yellow: true,
            green: true,
            lblue: true,
            dblue: true,
        }
    }
}

/// Enum to represent all possible collapsed components


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
    /// Method to get enum value of tile state
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


/// Setup for Generation settings so generation is seedable
fn setup(mut commands: Commands, settings: Res<GenerationSettings>) {
    let rng = RandomSource::seed_from_u64(settings.seed);
    commands.insert_resource(GenerationRand(rng));
}

/// Spawns tilemap
fn spawn_map(mut commands: Commands, tile_texture: Res<HexTileImage>) {
    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(MAP_SIZE);
    let origin = TilePos {
        x: MAP_SIZE.x / 2,
        y: MAP_SIZE.y / 2,
    };

    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&origin, HEX_COORD_SYSTEM),
        MAP_RADIUS,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(HEX_COORD_SYSTEM));

    commands.entity(tilemap_entity).with_children(|parent| {
        for tile_pos in tile_positions {
            let id = parent
                .spawn((
                    MapTile,
                    ValidTiles::default(),
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(OUTLINE_TILE),
                        ..Default::default()
                    },
                ))
                .id();
            tile_storage.checked_set(&tile_pos, id);
        }
    });

    commands.entity(tilemap_entity).insert((
        MapTilemap,
        TilemapBundle {
            grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: MAP_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_texture.image.clone()),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_translation(WORLD_MAP_ORIGIN),
            ..Default::default()
        },
    ));
}

/// finds the origin of the Map
fn create_origin(mut commands: Commands, tilestorage_q: Query<&mut TileStorage, With<MapTilemap>>) {
    for tile_storage in &tilestorage_q {
        let tile = tile_storage
            .get(&MAP_ORIGIN)
            .expect("The origin should exist, as we just made it...");

        commands.entity(tile).insert((
            Collapsed::Red,
            RoomInfo::from_type(RoomType::Entrance, 0xDeadBeef),
        ));
    }
}

/// updates the entropy of neighbor tiles that are changed
fn update_neighbors(
    changed_tile_q: Query<(&Collapsed, &TilePos), Changed<Collapsed>>,
    tilestorage_q: Single<&TileStorage, With<MapTilemap>>,
    mut valid_tile_q: Query<&mut ValidTiles>,
) {
    let tile_storage = *tilestorage_q;
    for (collapsed, tile_pos) in changed_tile_q {
        let neighbors =
            HexNeighbors::<TilePos>::get_neighboring_positions_standard(&tile_pos, &MAP_SIZE);
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
                }
            }
        }
    }
}

/// collapses a tile
fn collapse_tile(
    mut commands: Commands,
    tile_storage: Single<&TileStorage, With<MapTilemap>>,
    valid_tile_q: Query<&ValidTiles>,
    mut tile_text_q: Query<&mut TileTextureIndex>,
    mut tile_rand: ResMut<GenerationRand>,
    mut generation_progress: ResMut<GenerationProgress>,
) {
    let mut entity_vec: Vec<Entity> = Vec::new();
    let mut lowest = u8::MAX;

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

    if entity_vec.len() == 0 {
        generation_progress.world_done = true;
        return;
    }

    let selected_entity = entity_vec[tile_rand.0.random_range(0..entity_vec.len())];
    let mut tile_texture = tile_text_q.get_mut(selected_entity).unwrap();
    let Some(collapsed) = valid_tile_q
        .get(selected_entity)
        .unwrap()
        .collapse(&mut tile_rand.0)
    else {
        commands.entity(selected_entity).remove::<ValidTiles>();
        return;
    };

    *tile_texture = collapsed.to_texture();
    commands
        .entity(selected_entity)
        .insert(collapsed)
        .remove::<ValidTiles>();
}
