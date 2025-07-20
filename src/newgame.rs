use std::fmt::Debug;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};

pub struct NewGamePlugin;

const ROOM_SIZE: TilemapSize = TilemapSize { x: 11, y: 11 }; // Made changes here
const ROOM_TILE_LAYER: f32 = 0.0;
const RADIUS: u32 = 5; //Made changes here

const SQUARE_LAYER: f32 = 1.0;
const SQUARE_SIZE: f32 = 20.0;
const HOVER_INDICATOR_LAYER: f32 = ROOM_TILE_LAYER + 0.1;

impl Plugin for NewGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileRand(RandomSource::from_os_rng()))
            .add_systems(Startup, spawn_hover_indicator)
            .add_systems(OnEnter(GameState::Game), (spawn_room, spawn_square).chain())
            .add_systems(Update, move_to_target.run_if(in_state(GameState::Game)));
    }
}
#[derive(Resource)]
struct TileRand(pub RandomSource);

/// The entity that displays the hover indicator
#[derive(Resource)]
struct HoverIndicator(pub Entity);

/// Stored in axial coordinate
#[derive(Component)]
struct MoveToTile(TilePos);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTile;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RoomTilemap;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CenterSquare;

#[derive(Component, Reflect)]
struct Player {
    player_speed: f32,
}

#[derive(Component, Reflect)]
struct IsSelected;

fn spawn_hover_indicator(mut commands: Commands, tile_image: Res<HexTileImage>) {
    let id = commands
        .spawn((
            Sprite {
                image: tile_image.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    index: OUTLINE_TILE as usize,
                    layout: tile_image.layout.clone(),
                }),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, HOVER_INDICATOR_LAYER),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .id();

    commands.insert_resource(HoverIndicator(id));
}

fn spawn_room(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<TileRand>) {
    let texture_handle: Handle<Image> = asset_server.load(TILE_ASSET_LOAD_PATH);

    let tilemap_entity = commands
        .spawn((Visibility::Inherited, Transform::IDENTITY))
        .id();
    let mut tile_storage = TileStorage::empty(ROOM_SIZE);

    let origin = TilePos { x: 5, y: 5 }; // Made changes here

    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&origin, HEX_COORD_SYSTEM),
        RADIUS,
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
                        texture_index: TileTextureIndex(rng.0.random_range(FLOOR_TILE_VARIENTS)),
                        ..Default::default()
                    },
                    Pickable::default(),
                    Visibility::Inherited,
                ))
                .observe(tile_hover_indicator)
                .observe(tile_hover_indicator_remove)
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
            texture: TilemapTexture::Single(texture_handle),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., ROOM_TILE_LAYER),
            visibility: Visibility::Visible,
            ..Default::default()
        },
        Pickable::default(),
    ));
}

fn spawn_square(mut commands: Commands) {
    for x in 4..=6 {
        let center_tile_pos = TilePos { x, y: 5 };

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
            .observe(select_player::<Pointer<Click>>);
    }
}

fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
            return;
        };
        sprite.color = color;
    }
}

fn select_player<E: Debug + Clone>(
    ev: Trigger<E>,
    mut commands: Commands,
    query_player: Query<Option<&IsSelected>, With<Player>>,
) {
    if let Ok(is_selected) = query_player.get(ev.target()) {
        match is_selected {
            Some(_) => {
                commands.entity(ev.target()).remove::<IsSelected>();
            }
            None => {
                commands.entity(ev.target()).insert(IsSelected);
            }
        }
    }
}

fn move_to_target(
    mut commands: Commands,
    mut query_player: Query<(Entity, &mut Transform, &Player, &MoveToTile)>,
    time: Res<Time>,
) {
    for (entity, mut transform, player, MoveToTile(target_tile)) in query_player.iter_mut() {
        let target_tile = TilePos::center_in_world(
            target_tile,
            &ROOM_SIZE,
            &TilemapGridSize {
                x: TILE_SIZE.x,
                y: TILE_SIZE.y,
            },
            &TILE_SIZE,
            &TilemapType::Hexagon(HexCoordSystem::Row),
            &TilemapAnchor::Center,
        );

        let direction = target_tile - transform.translation.xy();
        let distance = direction.length();

        let move_player = direction.normalize_or_zero()
            * (player.player_speed * time.delta_secs()).clamp(0.0, distance);

        transform.translation += move_player.extend(0.0);

        if transform.translation.xy() == target_tile {
            commands.get_entity(entity).unwrap().remove::<MoveToTile>();
        }
    }
}

fn tile_hover_indicator(
    mut trigger: Trigger<Pointer<Over>>,
    hover_indicator: Res<HoverIndicator>,
    mut hover_indicator_q: Query<(&mut Transform, &mut Visibility)>,
    tile_q: Query<(&ChildOf, &TilePos)>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
) {
    trigger.propagate(false);

    let Ok((parent, tile_pos)) = tile_q.get(trigger.target()) else {
        // this should be observing a tile, and that tile should have a position.
        return;
    };

    let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.get(parent.parent())
    else {
        return;
    };

    let Ok((mut transform, mut visibility)) = hover_indicator_q.get_mut(hover_indicator.0) else {
        return;
    };

    transform.translation = tile_pos
        .center_in_world(map_size, grid_size, tile_size, map_type, anchor)
        .extend(transform.translation.z);

    *visibility = Visibility::Visible;
}

fn tile_hover_indicator_remove(
    mut trigger: Trigger<Pointer<Out>>,
    hover_indicator: Res<HoverIndicator>,
    mut hover_indicator_q: Query<&mut Visibility>,
) {
    trigger.propagate(false);

    let Ok(mut visibility) = hover_indicator_q.get_mut(hover_indicator.0) else {
        return;
    };

    *visibility = Visibility::Hidden;
}
