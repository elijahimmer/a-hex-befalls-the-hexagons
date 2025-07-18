use std::fmt::Debug;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};
//use crate::tiles::spawn_tile_labels;

pub struct NewGamePlugin;

const ROOM_SIZE: TilemapSize = TilemapSize { x: 11, y: 11 }; // Made changes here
const ROOM_TILE_LAYER: f32 = 0.0;
const RADIUS: u32 = 5; //Made changes here

const SQUARE_LAYER: f32 = 1.0;
const SQUARE_SIZE: f32 = 20.0;

impl Plugin for NewGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileRand(RandomSource::from_os_rng()))
            .add_systems(OnEnter(GameState::Game), (spawn_room, spawn_square).chain())
            .insert_resource(MyWorldCoords::default())
            .add_systems(Update, (move_to_target, get_tile_pos));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTile;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTileMap;

//////////////////////////////
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CenterSquare;

#[derive(Component)]
struct Player {
    player_speed: f32,
}
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

#[derive(Component)]
struct IsSelected;

///////////////////////

#[derive(Resource)]
struct TileRand(pub RandomSource);

fn spawn_room(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<TileRand>) {
    let texture_handle: Handle<Image> = asset_server.load(TILE_ASSET_LOAD_PATH);

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(ROOM_SIZE);

    let origin = TilePos { x: 5, y: 5 }; // Made changes here

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
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(rng.0.random_range(FLOOR_TILE_VARIENTS)),
                        ..Default::default()
                    },
                    //Pickable::default(),
                    Pickable::IGNORE,
                ))
                //.observe(print_position::<Pointer<Click>>())
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
        Pickable::IGNORE,
    ));
}

fn spawn_square(mut commands: Commands) {
    let center_tile_pos = TilePos { x: 5, y: 5 };

    let world_pos = center_tile_pos.center_in_world(
        &ROOM_SIZE,
        &TilemapGridSize {
            x: TILE_SIZE.x,
            y: TILE_SIZE.y,
        },
        &TILE_SIZE,
        &TilemapType::Hexagon(HexCoordSystem::Row),
        &TilemapAnchor::Center,
    );

    commands
        .spawn((
            CenterSquare,
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
        .observe(recolor_on::<Pointer<Over>>(Color::srgb(0.0, 0.8, 0.2)))
        .observe(recolor_on::<Pointer<Out>>(Color::srgb(1.0, 1.0, 0.0)))
        .observe(select_player::<Pointer<Click>>());
}

fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
            return;
        };
        sprite.color = color;
    }
}

fn select_player<E: Debug + Clone + Reflect>()
-> impl Fn(Trigger<E>, Commands, Query<Option<&IsSelected>, With<Player>>) {
    move |ev, mut commands, query_player| {
        if let Ok(is_selected) = query_player.get(ev.target()) {
            match is_selected {
                Some(_) => {
                    commands.entity(ev.target()).remove::<IsSelected>();
                    println!("unselected");
                }
                None => {
                    commands.entity(ev.target()).insert(IsSelected);
                    println!("selected")
                }
            }
        }
    }
}

fn move_to_target(
    mycoords: Res<MyWorldCoords>,
    mut query_player: Query<(&mut Transform, &Player), With<IsSelected>>,
    time: Res<Time>,
) {
    for (mut transform, player) in query_player.iter_mut() {
        let direction = mycoords.0 - transform.translation.xy();
        let distance = direction.length();

        let move_player = direction.normalize_or_zero()
            * player.player_speed.clamp(0.0, distance)
            * time.delta_secs();
        transform.translation += move_player.extend(0.0);
    }
}

fn get_tile_pos() {}
