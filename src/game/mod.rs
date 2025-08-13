mod combat;

pub use combat::*;

use crate::prelude::*;
use crate::room::{
    CurrentRoom, EntranceDirection, InRoom, ROOM_CENTER, ROOM_RADIUS, mark_room_cleared,
    spawn_room, spawn_room_entities,
};
#[cfg(feature = "sqlite")]
use crate::saving::save_game;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexNeighbors;
use bevy_ecs_tilemap::prelude::*;
use rand::{Rng, SeedableRng};
use std::collections::VecDeque;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameState>();

        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<GameState>);

        app.add_systems(
            OnEnter(AppState::Game),
            (init_room_rng, spawn_room, place_player_actors).chain(),
        )
        .add_systems(Update, set_room_rng.run_if(in_state(AppState::Game)))
        .add_systems(
            OnEnter(GameState::EnterRoom),
            (
                (despawn_filtered::<With<InRoom>>, spawn_room_entities).chain(),
                change_state(GameState::TriggerEvent),
            ),
        )
        .add_systems(
            OnEnter(GameState::TriggerEvent),
            (init_resource::<TriggerEventTimer>, display_trigger_or_skip),
        )
        .add_systems(
            Update,
            wait_for_trigger.run_if(in_state(GameState::TriggerEvent)),
        )
        .add_systems(
            OnExit(GameState::TriggerEvent),
            remove_resource::<TriggerEventTimer>,
        )
        .add_systems(
            OnEnter(GameState::Navigation),
            (
                (
                    mark_room_cleared,
                    #[cfg(feature = "sqlite")]
                    save_game,
                )
                    .chain(),
                navigation_enter,
            ),
        )
        .add_systems(
            OnExit(GameState::Navigation),
            despawn_filtered::<With<EntranceDirection>>,
        )
        .add_plugins(CombatPlugin);
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::Game)]
#[states(scoped_entities)]
pub enum GameState {
    /// The initial state in the GameState loop that
    /// displays the room.
    /// This will delete the old room's content, and spawn
    /// the new room's contents.
    ///
    /// OnEnter: Deletes old room contents (if any)
    ///          Spawns new room contents
    ///          Sets game state to `Combat` if there are enemies alive.
    ///          Otherwise set game state to `Navigation`
    #[default]
    EnterRoom,
    /// Trigger an event in a room. If that event
    /// is combat, this will switch to [`Combat`]
    TriggerEvent,
    /// The combat state. See [`CombatState`]
    Combat,
    /// The UI for navigation pops up,
    /// and any things in the room are there.
    /// i.e. Item chests and spike traps
    Navigation,
    /// Show game over screen.
    /// This happens when all of your party members die.
    GameOver,
    /// Shows victory screen and concludes the game.
    Victory,
}

#[derive(Resource)]
pub struct TriggerEventTimer {
    trigger_timer: Timer,
    pause_timer: Timer,
}

