use crate::prelude::*;
use crate::room::{CurrentRoom, spawn_room};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

const PLAYER_LAYER: f32 = 1.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameState>().add_systems(
            OnEnter(AppState::Game),
            (setup_current_room, spawn_room, place_actors).chain(),
        );
        app.add_systems(OnEnter(GameState::Navigation), crate::saving::save_game);
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

// Whenever we change rooms,
// despawn all that are in the old room.

/// OnEnter: Set [`TurnOrder`]
///          Place actors where they should go
///          Etc.
/// OnExit:  Removes [`TurnOrder`]
#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Combat)]
#[states(scoped_entities)]
pub enum CombatState {
    /// Everything to set up the turn that is about to come
    ///
    /// [`ActingActor`] is front of queue
    /// Asserts it is not empty
    ///
    /// Set [`ActingActor`]
    #[default]
    TurnSetup,
    /// Move the choosen actor to the next state.
    ///
    /// Update: Move [`AttackingActor`]
    MoveToCenter,
    /// The player is prompted or the monster
    /// randomizes the attack
    ///
    /// OnEnter: if [`ActingActor`] is automated, decide the attack and move on
    ///          OTHERWISE Show UI
    /// Update:  User interaction, if user picks action, set it as [`ActingActorAction`]
    ChooseAction,
    /// The attacking actor does the attack
    /// and the attackee gets hurt
    ///
    /// Update: Update animation timer
    ///         When timer done, move on
    /// OnExit: Deal Damage
    ///         Removes [`ActingActorAction`]
    ///
    /// If an actor gets an additional turn,
    /// go back to `ChooseAction`
    PerformAction,
    /// The actor moves back to where they belong
    /// After, sets next [`CombatState`]
    ///
    /// Update: Move [`AttackingActor`]
    MoveBack,
    /// Remove all the dead actors for [`TurnOrder`]
    /// Ends the turn if at most 1 team survives
    ///
    /// If both teams are alive, move to [`TurnSetup`]
    /// Rotate Left [`TurnOrder`]
    EndOfTurn,
}

/// The combat queue of actors
#[derive(Resource)]
pub struct TurnOrder {
    queue: VecDeque<Entity>,
}

impl TurnOrder {
    pub fn new(actors: &[Entity]) -> Self {
        Self {
            queue: Vec::from(actors).into(),
        }
    }

    /// asserts that the queue isn't empty
    pub fn first(&self) -> Entity {
        *self.queue.front().unwrap()
    }

    pub fn queue(&self) -> &VecDeque<Entity> {
        &self.queue
    }
}

/// The action being taken by the acting actor
#[derive(Resource, Deref, DerefMut)]
pub struct ActingActorActon(pub Action);

/// The action the [`ActingActor`] is taking
pub enum Action {
    /// The actor does damage to the `target`
    Attack {
        target: Entity,
    },
    // TBD
    SpecialAction {
        target: Entity,
    },
    /// The actor does damage to the `target`
    UseItem {
        item: (),
        target: Entity,
    },
    SkipTurn,
}

/// The default player positons in Axial coordinate space

const PLAYER_POSITIONS: [IVec2; 3] = [IVec2::new(-1, -1), IVec2::new(1, -2), IVec2::new(2, -1)];
const ENEMY_POSITIONS: [IVec2; 3] = [IVec2::new(1, 1), IVec2::new(-1, 2), IVec2::new(-2, 1)];

fn setup_current_room(mut commands: Commands) {
    commands.insert_resource(CurrentRoom(RoomInfo::Entrance));
}

fn place_actors(
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
    mut actors: Query<(Entity, &mut Transform, &Team)>,
) {
    let (map_size, grid_size, tile_size, map_type, map_anchor) = *tilemap;

    let center_tile_pos = UVec2 {
        x: map_size.x / 2,
        y: map_size.y / 2,
    };

    let mut player_pos = PLAYER_POSITIONS.into_iter();
    let mut enemy_pos = ENEMY_POSITIONS.into_iter();
    for (entity, mut transform, team) in actors.iter_mut() {
        let pos_offset = match *team {
            Team::Player => player_pos.next().unwrap(),
            Team::Enemy => enemy_pos.next().unwrap(),
        };

        let actor_pos: TilePos = (center_tile_pos.as_ivec2() + pos_offset).as_uvec2().into();
        let world_pos =
            actor_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);

        *transform = Transform::from_xyz(world_pos.x, world_pos.y, PLAYER_LAYER);

        commands
            .entity(entity)
            .insert((Pickable::default(), Visibility::Visible));
    }
}
