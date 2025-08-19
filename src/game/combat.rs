use super::*;
use crate::prelude::*;
use crate::update_player_hp_bar;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;
use std::fmt;

pub struct CombatPlugin;
const ACTOR_SPEED: f32 = 300.0;
const DAMAGE_MULTIPLIER: f32 = 1.2;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<CombatState>();

        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<CombatState>);
        app.add_systems(
            OnEnter(GameState::Combat),
            (setup_turn_order, store_actor_positions),
        )
        .add_systems(OnEnter(CombatState::TurnSetup), prep_turn_order)
        .add_systems(OnEnter(CombatState::MoveToCenter), move_to_center)
        .add_systems(OnEnter(CombatState::MoveBack), move_back)
        .add_systems(
            Update,
            (move_to_target, move_to_center_check).run_if(in_state(CombatState::MoveToCenter)),
        )
        .add_systems(OnEnter(CombatState::CheckTeam), check_team)
        .add_systems(OnEnter(CombatState::MonsterAttack), choose_action)
        .add_systems(
            OnEnter(CombatState::SpawnMenu),
            attack_options::create_attack_menu,
        )
        .add_systems(
            OnEnter(CombatState::PerformAction),
            (despawn_attack_menu, perform_action).chain(),
        )
        .add_systems(
            Update,
            (move_to_target, move_back_check).run_if(in_state(CombatState::MoveBack)),
        )
        .add_systems(OnEnter(CombatState::EndOfTurn), end_turn)
        .add_systems(OnExit(GameState::Combat), cleanup_positions);
    }
}

////////////////////////ENUMS////////////////////////////

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
    /// Spawns Menu
    SpawnMenu,
    /// Checks which Team is Attacking
    CheckTeam,
    /// Monster Attack
    MonsterAttack,
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

/// The action the [`ActingActor`] is taking
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
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

////////////COMPONENTS//////////////////

//The current acting actor
#[derive(Component)]
pub struct ActingActor;

//Stoes the original positions of all actors
#[derive(Component, Deref, DerefMut)]
pub struct ActorOriginalPosition(pub Vec2);

//Stores the position that actor is going to
#[derive(Component, Deref, DerefMut)]
pub struct ActorTargetPosition(pub Vec2);

////////////RESOURCES//////////////////
/// The action being taken by the acting actor
#[derive(Resource, Deref, DerefMut)]
pub struct ActingActorAction(pub Action);

