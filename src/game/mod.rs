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
    pub fn new(actors: &[Entity], speed_q: Query<&AttackSpeed>) -> Self {
        let mut queue = VecDeque::from(Vec::from(actors));

        queue.shrink_to_fit();
        queue
            .make_contiguous()
            .sort_by_cached_key(|entity| speed_q.get(*entity).unwrap().0);

        Self { queue }
    }

    /// Gets the active actor.
    /// asserts that the queue isn't empty
    pub fn active(&self) -> Entity {
        *self.queue.front().unwrap()
    }

    /// Should be called at end of turn to set the first actor in the
    /// queue as the first elegible actor to take a turn (i.e. skipping over dead actors)
    ///
    /// Asserts at least 1 actor is left alive.
    pub fn skip_to_next(&mut self, health_q: Query<&Health>) {
        let idx = self
            .queue
            .iter()
            // skip the one that was alive last round
            .skip(1)
            .filter_map(|entity| health_q.get(*entity).ok())
            .enumerate()
            .find_map(|(idx, health)| health.is_alive().then_some(idx))
            .unwrap();

        self.queue.rotate_left(idx + 1);
    }

    pub fn teams_alive(&mut self, actor_q: Query<(&Health, &Team)>) -> TeamAlive {
        self.queue
            .iter()
            .map(|e| actor_q.get(*e).unwrap())
            .filter_map(|(health, team)| health.is_alive().then_some(team))
            .fold(TeamAlive::Neither, |acc, elm| acc.found(*elm))
    }

    pub fn queue(&self) -> &VecDeque<Entity> {
        &self.queue
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TeamAlive {
    Both,
    Player,
    Enemy,
    Neither,
}

impl TeamAlive {
    pub fn found(&self, team: Team) -> Self {
        match (team, *self) {
            (_, Self::Both) => Self::Both,
            (Team::Player, Self::Player) => Self::Player,
            (Team::Enemy, Self::Enemy) => Self::Enemy,
            (Team::Enemy, Self::Player) => Self::Both,
            (Team::Player, Self::Enemy) => Self::Both,
            (Team::Player, Self::Neither) => Self::Player,
            (Team::Enemy, Self::Neither) => Self::Enemy,
        }
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

fn prep_turn_order(
    mut queue: ResMut<TurnOrder>,
    actor_q: Query<(&Health, &Team)>,
    health_q: Query<&Health>,
) {
    match queue.teams_alive(actor_q) {
        TeamAlive::Both => {}
        // End the turn in this case (likely another function)
        TeamAlive::Player | TeamAlive::Enemy | TeamAlive::Neither => {}
    }
    queue.skip_to_next(health_q);
}

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
