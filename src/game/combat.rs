use super::*;
use crate::prelude::*;
use crate::room::ROOM_SIZE;
use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<CombatState>()
            .add_systems(
                OnEnter(GameState::Combat),
                (setup_turn_order, store_actor_positions),
            )
            .add_systems(OnEnter(CombatState::TurnSetup), prep_turn_order)
            .add_systems(OnEnter(CombatState::MoveToCenter), move_to_center)
            .add_systems(OnEnter(CombatState::MoveBack), move_back)
            .add_systems(
                Update,
                (
                    move_to_target.run_if(in_state(CombatState::MoveToCenter)),
                    move_to_center_check.run_if(in_state(CombatState::MoveToCenter)),
                    move_to_target.run_if(in_state(CombatState::MoveBack)),
                    move_back_check.run_if(in_state(CombatState::MoveBack)),
                ),
            )
            .add_systems(OnExit(GameState::Combat), cleanup_positions);
    }
}

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
//The current acting actor
#[derive(Component)]
pub struct ActingActor;

/// The combat queue of actors
#[derive(Resource)]
pub struct TurnOrder {
    queue: VecDeque<Entity>,
}

impl TurnOrder {
    pub fn new(actor_q: Query<Entity, With<Actor>>, speed_q: Query<&AttackSpeed>) -> Self {
        let mut queue = actor_q.iter().collect::<VecDeque<_>>();

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

        // + 1 because we skipped one
        self.queue.rotate_left(idx + 1);
    }

    pub fn teams_alive(&mut self, actor_q: Query<(&Health, &Team)>) -> TeamAlive {
        self.queue
            .iter()
            .map(|e| actor_q.get(*e).unwrap())
            .filter_map(|(health, team)| health.is_alive().then_some(team))
            .fold(TeamAlive::Neither, |acc, elm| acc.found(elm))
    }

    pub fn queue(&self) -> &VecDeque<Entity> {
        &self.queue
    }
}

/// The action being taken by the acting actor
#[derive(Resource, Deref, DerefMut)]
pub struct ActingActorActon(pub Action);

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TeamAlive {
    Both,
    Player,
    Enemy,
    Neither,
}

impl TeamAlive {
    pub fn found(&self, team: &Team) -> Self {
        match (team, self) {
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

#[derive(Component, Deref, DerefMut)]
pub struct ActorOriginalPosition(pub Vec2);

#[derive(Component, Deref, DerefMut)]
pub struct ActorTargetPosition(pub Vec2);

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

//sets up the turn queue
fn setup_turn_order(
    mut commands: Commands,
    actor_q: Query<Entity, With<Actor>>,
    speed_q: Query<&AttackSpeed>,
) {
    commands.insert_resource(TurnOrder::new(actor_q, speed_q));
}

//stores the actors original positions
fn store_actor_positions(
    mut commands: Commands,
    actors_q: Query<(Entity, &Transform), With<Actor>>,
) {
    for (entity, transform) in actors_q.iter() {
        commands
            .entity(entity)
            .insert(ActorOriginalPosition(transform.translation.xy()));
    }
}

//removes the actors original positions
fn cleanup_positions(mut commands: Commands, queue: ResMut<TurnOrder>) {
    commands
        .entity(queue.active())
        .remove::<ActorOriginalPosition>()
        .remove::<ActorTargetPosition>();
}

//sets the active actor and insert the composnent
fn prep_turn_order(
    mut commands: Commands,
    mut queue: ResMut<TurnOrder>,
    mut next_state: ResMut<NextState<CombatState>>,
    actor_q: Query<(&Health, &Team)>,
    health_q: Query<&Health>,
) {
    match queue.teams_alive(actor_q) {
        TeamAlive::Both => {
            commands.entity(queue.active()).remove::<ActingActor>();
            queue.skip_to_next(health_q);
            commands.entity(queue.active()).insert(ActingActor);
            next_state.set(CombatState::MoveToCenter);
        }
        // End the turn in this case (likely another function)
        TeamAlive::Player | TeamAlive::Enemy | TeamAlive::Neither => {
            commands.entity(queue.active()).remove::<ActingActor>();
        }
    }
}

//sets target postion to be center
fn move_to_center(mut commands: Commands, active_actor: Single<Entity, With<ActingActor>>) {
    //set the center_world_pos
    let center_tile_pos = TilePos {
        x: ROOM_SIZE.x / 2,
        y: ROOM_SIZE.y / 2,
    };

    let center_world_pos = center_tile_pos.center_in_world(
        &ROOM_SIZE,
        &TilemapGridSize {
            x: TILE_SIZE.x,
            y: TILE_SIZE.y,
        },
        &TILE_SIZE,
        &TilemapType::Hexagon(HexCoordSystem::Row),
        &TilemapAnchor::Center,
    );
    //Set a component with the target position
    commands
        .entity(*active_actor)
        .insert(ActorTargetPosition(center_world_pos));
}

//checks if actor is in center and then sets the state
fn move_to_center_check(
    mut commands: Commands,
    mut next_state: ResMut<NextState<CombatState>>,
    active_actor: Single<(Entity, &Transform, &ActorTargetPosition), With<ActingActor>>,
) {
    //Encapsulate the set state in a check that checks if transform equals target position
    let (entity, transform, target) = active_actor.into_inner();
    if transform.translation.xy() == target.0 {
        commands.entity(entity).remove::<ActorTargetPosition>();
        next_state.set(CombatState::ChooseAction);
    }
}

//sets the target position to the actors original position
fn move_back(
    mut commands: Commands,
    active_actor: Single<(Entity, &ActorOriginalPosition), With<ActingActor>>,
) {
    let (entity, origin) = active_actor.into_inner();
    commands
        .entity(entity)
        .insert(ActorTargetPosition(origin.0));
}

//checks if actor is in original positions and then sets the state
fn move_back_check(
    mut commands: Commands,
    mut next_state: ResMut<NextState<CombatState>>,
    active_actor: Single<(Entity, &Transform, &ActorTargetPosition), With<ActingActor>>,
) {
    //Encapsulate the set state in a check that checks if transform equals target position
    let (entity, transform, target) = active_actor.into_inner();
    if transform.translation.xy() == target.0 {
        commands.entity(entity).remove::<ActorTargetPosition>();
        next_state.set(CombatState::EndOfTurn);
    }
}

//Moves the ActingActor to target position and then removes target position
fn move_to_target(
    mut active_actor: Single<(Entity, &mut Transform, &ActorTargetPosition), With<ActingActor>>,
    time: Res<Time>,
) {
    let (entity, mut transform, target_pos) = active_actor.into_inner();

    let direction = target_pos.0 - transform.translation.xy();
    let distance = direction.length();
    let movement = direction.normalize_or_zero() * (300.0 * time.delta_secs()).clamp(0.0, distance);
    transform.translation += movement.extend(0.0);
}
