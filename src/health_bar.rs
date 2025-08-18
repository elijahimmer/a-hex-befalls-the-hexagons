use crate::embed_asset;
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
        app.add_systems(
            OnEnter(AppState::Game),
            (create_hp_bars, spawn_hp, update_hp_bar).chain(),
        );
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

    commands
        .spawn((
            Node {
                top: Val::Px(57.5),
                left: Val::Px(46.5),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            HPBar,
        ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
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
        });
    commands
        .spawn((
            Node {
                top: Val::Px(57.5),
                left: Val::Px(167.5),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            HPBar,
        ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
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
        });

    commands
        .spawn((
            Node {
                top: Val::Px(57.5),
                left: Val::Px(287.5),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            HPBar,
        ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
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
        });
}

fn update_hp_bar(commands: Commands, hp_q: Query<&Text, With<HPBar>>) {}
