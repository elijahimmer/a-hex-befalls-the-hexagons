use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::Range;

pub const ROOM_RADIUS: u32 = 3;
// + 1 for center and + 2 for doors
pub const ROOM_SIZE: TilemapSize = TilemapSize {
    x: ROOM_RADIUS * 2 + 3,
    y: ROOM_RADIUS * 2 + 3,
};
// + 1 for margins
pub const ROOM_CENTER: TilePos = TilePos {
    x: ROOM_RADIUS + 1,
    y: ROOM_RADIUS + 1,
};

pub const ROOM_TILE_LAYER: f32 = -1.0;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub cleared: bool,
    pub r_type: RoomType,
    pub rng_seed: u64,
}

impl RoomInfo {
    pub fn from_type(r_type: RoomType, rng_seed: u64) -> Self {
        Self {
            cleared: false,
            r_type,
            rng_seed,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
/// All of the information about a given room.
pub enum RoomType {
    /// An empty room with nothing interesting
    EmptyRoom,
    /// A room that holds enemies to fight
    /// Stores the enemies that are inside the room
    /// When cleared, all of the given actors should be spawned dead.
    /// Otherwise they are alive.
    Combat(Box<[ActorName]>),
    /// A room that deals damage upon entry
    /// Stores the range of damage that can be
    /// done by the spike pit
    /// When cleared, the pit is trigged, otherwise it
    /// will trigger on entrance
    Pit(u32),
    /// A room that grants an item upone entry.
    /// Stores the item that is inside the room,
    /// zero
    ///
    /// When cleared, the item is automatically collected
    /// thus later visits will not grant the item again.
    ///
    /// TODO: Replace the `()` with the `Item` type when
    /// that is created.
    Item(Item),
    /// The entrance room, with nothing interesting
    ///
    /// Also acts as the exit once you have collected all
    /// nessesary parts
    Entrance,
    Pillar,
}

impl RoomType {
    pub fn from_rng(rng: &mut impl Rng) -> RoomType {
        let val = rng.random_range(0..3);

        match val {
            0 => RoomType::EmptyRoom,
            1 => RoomType::Combat(ActorName::get_enemies(rng)),
            2 => RoomType::Pit(rng.random_range(0..21)),
            //3 => RoomType::Item(Item::get_rand_item(rng)),
            _ => unreachable!(),
        }
    }
}

/// Marker to indicate the current room the player
/// is in
#[derive(Component)]
pub struct CurrentRoom;

/// Marker to indicate whether an entity should despawn
/// when the room it was spawned in is exited.
#[derive(Component)]
pub struct InRoom;

/// Marker to indicate the room hex tiles
#[derive(Component)]
pub struct RoomTile;

/// Marker to indicate the room tile map
#[derive(Component)]
pub struct RoomTilemap;

pub fn spawn_room(mut commands: Commands, tile_texture: Res<HexTileImage>) {
    let tilemap_entity = commands.spawn((Visibility::Visible,)).id();

    let mut tile_storage = TileStorage::empty(ROOM_SIZE);
    let origin = ROOM_CENTER;

    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&origin, HEX_COORD_SYSTEM),
        ROOM_RADIUS,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(HEX_COORD_SYSTEM));

    commands.entity(tilemap_entity).with_children(|parent| {
        for tile_pos in tile_positions {
            let id = parent
                .spawn((
                    RoomTile,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(FLOOR_TILE_VARIENTS.start),
                        ..Default::default()
                    },
                ))
                .id();
            tile_storage.set(&tile_pos, id);
        }
    });

    commands.entity(tilemap_entity).insert((
        RoomTilemap,
        Pickable::default(),
        TilemapBundle {
            grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: ROOM_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_texture.image.clone()),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
            visibility: Visibility::Visible,
            ..Default::default()
        },
    ));
}

pub const ENEMY_POSITIONS: [IVec2; 3] = [IVec2::new(1, 1), IVec2::new(-1, 2), IVec2::new(-2, 1)];
pub const ITEM_POSITION: IVec2 = IVec2::new(1, 1);

pub fn spawn_room_entities(
    mut commands: Commands,
    info: Single<&RoomInfo, With<CurrentRoom>>,
    asset_server: Res<AssetServer>,
    tilemap: Single<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TilemapType,
            &TilemapAnchor,
        ),
        With<RoomTilemap>,
    >,
) {
    let (map_size, grid_size, tile_size, map_type, map_anchor) = *tilemap;

    let center_tile_pos = UVec2 {
        x: map_size.x / 2,
        y: map_size.y / 2,
    };

    let RoomInfo {
        cleared, r_type, ..
    } = *info;

    use RoomType as R;
    match &r_type {
        R::EmptyRoom => {}
        R::Entrance => {}
        R::Combat(enemies) => {
            for (name, pos_offset) in enemies.iter().zip(ENEMY_POSITIONS.into_iter()) {
                let actor_pos: TilePos =
                    (center_tile_pos.as_ivec2() + pos_offset).as_uvec2().into();

                let world_pos =
                    actor_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);

                let transform = Transform::from_xyz(world_pos.x, world_pos.y, ACTOR_LAYER);

                commands.spawn((
                    InRoom,
                    ActorBundle::from_name(&asset_server, *name, Team::Enemy, transform, !cleared),
                    Pickable::default(),
                    Visibility::Visible,
                ));
            }
        }
        R::Item(item) => match item {
            Item::HealingPotion => {}

            Item::VisionPotion => {}
        },
        R::Pit(damage) => {}
        R::Pillar => {}
    }
}

/// Should be run after the room
pub fn mark_room_cleared(mut info: Single<&mut RoomInfo, With<CurrentRoom>>) {
    match info.r_type {
        RoomType::Entrance => {}
        _ => info.cleared = true,
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EntranceDirection {
    NorthEast,
    North,
    NorthWest,
    SouthWest,
    South,
    SouthEast,
}

impl EntranceDirection {
    pub const ALL: [EntranceDirection; 6] = [
        EntranceDirection::NorthEast,
        EntranceDirection::North,
        EntranceDirection::NorthWest,
        EntranceDirection::SouthWest,
        EntranceDirection::South,
        EntranceDirection::SouthEast,
    ];

    pub fn axial_offset(&self) -> AxialPos {
        match self {
            EntranceDirection::NorthEast => AxialPos::new(1, 0),
            EntranceDirection::North => AxialPos::new(0, 1),
            EntranceDirection::NorthWest => AxialPos::new(-1, 1),
            EntranceDirection::SouthWest => AxialPos::new(-1, 0),
            EntranceDirection::South => AxialPos::new(0, -1),
            EntranceDirection::SouthEast => AxialPos::new(1, -1),
        }
    }

    pub fn door_offset(
        &self,
        origin: &TilePos,
        distance: u32,
        coord_sys: HexCoordSystem,
    ) -> TilePos {
        let pos = AxialPos::from_tile_pos_given_coord_system(origin, coord_sys);
        let offset = match self {
            EntranceDirection::NorthEast => AxialPos::new(1, 1),
            EntranceDirection::North => AxialPos::new(-1, 2),
            EntranceDirection::NorthWest => AxialPos::new(-2, 1),
            EntranceDirection::SouthWest => AxialPos::new(-1, -1),
            EntranceDirection::South => AxialPos::new(1, -2),
            EntranceDirection::SouthEast => AxialPos::new(2, -1),
        };
        let pos = AxialPos {
            q: pos.q + offset.q * distance.div_ceil(2) as i32,
            r: pos.r + offset.r * distance.div_ceil(2) as i32,
        };
        pos.as_tile_pos_given_coord_system(coord_sys)
    }
}
