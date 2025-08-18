use super::*;
use crate::embed_asset;
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

pub struct AttackOptionsPlugin;

impl Plugin for AttackOptionsPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/Basic-button.png");
        embed_asset!(app, "assets/sprites/Move Banner.png");
        embed_asset!(app, "assets/sprites/Special Move.png");
        embed_asset!(app, "assets/sprites/buttons.png");
        embed_asset!(app, "assets/sprites/Game Over.png");
    }
}

#[derive(Component)]
pub struct AttackMenu;

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

pub fn despawn_attack_menu(
    mut commands: Commands,
    mut menu_entity: Single<Entity, With<AttackMenu>>,
) {
    commands.entity(*menu_entity).despawn();
}

pub fn spawn_gameover_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                    flex_basis: Val::Px(54.0),
                    ..default()
                },
            ));

            builder
                .spawn((
                    ImageNode {
                        image: asset_server.load(BUTTON_IMAGE_PATH),
                        ..default()
                    },
                    Node {
                        top: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_basis: Val::Px(50.0),
                        ..default()
                    },
                    Button,
                ))
                .observe(exit_gameover);
        });
}

fn basic_attack(
    mut click: Trigger<Pointer<Click>>,
    mut next_state: ResMut<NextState<CombatState>>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        next_state.set(CombatState::ChooseAction);
        info!("WORKING");
    }
}

fn special_move(mut click: Trigger<Pointer<Click>>) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        info!("Special Move Successful!!!");
    }
}

fn exit_gameover(
    mut click: Trigger<Pointer<Click>>,
    //mut update_gamestate: ResMut<NextState<MenuState>>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        /*
            update_gamestate.set(MenuState::Main);
        */

        info!("exit_gameover working!!!");
    }
}
