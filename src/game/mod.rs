pub mod controls;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use controls::*;

const SQUARE_LAYER: f32 = 1.0;
const SQUARE_SIZE: f32 = 20.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(controls::GameControlsPlugin);
        app.add_systems(OnEnter(GameState::Game), (fixup_room, spawn_square));
    }
}

fn fixup_room(mut commands: Commands, tilemap: Single<(Entity, &TileStorage), With<RoomTilemap>>) {
    let (entity, tile_storage) = *tilemap;

    commands.entity(entity).insert(Pickable::default());

    for tile in tile_storage.iter().filter_map(|t| *t) {
        commands
            .entity(tile)
            .insert((Pickable::default(), Visibility::Visible))
            .observe(tile_hover_indicator)
            .observe(tile_hover_indicator_remove);
    }
}

//        &Pickable,
//        ,
//        (
//            &TileStorage,
//            &TilemapSize,
//            &TilemapGridSize,
//            &TilemapTileSize,
//            &TilemapType,
//            &TilemapAnchor,
//        ),

// TODO: Use the generated map, don't make it here
//fn spawn_room(mut commands: Commands, asset_server: Res<AssetServer>) {
//    let mut rng = RandomSource::from_os_rng();
//
//    let texture_handle: Handle<Image> = asset_server.load(TILE_ASSET_LOAD_PATH);
//
//    let tilemap_entity = commands
//        .spawn(())
//        .id();
//    let mut tile_storage = TileStorage::empty(ROOM_SIZE);
//
//    let origin = TilePos { x: 5, y: 5 }; // Made changes here
//
//    let tile_positions = generate_hexagon(
//        AxialPos::from_tile_pos_given_coord_system(&origin, HEX_COORD_SYSTEM),
//        RADIUS,
//    )
//    .into_iter()
//    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(HEX_COORD_SYSTEM));
//
//    commands.entity(tilemap_entity).with_children(|parent| {
//        for tile_pos in tile_positions {
//            let id = parent
//                .spawn((
//                    RoomTile,
//                    TileBundle {
//                        position: tile_pos,
//                        tilemap_id: TilemapId(tilemap_entity),
//                        texture_index: TileTextureIndex(rng.random_range(FLOOR_TILE_VARIENTS)),
//                        ..Default::default()
//                    },
//                    Pickable::default(),
//                    Visibility::Inherited,
//                ))
//                .observe(tile_hover_indicator)
//                .observe(tile_hover_indicator_remove)
//                .id();
//
//            tile_storage.checked_set(&tile_pos, id);
//        }
//    });
//
//    commands.entity(tilemap_entity).insert((
//        RoomTilemap,
//        TilemapBundle {
//            grid_size: TILE_SIZE.into(),
//            map_type: TilemapType::Hexagon(HexCoordSystem::Row),
//            size: ROOM_SIZE,
//            storage: tile_storage,
//            texture: TilemapTexture::Single(texture_handle),
//            tile_size: TILE_SIZE,
//            anchor: TilemapAnchor::Center,
//            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
//            visibility: Visibility::Visible,
//            ..Default::default()
//        },
//        Pickable::default(),
//    ));
//}

fn spawn_square(
    mut commands: Commands,
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

    let lower = map_size.x / 2 - 1;
    let range = lower..(lower + 3);

    for x in range {
        let center_tile_pos = TilePos {
            x,
            y: map_size.y / 2,
        };

        let world_pos =
            center_tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);

        commands
            .spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::splat(SQUARE_SIZE)),
                    ..Default::default()
                },
                Transform::from_xyz(world_pos.x, world_pos.y, SQUARE_LAYER),
                Pickable::default(),
                Player {
                    player_speed: 300.0,
                },
            ))
            .observe(select_player::<Pointer<Click>>);
    }
}

//fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
//    move |ev, mut sprites| {
//        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
//            return;
//        };
//        sprite.color = color;
//    }
//}
