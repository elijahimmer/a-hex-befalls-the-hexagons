use super::*;
use crate::embed_asset;
use crate::menu::*;
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;
use std::fmt;

pub const BASIC_BUTTON_IMAGE_PATH: &str = "embedded://assets/sprites/Basic-button.png";
pub const MOVE_BANNER_IMAGE_PATH: &str = "embedded://assets/sprites/Move Banner.png";
pub const SPECIAL_MOVE_IMAGE_PATH: &str = "embedded://assets/sprites/Special Move.png";
pub const BUTTON_IMAGE_PATH: &str = "embedded://assets/sprites/buttons.png";
pub const GAMEOVER_IMAGE_PATH: &str = "embedded://assets/sprites/Game Over.png";
pub const VICTORY_IMAGE_PATH: &str = "embedded://assets/sprites/Victory.png";

pub struct AttackOptionsPlugin;

impl Plugin for AttackOptionsPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/Basic-button.png");
        embed_asset!(app, "assets/sprites/Move Banner.png");
        embed_asset!(app, "assets/sprites/Special Move.png");
        embed_asset!(app, "assets/sprites/buttons.png");
        embed_asset!(app, "assets/sprites/Game Over.png");
        embed_asset!(app, "assets/sprites/Victory.png");
    }
}

#[derive(Component)]
pub struct AttackMenu;

#[derive(Component)]
pub struct TargetActor;

pub fn create_attack_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<CombatState>>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(37.5),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                ..default()
            },
            AttackMenu,
        ))
        .with_children(|builder| {
            builder.spawn((
                ImageNode {
                    image: asset_server.load(MOVE_BANNER_IMAGE_PATH),
                    ..default()
                },
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_basis: Val::Px(54.0),
                    ..default()
                },
            ));

            builder
                .spawn((
                    ImageNode {
                        image: asset_server.load(BASIC_BUTTON_IMAGE_PATH),
                        ..default()
                    },
                    Node {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_basis: Val::Px(50.0),
                        ..default()
                    },
                    Button,
                ))
                .observe(basic_attack);

            builder
                .spawn((
                    ImageNode {
                        image: asset_server.load(SPECIAL_MOVE_IMAGE_PATH),
                        ..default()
                    },
                    Node {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_basis: Val::Px(50.0),
                        ..default()
                    },
                    Button,
                ))
                .observe(special_move);
        });
}

pub fn despawn_attack_menu(mut commands: Commands, menu_entity: Single<Entity, With<AttackMenu>>) {
    commands.entity(*menu_entity).despawn();
}

pub fn spawn_gameover_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    style: Res<Style>,
    keybinds: Res<Controls>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(GameState::GameOver),
        ))
        .with_children(|builder| {
            builder.spawn((
                ImageNode {
                    image: asset_server.load(GAMEOVER_IMAGE_PATH),
                    ..default()
                },
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_basis: Val::Px(100.0),
                    ..default()
                },
            ));

            builder
                .spawn((Node::default(),))
                .with_children(|builder| {
                    style.display_keybind(builder, &Keybind(Control::Pause, keybinds.pause))
                })
                .observe(exit_gameover);
        });
}

pub fn spawn_victory_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    style: Res<Style>,
    keybinds: Res<Controls>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                ..default()
            },
            StateScoped(GameState::Victory),
        ))
        .with_children(|builder| {
            builder.spawn((
                ImageNode {
                    image: asset_server.load(VICTORY_IMAGE_PATH),
                    ..default()
                },
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_basis: Val::Px(100.0),
                    ..default()
                },
            ));

            builder
                .spawn((Node {
                    align_content: AlignContent::Center,
                    ..default()
                },))
                .with_children(|builder| {
                    style.display_keybind(builder, &Keybind(Control::Pause, keybinds.pause))
                })
                .observe(exit_victory);
        });
}

fn basic_attack(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    rng: ResMut<EventRng>,
    queue: ResMut<TurnOrder>,
    active_actor: Single<(Entity, &Team), With<ActingActor>>,
    actor_q: Query<(&Health, &Team)>,
    mut next_state: ResMut<NextState<CombatState>>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        commands.insert_resource(ActingActorAction(Action::Attack {
            target: choose_target(rng, queue, active_actor, actor_q),
        }));
        next_state.set(CombatState::PerformAction);
    }
}

fn special_move(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<CombatState>>,
    rng: ResMut<EventRng>,
    queue: ResMut<TurnOrder>,
    active_actor: Single<(Entity, &Team, &ActorName), With<ActingActor>>,
    actor_q: Query<(&Health, &Team)>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        commands.insert_resource(ActingActorAction(Action::SpecialAction {
            target: choose_special_target(rng, queue, active_actor, actor_q),
        }));
        next_state.set(CombatState::PerformAction);
    }
}

pub fn choose_target(
    mut rng: ResMut<EventRng>,
    queue: ResMut<TurnOrder>,
    active_actor: Single<(Entity, &Team), With<ActingActor>>,
    actor_q: Query<(&Health, &Team)>,
) -> Entity {
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

    targets[rng.random_range(0..targets.len())]
}

pub fn choose_special_target(
    mut rng: ResMut<EventRng>,
    queue: ResMut<TurnOrder>,
    active_actor: Single<(Entity, &Team, &ActorName), With<ActingActor>>,
    actor_q: Query<(&Health, &Team)>,
) -> Entity {
    let (_, team, name) = *active_actor;
    match name {
        ActorName::Priestess => {
            let mut players: Vec<(Entity, u32)> = queue
                .queue()
                .iter()
                .filter_map(|&entity| {
                    if let Ok((health, target_team)) = actor_q.get(entity) {
                        if *target_team == *team {
                            let current_health = health.current().map(|h| h.get()).unwrap_or(0);
                            Some((entity, current_health))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            players.sort_by(|a, b| a.1.cmp(&b.1));

            players[0].0
        }
        _ => {
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

            targets[rng.random_range(0..targets.len())]
        }
    }
}

fn exit_gameover(
    mut click: Trigger<Pointer<Click>>,
    mut update_appstate: ResMut<NextState<AppState>>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        update_appstate.set(AppState::Menu);
    }
}

fn exit_victory(
    mut click: Trigger<Pointer<Click>>,
    mut update_appstate: ResMut<NextState<AppState>>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        update_appstate.set(AppState::Menu);
    }
}
