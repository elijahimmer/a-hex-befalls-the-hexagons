use crate::prelude::*;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<BattleState>()
            .add_event::<CombatEndEvent>()
            .add_event::<TurnStartEvent>()
            .init_resource::<CombatState>();
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::Game)]
#[states(scoped_entities)]
pub enum BattleState {
    #[default]
    PlayerTurn,
    EnemyTurn,
}

#[derive(Event)]
pub struct CombatEndEvent {
    pub winner: Team,
}

#[derive(Event)]
pub struct TurnStartEvent {
    pub actor: Entity,
    pub team: Team,
}

#[derive(Resource, Default)]
pub struct CombatState {
    pub combat_order: Vec<Entity>,
}

fn determine_combat_order(
    mut combat_state: ResMut<CombatState>,
    actors: Query<(Entity, &Attack), With<Health>>,
) {
    //Make tuples so I can sort by speed Note: find a way to get rid of speed when adding to combat
    //order
    let mut actor_speeds: Vec<(Entity, u32)> = actors
        .iter()
        .map(|(entity, attack)| (entity, attack.speed))
        .collect();
    
    //sort by speed
    ////////////////

    //This needs to adds to combat order and gets rid of speed
    combat_state.combat_order = actor_speeds
        .into_iter()
        .map(|(entity, _)| entity)
        .collect();
}


//Th
fn manage_combat_turns(
    mut combat_state: ResMut<CombatState>,
    mut next_state: ResMut<NextState<BattleState>>,
    mut turn_events: EventWriter<TurnStartEvent>,
    actors: Query<(&Team, &Health)>,
) {}


//This will check if combat is over or not and handle what comes next.
fn check_combat_end(
    mut next_state: ResMut<NextState<BattleState>>,
    mut combat_state: ResMut<CombatState>,
    actors: Query<(&Health, &Team)>,
    mut combat_end_events: EventWriter<CombatEndEvent>,
) {}





