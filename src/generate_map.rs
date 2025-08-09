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
pub const MAP_RADIUS: u32 = 5;
pub const PILLAR_OFFSET_VERT: u32 = 3;
pub const PILLAR_OFFSET_HORZ_X: u32 = 4;
pub const PILLAR_OFFSET_HORZ_Y: u32 = 1;
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

/// Plugin to setup map generation
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
                    create_origin_and_pillars,
                    build_paths,
                    spawn_tile_labels::<With<MapTilemap>, With<MapTile>>,
                )
                    .chain(),
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
#[derive(Resource, Deref, DerefMut)]
struct GenerationRand(pub RandomSource);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MapTile;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MapTilemap;

#[derive(Component)]
pub struct Created;

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

#[derive(Component, Clone, Copy)]
pub enum Pillars {
    North,
    East,
    South,
    West,
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
                    //ValidTiles::default(),
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

// TODO: change values to enums

/// finds the origin of the Map
fn create_origin_and_pillars(
    mut commands: Commands,
    tilestorage_q: Query<&mut TileStorage, With<MapTilemap>>,
    mut tile_rand: ResMut<GenerationRand>,
    mut tile_text_q: Query<&mut TileTextureIndex>,
) {
    let north_tile_pos: TilePos = TilePos {
        x: tile_rand.0.random_range(MAP_RADIUS-PILLAR_OFFSET_VERT..=MAP_RADIUS),
        y: tile_rand.0.random_range(MAP_RADIUS+PILLAR_OFFSET_VERT..=MAP_RADIUS+MAP_RADIUS),
    };
    let east_tile_pos: TilePos = TilePos {
        x: tile_rand.0.random_range(MAP_RADIUS-PILLAR_OFFSET_HORZ_X..=MAP_RADIUS-PILLAR_OFFSET_VERT),
        y: tile_rand.0.random_range(MAP_RADIUS-PILLAR_OFFSET_HORZ_Y..=MAP_RADIUS+PILLAR_OFFSET_HORZ_Y),
    };
    let south_tile_pos: TilePos = TilePos {
        x: tile_rand.0.random_range(MAP_RADIUS..=MAP_RADIUS+PILLAR_OFFSET_VERT),
        y: tile_rand.0.random_range(MAP_RADIUS-MAP_RADIUS..=MAP_RADIUS-PILLAR_OFFSET_VERT),
    };
    let west_tile_pos: TilePos = TilePos {
        x: tile_rand.0.random_range(MAP_RADIUS+PILLAR_OFFSET_VERT..=MAP_RADIUS+PILLAR_OFFSET_HORZ_X),
        y: tile_rand.0.random_range(MAP_RADIUS-PILLAR_OFFSET_HORZ_Y..=MAP_RADIUS+PILLAR_OFFSET_HORZ_Y),
    };
    for tile_storage in &tilestorage_q {
        let start = tile_storage
            .get(&MAP_ORIGIN)
            .expect("The origin should exist, as we just made it...");
        let north = tile_storage
            .get(&north_tile_pos)
            .expect("The north should exist");
        let east = tile_storage
            .get(&east_tile_pos)
            .expect("The east should exist");
        let south = tile_storage
            .get(&south_tile_pos)
            .expect("The south should exist");
        let west = tile_storage
            .get(&west_tile_pos)
            .expect("The west should exist");

        let collapsed = Collapsed::Red;

        let mut tile_texture = tile_text_q.get_mut(start).unwrap();
        *tile_texture = collapsed.to_texture();

        let mut tile_text_north = tile_text_q.get_mut(north).unwrap();
        *tile_text_north = collapsed.to_texture();

        let mut tile_text_east = tile_text_q.get_mut(east).unwrap();
        *tile_text_east = collapsed.to_texture();

        let mut tile_text_south = tile_text_q.get_mut(south).unwrap();
        *tile_text_south = collapsed.to_texture();

        let mut tile_text_west = tile_text_q.get_mut(west).unwrap();
        *tile_text_west = collapsed.to_texture();

        commands
            .entity(start)
            .insert((
                collapsed,
                RoomInfo::from_type(RoomType::Entrance, 0xDeadBeef),
            ));
        commands
            .entity(north)
            .insert((
                Pillars::North,
                collapsed,
                RoomInfo::from_type(RoomType::Pillar, 0xDeadBeef),
            )); commands
            .entity(east)
            .insert((
                Pillars::East,
                collapsed,
                RoomInfo::from_type(RoomType::Pillar, 0xDeadBeef),
            ));
        commands
            .entity(south)
            .insert((
                Pillars::South,
                collapsed,
                RoomInfo::from_type(RoomType::Pillar, 0xDeadBeef),
            ));
        commands
            .entity(west)
            .insert((
                Pillars::West,
                collapsed,
                RoomInfo::from_type(RoomType::Pillar, 0xDeadBeef),
            ));
    }
}

fn build_paths(
    mut commands: Commands,
    pillars_q: Query<&TilePos, With<Pillars>>,
    tilestorage_q: Query<&mut TileStorage, With<MapTilemap>>,
    mut tile_text_q: Query<&mut TileTextureIndex>,
     mut rng: ResMut<GenerationRand>,
) {
    let mut seen: Vec<TilePos> = Vec::new();
    for tile_storage in tilestorage_q {
        for pillar in pillars_q {
            let mut current_pos: TilePos = TilePos { x: 5, y: 5 };

            while current_pos.x != pillar.x || current_pos.y != pillar.y {
                let neighbors =
                    HexNeighbors::<TilePos>::get_neighboring_positions_standard(&current_pos, &MAP_SIZE);
                
                let mut least: u32 = 20;
                let mut store_x: u32 = current_pos.x;
                let mut store_y: u32 = current_pos.y;

                for neighbor in neighbors.iter() {
                    let x_diff = ((pillar.x as i32) - (neighbor.x as i32)).abs() as u32;
                    let y_diff = ((pillar.y as i32) - (neighbor.y as i32)).abs() as u32;

                    if least > x_diff + y_diff {
                        least = x_diff + y_diff;
                        store_x = neighbor.x;
                        store_y = neighbor.y;
                    }
                }

                current_pos.x = store_x;
                current_pos.y = store_y;

                if current_pos.x == pillar.x && current_pos.y == pillar.y {
                    break;
                }

                let mut check = true;

                for seenIdx in 0..seen.len() {
                    let tile_pos: &TilePos = seen.get(seenIdx).unwrap();
                    if tile_pos.x == current_pos.x && tile_pos.y == current_pos.y {
                        check = false;
                        break;
                    }
                }

                if check {
                    seen.push(current_pos);

                    let selected_tile = tile_storage.get(&current_pos).unwrap();

                    let mut selected_texture = tile_text_q.get_mut(selected_tile).unwrap();
                    *selected_texture = Collapsed::Gray.to_texture();

                    commands
                        .entity(selected_tile)
                        .insert((
                            Collapsed::Gray,
                            RoomInfo::from_type(RoomType::from_rng(&mut *rng), 0xDeadBeef),
                        ));
                }
            }
        }
    }
}
