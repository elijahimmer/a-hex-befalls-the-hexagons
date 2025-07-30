use super::*;
use crate::prelude::*;
use bevy::{prelude::*, state::commands};
use std::collections::HashMap;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<CombatState>()
            .add_systems(OnEnter(GameState::Combat), (setup_turn_order, store_actor_positions))
            .add_systems(OnEnter(CombatState::TurnSetup), prep_turn_order)
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
#[derive(Resource, Deref, DerefMut)]
pub struct ActingActor(pub Entity);

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


#[derive(Resource)]
pub struct ActorOriginalPositions {
    positions: HashMap<Entity, Vec3>,
}

impl ActorOriginalPositions {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }
    pub fn store_position(&mut self, entity: Entity, position: Vec3) {
        self.positions.insert(entity, position);
    }
    pub fn get_position(&self, entity: Entity) -> Option<Vec3> {
        self.positions.get(&entity).copied()
    }
}






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

fn setup_turn_order(
    mut commands: Commands,
    actor_q: Query<Entity, With<Actor>>,
    speed_q: Query<&AttackSpeed>,
) {
    commands.insert_resource(TurnOrder::new(actor_q, speed_q));
}

fn store_actor_positions(
    mut commands: Commands,
    actors_q: Query<(Entity, &Transform), With<ActorName>>,
) {
    let mut positions = ActorOriginalPositions::new();
    
    for (entity, transform) in actors_q.iter() {
        positions.store_position(entity, transform.translation);
        }
    commands.insert_resource(positions);
}

fn cleanup_positions(mut commands: Commands) {
    commands.remove_resource::<ActorOriginalPositions>();
}

fn prep_turn_order(
    mut commands: Commands,
    mut queue: ResMut<TurnOrder>,
    mut next_state: ResMut<NextState<CombatState>>,
    actor_q: Query<(&Health, &Team)>,
    health_q: Query<&Health>,
) {
    match queue.teams_alive(actor_q) {
        TeamAlive::Both => {
            queue.skip_to_next(health_q);
            assert!(!queue.queue().is_empty(), "The queu is empty");

            commands.insert_resource(ActingActor(queue.active()));
            next_state.set(CombatState::MoveToCenter);
        }
        // End the turn in this case (likely another function)
        TeamAlive::Player | TeamAlive::Enemy | TeamAlive::Neither => {
            commands.remove_resource::<ActingActor>();
        }
    }
}
