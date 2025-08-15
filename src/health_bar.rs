use crate::embed_asset;
use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const NORMAL_TICK_SPEED: usize = 1;

pub struct HpPlugin;

pub const HP_SPRITE_IMAGE_PATH: &str = "embedded://assets/sprites/HP-Sprite.png";
pub const HP_BAR_IMAGE_PATH: &str = "embedded://assets/sprites/HP-Bar.png";
pub const PRIESTESS_IMAGE_PATH: &str = "embedded://assets/sprites/Priestess_name.png";
pub const THIEF_IMAGE_PATH: &str = "embedded://assets/sprites/thief_name.png";
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
        app.add_systems(OnEnter(AppState::Game), create_hp_bars);
    }
}

fn create_hp_bars(mut commands: Commands, style: Res<Style>, asset_server: Res<AssetServer>) {
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
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                    
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(PRIESTESS_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(100.0),
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(THIEF_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            flex_grow: STANDARD_FLEX_GROW,
                            flex_basis: Val::Px(100.0),
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                });
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Start,
                    ..default()
                })
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

fn update_hp_bars() {}