/// The combat queue of actors
#[derive(Resource, Debug)]
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
        *self.queue.back().unwrap()
    }

    /// Should be called at end of turn to set the first actor in the
    /// queue as the first elegible actor to take a turn (i.e. skipping over dead actors)
    ///
    /// Asserts at least 1 actor is left alive.
    pub fn skip_to_next(&mut self, health_q: Query<&Health>) {
        let idx = self
            .queue
            .iter()
            .rev()
            .enumerate()
            .skip(1)
            .filter_map(|(idx, entity)| health_q.get(*entity).ok().map(|a| (idx, a)))
            .find_map(|(idx, health)| health.is_alive().then_some(idx))
            .unwrap();

        // + 1 because we skipped one
        self.queue.rotate_right(idx);

        assert!(health_q.get(self.active()).unwrap().is_alive());
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

    pub fn display_with_names(&self, name_q: &Query<&ActorName>) -> String {
        self.queue
            .iter()
            .map(|entity| {
                name_q
                    .get(*entity)
                    .map(|name| name.to_string())
                    .unwrap_or("Unknown".to_string())
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

////////////////EVENTS///////////////////

//An event for when an action is done
#[derive(Event)]
pub struct ActionEvent {
    pub actor: Entity,
    pub action: Action,
    pub target: Entity,
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
    name_q: Query<&ActorName>,
) {
    println!("Turn order: {}", queue.display_with_names(&name_q));
    match queue.teams_alive(actor_q) {
        TeamAlive::Both => {
            //commands.entity(queue.active()).remove::<ActingActor>();
            commands.entity(queue.active()).insert(ActingActor);
            next_state.set(CombatState::MoveToCenter);
        }
        // End the turn in this case (likely another function)
        TeamAlive::Player | TeamAlive::Enemy | TeamAlive::Neither => {
            commands.entity(queue.active()).remove::<ActingActor>();
        }
    }
    println!("Turn order: {}", queue.display_with_names(&name_q));
}

//////////FROM HERE ARE MOVEMENT SYSTEMS//////////////////

//sets target postion to be center
fn move_to_center(
    mut commands: Commands,
    active_actor: Single<Entity, With<ActingActor>>,
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
    //set the center_world_pos
    let (map_size, grid_size, tile_size, map_type, map_anchor) = *tilemap;

    let center_tile_pos = TilePos {
        x: map_size.x / 2,
        y: map_size.y / 2,
    };

    let center_world_pos =
        center_tile_pos.center_in_world(&map_size, &grid_size, &tile_size, &map_type, &map_anchor);
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
        next_state.set(CombatState::CheckTeam);
        //next_state.set(CombatState::MoveBack);
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

fn check_team(
    mut next_state: ResMut<NextState<CombatState>>,
    team_q: Query<&Team, With<ActingActor>>,
) {
    for team in team_q {
        match team {
            Team::Player => next_state.set(CombatState::SpawnMenu),
            Team::Enemy => next_state.set(CombatState::MonsterAttack),
        }
    }
}

//Moves the ActingActor to target position and then removes target position
fn move_to_target(
    mut active_actor: Single<(&mut Transform, &ActorTargetPosition), With<ActingActor>>,
    time: Res<Time>,
) {
    let (ref mut transform, target_pos) = *active_actor;

    let direction = target_pos.0 - transform.translation.xy();
    let distance = direction.length();
    let movement =
        direction.normalize_or_zero() * (ACTOR_SPEED * time.delta_secs()).clamp(0.0, distance);
    transform.translation += movement.extend(0.0);
}

////////////////Choose action/////////////////////
pub fn choose_action(
    mut commands: Commands,
    mut next_state: ResMut<NextState<CombatState>>,
    mut rng: ResMut<EventRng>,
    queue: ResMut<TurnOrder>,
    active_actor: Single<(Entity, &Team), With<ActingActor>>,
    actor_q: Query<(&Health, &Team)>,
) {
    //remove any current action
    let (_, team) = *active_actor;
    let targets: Vec<Entity> = queue
        .queue()
        .iter()
        .filter_map(|&entity| {
            if let Ok((health, target_team)) = actor_q.get(entity) {
                if health.is_alive() && *target_team != *team {
                    Some(entity)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let chosen_target = targets[rng.random_range(0..targets.len())];
    let combat_action = Action::Attack {
        target: chosen_target,
    };
    debug!("CHOSEN TARGET {:?}", chosen_target);

    // Get the Attack and do .conduct on that

    commands.insert_resource(ActingActorAction(combat_action));
    next_state.set(CombatState::PerformAction);
}

///////////////Perform Action///////////////////

fn perform_action(
    mut commands: Commands,
    mut next_state: ResMut<NextState<CombatState>>,
    mut rng: ResMut<EventRng>,
    active_actor: Single<(Entity, &Attack), With<ActingActor>>,
    actor_action: Res<ActingActorAction>,
    mut actor_q: Query<(&mut Health, &BlockChance), With<Actor>>,
    actor_name: Single<&ActorName, With<ActingActor>>,
) {
    let (_, a_attack) = *active_actor;
    match **actor_action {
        Action::Attack { target } => {
            let attack = a_attack.clone();

            let attack_result = attack.conduct(&mut *rng);
            debug!("ATTACK RESULT {:?}", attack_result);

            match attack_result {
                AttackDamage::Hit(damage) => {
                    if let Ok((mut target_health, block_chance)) = actor_q.get_mut(target) {
                        debug!("TARGETS BLOCK CHANCE: {}\n", block_chance.0);
                        let blocked = rng.random_bool(block_chance.0.into());
                        debug!("Block chance: {:?}, Blocked: {}\n", block_chance.0, blocked);
                        if !blocked {
                            target_health.damage(damage.get());
                            let current_health =
                                target_health.current().map(|h| h.get()).unwrap_or(0);
                            debug!(
                                "DAMAGE DEALT: {}, TARGET HEALTH: {}\n",
                                damage.get(),
                                current_health
                            );

                            if !target_health.is_alive() {
                                debug!("{:?} IS DEAD!!!!!!!!!!!!!!\n", target);
                            }
                        }
                    }
                }
                AttackDamage::Miss => {
                    debug!("MISSED!!!!!!!!!!!!!!\n");
                }
            }
        }
        Action::SpecialAction { target } => match **actor_name {
            ActorName::Warrior => {
                if let Ok((mut target_health, _)) = actor_q.get_mut(target) {
                    let attack_result = a_attack.conduct(&mut *rng);
                    match attack_result {
                        AttackDamage::Hit(damage) => {
                            let extra_damage = (damage.get() as f32 * DAMAGE_MULTIPLIER) as u32;
                            target_health.damage(extra_damage);
                        }
                        AttackDamage::Miss => {}
                    }
                }
            }
            ActorName::Priestess => {
                if let Ok((mut target_health, _)) = actor_q.get_mut(target) {
                    let health_before = target_health.current().map(|h| h.get()).unwrap_or(0);
                    debug!("target {} health is {}", target, health_before);
                    let heal_num = rng.random_range(15..30);
                    target_health.heal_or_revive(heal_num);
                    let health_after = target_health.current().map(|h| h.get()).unwrap_or(0);
                    debug!(
                        "{} has healed {} points, health is now {}",
                        target, heal_num, health_after
                    );
                }
            }
            ActorName::Theif => {
                let theif_attack = a_attack.clone();

                let attack_result = theif_attack.conduct(&mut *rng);

                match attack_result {
                    AttackDamage::Hit(damage) => {
                        if let Ok((mut target_health, block_chance)) = actor_q.get_mut(target) {
                            let blocked = rng.random_bool(block_chance.0.into());
                            if !blocked {
                                target_health.damage(damage.get());
                            }
                        }
                    }
                    AttackDamage::Miss => {
                        debug!("MISSED!!!!!!!!!!!!!!\n");
                    }
                }
            }
            _ => {}
        },

        Action::UseItem { target, item } => {}
        Action::SkipTurn => {}
    }

    commands.run_system_cached(update_player_hp_bar);

    next_state.set(CombatState::MoveBack);
}

pub fn end_turn(
    mut commands: Commands,
    mut queue: ResMut<TurnOrder>,
    mut next_state: ResMut<NextState<CombatState>>,
    mut update_gamestate: ResMut<NextState<GameState>>,
    actor_q: Query<(&Health, &Team)>,
    health_q: Query<&Health>,
    actor_name: Single<&ActorName, With<ActingActor>>,
    actor_action: Res<ActingActorAction>,
) {
    if matches!(**actor_name, ActorName::Theif)
        && matches!(actor_action.0, Action::SpecialAction { .. })
    {
    } else {
        commands.entity(queue.active()).remove::<ActingActor>();
        queue.skip_to_next(health_q);
    }
    commands.remove_resource::<ActingActorAction>();

    match queue.teams_alive(actor_q) {
        TeamAlive::Both => {
            next_state.set(CombatState::TurnSetup);
        }

        //TODO: If you have time, despawn enemies
        TeamAlive::Player => {
            debug!("Players won");
            update_gamestate.set(GameState::Navigation);
        }
        TeamAlive::Enemy => {
            debug!("ENEMY WON");
            update_gamestate.set(GameState::GameOver);
        }
        TeamAlive::Neither => {
            debug!("Everyone is dead!!!!!");
            update_gamestate.set(GameState::GameOver);
        }
    }
}