impl Default for TriggerEventTimer {
    fn default() -> Self {
        Self {
            trigger_timer: Timer::from_seconds(1.0, TimerMode::Once),
            pause_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct EventRng(pub RandomSource);
// Whenever we change rooms,
// despawn all that are in the old room.

/// The default player positons in Axial coordinate space

const PLAYER_POSITIONS: [IVec2; 3] = [IVec2::new(-1, -1), IVec2::new(1, -2), IVec2::new(2, -1)];

fn place_player_actors(
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
    mut actors: Query<(Entity, &mut Transform), With<Actor>>,
) {
    let (map_size, grid_size, tile_size, map_type, map_anchor) = *tilemap;

    let center_tile_pos = UVec2 {
        x: map_size.x / 2,
        y: map_size.y / 2,
    };

    for ((entity, mut transform), pos_offset) in actors.iter_mut().zip(PLAYER_POSITIONS.into_iter())
    {
        let actor_pos: TilePos = (center_tile_pos.as_ivec2() + pos_offset).as_uvec2().into();
        let world_pos =
            actor_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);

        *transform = Transform::from_xyz(world_pos.x, world_pos.y, ACTOR_LAYER);

        commands
            .entity(entity)
            .insert((Pickable::default(), Visibility::Visible));
    }
}

fn init_room_rng(mut commands: Commands, info: Single<&RoomInfo, With<CurrentRoom>>) {
    commands.insert_resource(EventRng(RandomSource::seed_from_u64(info.rng_seed)));
}

fn set_room_rng(
    info: Single<&RoomInfo, (With<CurrentRoom>, Added<CurrentRoom>)>,
    mut rng: ResMut<EventRng>,
) {
    rng.0 = RandomSource::seed_from_u64(info.rng_seed);
}

/// Shows a text box with the event happening,
/// or if no event should happen (i.e. the room is empty or already cleared)
/// Skip to the navigation state.
fn display_trigger_or_skip(
    mut commands: Commands,
    info: Single<&RoomInfo, With<CurrentRoom>>,
    mut game_state: ResMut<NextState<GameState>>,
    style: Res<Style>,
) {
    let RoomInfo {
        cleared, r_type, ..
    } = *info;

    if *cleared || *r_type == RoomType::EmptyRoom || *r_type == RoomType::Entrance {
        game_state.set(GameState::Navigation);
    } else {
        use RoomType as R;
        let event_text = match r_type {
            R::EmptyRoom | R::Entrance | R::Pillar => unreachable!(),
            R::Combat(_) => format!("Monsters attack!"),
            R::Pit(damage) => format!("You fell in a Pit O' Doom!\n\t    -{} Health", damage),
            // TODO: Display item name when we can
            R::Item(item) => format!("Found item: None"),
        };

        commands.spawn((
            Node {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..default()
            },
            Text::new(event_text),
            StateScoped(GameState::TriggerEvent),
            style.font(100.0),
            TextColor(style.text_color),
        ));
        // Display event text
    }
}

/// Waits for a time so the player can see the event, then do the event.
/// TODO: Let users skip over this by pressing space or something.
fn wait_for_trigger(
    mut commands: Commands,
    mut timer: ResMut<TriggerEventTimer>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    info: Single<&RoomInfo, With<CurrentRoom>>,
) {
    let RoomInfo { r_type, .. } = *info;

    let trigger = &mut timer.trigger_timer;
    if !trigger.finished() {
        trigger.tick(time.delta());
        if trigger.just_finished() {
            commands.run_system_cached(trigger_event);
        }
    } else {
        let pause = &mut timer.pause_timer;
        pause.tick(time.delta());
        if pause.just_finished() {
            if let RoomType::Combat(_) = &r_type {
                game_state.set(GameState::Combat);
            } else {
                game_state.set(GameState::Navigation)
            }
        }
    }
}

fn trigger_event(
    info: Single<&RoomInfo, With<CurrentRoom>>,
    mut actor_q: Query<&mut Health>,
    mut event_rng: ResMut<EventRng>,
) {
    let RoomInfo {
        cleared, r_type, ..
    } = *info;
    assert!(!*cleared);

    use RoomType as R;
    match r_type {
        R::EmptyRoom | R::Entrance => unreachable!(),
        R::Combat(_) => {}
        R::Pit(damage) => {
            let actor_count = actor_q.iter().filter(|h| h.is_alive()).count();
            assert!(actor_count > 0);

            let actor_damaged = event_rng.random_range(0..actor_count);

            actor_q
                .iter_mut()
                .filter(|h| h.is_alive())
                .skip(actor_damaged)
                .next()
                .unwrap()
                .damage_no_one_shot(*damage);
        }
        R::Item(item) => {}
        R::Pillar => {}
    }
}

fn navigation_enter(
    mut commands: Commands,
    current_room: Single<&TilePos, With<CurrentRoom>>,
    map_map: Single<(&TilemapSize, &TileStorage), (With<MapTilemap>, Without<RoomTilemap>)>,
    mut room_map: Single<(Entity, &mut TileStorage), (With<RoomTilemap>, Without<MapTilemap>)>,
    maptile_q: Query<&TileTextureIndex>,
) {
    let (map_size, map_storage) = *map_map;

    let (room_entity, ref mut room_storage) = *room_map;

    let room_center = ROOM_CENTER;

    let neighbors =
        HexNeighbors::<TilePos>::get_neighboring_positions_standard(&current_room, map_size);

    let door_directions = neighbors
        .iter()
        .zip(EntranceDirection::ALL)
        .filter_map(|(neighbor, dir)| map_storage.checked_get(neighbor).map(|n| (n, dir)))
        .filter_map(|(entity, dir)| {
            maptile_q
                .get(entity)
                .is_ok_and(|t| *t != TileTextureIndex(OUTLINE_TILE))
                .then_some(dir)
        });

    commands.entity(room_entity).with_children(move |parent| {
        for dir in door_directions {
            let tile_pos = dir.door_offset(&room_center, ROOM_RADIUS, HEX_COORD_SYSTEM);

            let id = parent
                .spawn((
                    StateScoped(GameState::Navigation),
                    dir,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(room_entity),
                        texture_index: TileTextureIndex(DOOR_TILE_VARIENT),
                        ..Default::default()
                    },
                ))
                .id();
            room_storage.set(&tile_pos, id);
        }
    });
}
