use crate::embed_asset;
use crate::game::*;
use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const NORMAL_TICK_SPEED: usize = 1;

pub struct HpPlugin;

pub const HP_SPRITE_IMAGE_PATH: &str = "embedded://assets/sprites/HP-Sprite.png";
pub const HP_BAR_IMAGE_PATH: &str = "embedded://assets/sprites/HP-Bar.png";
pub const PRIESTESS_IMAGE_PATH: &str = "embedded://assets/sprites/Priestess_name.png";
pub const THIEF_IMAGE_PATH: &str = "embedded://assets/sprites/Thief_name.png";
pub const WARRIOR_IMAGE_PATH: &str = "embedded://assets/sprites/Warrior_name.png";

pub const FONT_SIZE: f32 = 18.0;
pub const STANDARD_FLEX_GROW: f32 = 1.75;

impl Plugin for HpPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/HP-Sprite.png");
        embed_asset!(app, "assets/sprites/HP-Bar.png");
        embed_asset!(app, "assets/sprites/Priestess_name.png");
        embed_asset!(app, "assets/sprites/Thief_name.png");
        embed_asset!(app, "assets/sprites/Warrior_name.png");
        app.add_systems(OnEnter(AppState::Game), (create_hp_bars, spawn_hp).chain());
    }
}

#[derive(Component)]
pub struct HPBar;

fn create_hp_bars(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Left HP
    commands
        .spawn((Node {
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    align_items: AlignItems::Start,
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|builder| {
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(WARRIOR_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            top: Val::Px(20.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(100.0),
                            ..default()
                        },
                    ));

                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(PRIESTESS_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            top: Val::Px(20.0),
                            flex_grow: STANDARD_FLEX_GROW + 1.0,
                            flex_basis: Val::Px(120.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(THIEF_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            top: Val::Px(20.0),
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(80.0),
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                });
            builder
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(HP_SPRITE_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(100.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(HP_SPRITE_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(100.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(HP_SPRITE_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(100.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                });
        });
}

fn spawn_hp(
    mut commands: Commands,
    mut actors_health_q: Query<&Health, With<Actor>>,
    asset_server: Res<AssetServer>,
) {
    let mut actors_health: Vec<&Health> = Vec::new();

    for health in actors_health_q {
        actors_health.push(health);
    }

    commands.spawn((
        Node {
            top: Val::Px(67.5),
            left: Val::Px(56.5),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..default()
        },
        HPBar,
        ActorName::Warrior,
        Text::new(format!(
            "{}/{}",
            actors_health.get(0).unwrap().current().unwrap(),
            actors_health.get(0).unwrap().max()
        )),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
    ));
    commands.spawn((
        Node {
            top: Val::Px(67.5),
            left: Val::Px(177.5),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..default()
        },
        HPBar,
        ActorName::Priestess,
        Text::new(format!(
            "{}/{}",
            actors_health.get(1).unwrap().current().unwrap(),
            actors_health.get(1).unwrap().max()
        )),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
    ));

    commands.spawn((
        Node {
            top: Val::Px(67.5),
            left: Val::Px(297.5),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..default()
        },
        HPBar,
        ActorName::Theif,
        Text::new(format!(
            "{}/{}",
            actors_health.get(2).unwrap().current().unwrap(),
            actors_health.get(2).unwrap().max()
        )),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
    ));
}

pub fn update_player_hp_bar(
    mut commands: Commands,
    active_actor_team: Single<&Team, With<ActingActor>>,
    active_actor_name: Single<&ActorName, With<ActingActor>>,
    mut actor_q: Query<(&ActorName, &Health), With<Actor>>,
    mut text_q: Query<(Entity, &ActorName), With<HPBar>>,
    actor_action: Res<ActingActorAction>,
) {
    match *active_actor_team {
        Team::Enemy => match **actor_action {
            Action::Attack { target } => {
                if let Ok((actor_name, target_health)) = actor_q.get(target) {
                    let mut health_str: String = format!("");
                    if let Some(current_health) = target_health.current() {
                        health_str = format!("{}/{}", current_health, target_health.max());
                    } else {
                        health_str = format!("0/{}", target_health.max());
                    }

                    for (text_entity, text_actorname) in text_q {
                        if text_actorname == actor_name {
                            commands
                                .entity(text_entity)
                                .remove::<(Text, TextFont, TextLayout)>()
                                .insert((
                                    Text::new(health_str),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextLayout::new_with_justify(JustifyText::Left),
                                ));
                            break;
                        }
                    }
                }
            }
            _ => {}
        },
        Team::Player => match **actor_action {
            Action::SpecialAction { target } => match *active_actor_name {
                ActorName::Priestess => {
                    if let Ok((actor_name, target_health)) = actor_q.get(target) {
                        let mut health_str: String = format!("");
                        if let Some(current_health) = target_health.current() {
                            health_str = format!("{}/{}", current_health, target_health.max());
                        } else {
                            health_str = format!("0/{}", target_health.max());
                        }

                        for (text_entity, text_actorname) in text_q {
                            if text_actorname == actor_name {
                                commands
                                    .entity(text_entity)
                                    .remove::<(Text, TextFont, TextLayout)>()
                                    .insert((
                                        Text::new(health_str),
                                        TextFont {
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextLayout::new_with_justify(JustifyText::Left),
                                    ));
                                break;
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        },
    }
}

pub fn update_player_hp_bar_pit(
    mut commands: Commands,
    mut actor_q: Query<(&ActorName, &Health), With<Actor>>,
    mut text_q: Query<(Entity, &ActorName), With<HPBar>>,
) {
    for (actor_name, health) in actor_q {
        let mut health_str: String = format!("");
        if let Some(current_health) = health.current() {
            health_str = format!("{}/{}", current_health, health.max());
        } else {
            health_str = format!("0/{}", health.max());
        }

        for (entity, text_actor_name) in text_q {
            if actor_name == text_actor_name {
                commands
                    .entity(entity)
                    .remove::<(Text, TextFont, TextLayout)>()
                    .insert((
                        Text::new(health_str),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextLayout::new_with_justify(JustifyText::Left),
                    ));
                break;
            }
        }
    }
}
