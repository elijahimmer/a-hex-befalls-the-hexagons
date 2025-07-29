mod combat;

pub use combat::*;

use crate::prelude::*;
use crate::room::{CurrentRoom, InRoom, mark_room_cleared, spawn_room, spawn_room_entities};
use crate::saving::save_game;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameState>();

        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<GameState>);

        app.add_systems(
            OnEnter(AppState::Game),
            (setup_current_room, spawn_room, place_player_actors).chain(),
        )
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
            (wait_for_trigger).run_if(in_state(GameState::TriggerEvent)),
        )
        .add_systems(
            OnExit(GameState::TriggerEvent),
            remove_resource::<TriggerEventTimer>,
        )
        .add_systems(
            OnEnter(GameState::Navigation),
            ((mark_room_cleared, save_game).chain(), navigation_enter),
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

// Whenever we change rooms,
// despawn all that are in the old room.

/// The default player positons in Axial coordinate space

const PLAYER_POSITIONS: [IVec2; 3] = [IVec2::new(-1, -1), IVec2::new(1, -2), IVec2::new(2, -1)];

/// TODO: Remove this and set it in new game or load game
fn setup_current_room(mut commands: Commands) {
    commands.insert_resource(CurrentRoom(RoomInfo::from_type(RoomType::Combat(
        Box::new([ActorName::Ogre, ActorName::Goblin, ActorName::Skeleton]),
    ))));
}

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
    mut actors: Query<(Entity, &mut Transform), With<ActorName>>,
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

/// Shows a text box with the event happening,
/// or if no event should happen (i.e. the room is empty or already cleared)
/// Skip to the navigation state.
fn display_trigger_or_skip(
    mut commands: Commands,
    room: Res<CurrentRoom>,
    mut game_state: ResMut<NextState<GameState>>,
    style: Res<Style>,
) {
    if room.cleared
        || room.r_type == RoomType::EmptyRoom
        || room.r_type == RoomType::Entrance
        || room.r_type == RoomType::Exit
    {
        game_state.set(GameState::Navigation);
    } else {
        use RoomType as R;
        let event_text = match &room.r_type {
            R::EmptyRoom | R::Entrance | R::Exit => unreachable!(),
            R::Combat(_) => format!("Monsters attack!"),
            R::Pit(_) => format!("You fell in a Pit O' Doom!"),
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
    room: Res<CurrentRoom>,
) {
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
            if let RoomType::Combat(_) = room.r_type {
                game_state.set(GameState::Combat);
            } else {
                game_state.set(GameState::Navigation)
            }
        }
    }
}

fn trigger_event(room: Res<CurrentRoom>) {
    assert!(!room.cleared);
    use RoomType as R;
    match &room.r_type {
        R::EmptyRoom | R::Entrance => unreachable!(),
        R::Combat(_) => {}
        R::Exit => {}
        R::Pit(damage) => {}
        R::Item(item) => {}
    }
}

fn navigation_enter(
    mut commands: Commands,
    style: Res<Style>,
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
}
