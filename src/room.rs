use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::*;
use std::ops::Range;

pub const ROOM_RADIUS: u32 = 3;
pub const ROOM_SIZE: TilemapSize = TilemapSize {
    x: ROOM_RADIUS * 2 + 1,
    y: ROOM_RADIUS * 2 + 1,
};

pub const ROOM_TILE_LAYER: f32 = 0.0;

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
/// All of the information about a given room.
pub enum RoomInfo {
    /// An empty room with nothing interesting
    EmptyRoom,
    /// The entrance room, with nothing interesting
    Entrance,
    /// The exit room with nothing interesting,
    /// until you gather all the stuffs to leave this hell hole
    Exit,
    /// A room that holds enemies to fight
    Combat {
        /// The enemies that are inside the room
        /// Any room visited should have this list emptied
        enemies: Box<[ActorName]>,
        visited: bool,
    },
    /// A room that deals damage upon entry
    Pit {
        /// The range of damage that can be
        /// done by the spike pit
        damage: Range<u32>,
        /// Whether or not the trap has
        /// been triggered already or not.
        triggered: bool,
    },
    /// A room that grants an item upone entry.
    Item {
        /// The item that is inside the room,
        /// zero
        item: (), //Item,
        taken: bool,
    },
}

#[derive(Resource)]
pub struct CurrentRoom(pub RoomInfo);

#[derive(Component)]
pub struct RoomTile;

#[derive(Component)]
pub struct RoomTilemap;

pub fn spawn_room(mut commands: Commands, tile_texture: Res<HexTileImage>, room: Res<CurrentRoom>) {
    info!("spawn room");
    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(ROOM_SIZE);
    let origin = TilePos {
        x: ROOM_SIZE.x / 2,
        y: ROOM_SIZE.y / 2,
    };

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
            tile_storage.checked_set(&tile_pos, id);
        }
    });

    commands.entity(tilemap_entity).insert((
        RoomTilemap,
        TilemapBundle {
            grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
            size: ROOM_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_texture.image.clone()),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
            ..Default::default()
        },
    ));

    use RoomInfo as R;
    match &room.0 {
        R::EmptyRoom => {}
        R::Entrance => {}
        R::Exit => {}
        R::Combat { enemies, visited } => {}
        R::Item { item, taken } => {}
        R::Pit { damage, triggered } => {}
    }
}
