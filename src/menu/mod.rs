//! TODO: Make the UI hexagon based.
//! TODO: Implement title screen and pausing separately.
mod controls;
mod new_game;

use crate::embed_asset;
use crate::prelude::*;
use bevy::input_focus::InputFocus;
use bevy::{input::mouse::MouseScrollUnit, prelude::*};
use controls::*;
use new_game::*;

const TITLE_IMAGE_PATH: &str = "embedded://assets/sprites/title.png";

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/title.png");
        app.add_sub_state::<MenuState>();
        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<MenuState>);
        app.add_plugins(MenuControlsPlugin)
            .add_plugins(MenuNewGamePlugin)
            .add_systems(Update, button_highlight.run_if(in_state(GameState::Menu)))
            .add_systems(Update, escape_out.run_if(in_state(GameState::Menu)))
            .add_systems(OnEnter(MenuState::Main), main_enter)
            .add_systems(OnEnter(MenuState::Settings), settings_enter)
            .add_systems(OnEnter(MenuState::Display), display_enter)
            .add_systems(OnEnter(MenuState::Sound), sound_enter);
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Menu)]
#[states(scoped_entities)]
pub enum MenuState {
    #[default]
    Main,
    Settings,
    Display,
    Sound,
    Controls,
    NewGame,
}

/// Specifies the action that should be taken the button it is on is clicked.
///
/// The node will need to be observed by `menu_button_action` for this to take effect.
#[derive(Component)]
enum MenuButtonAction {
    MainMenu,
    Settings,
    Controls,
    Display,
    Sound,
    NewGame,
    Quit,
}

/// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

fn escape_out(
    menu_state: Res<State<MenuState>>,
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<MenuState>>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
        if let Some(_) = input_focus.0 {
            input_focus.clear();
            return;
        }

        use MenuState as M;
        match *menu_state.get() {
            M::Main
                // they implement it themselves
                | M::NewGame
                |M::Controls=> {}


            M::Settings => next_state.set(MenuState::Main),
            M::Sound | M::Display => next_state.set(MenuState::Settings),
        }
    }
}

fn button_highlight(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
    style: Res<Style>,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => {
                style.pressed_button_color.into()
            }
            (Interaction::Hovered, Some(_)) => style.hovered_pressed_button_color.into(),
            (Interaction::Hovered, Option::None) => style.hovered_button_color.into(),
            (Interaction::None, Option::None) => style.button_color.into(),
        }
    }
}

fn menu_button_click(
    mut click: Trigger<Pointer<Click>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    target_query: Query<&MenuButtonAction>,
) {
    if click.button == PointerButton::Primary {
        let Ok(menu_button_action) = target_query.get(click.target()) else {
            return;
        };
        match menu_button_action {
            MenuButtonAction::Quit => {
                app_exit_events.write(AppExit::Success);
            }
            MenuButtonAction::NewGame => menu_state.set(MenuState::NewGame),
            MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
            MenuButtonAction::Controls => menu_state.set(MenuState::Controls),
            MenuButtonAction::Display => menu_state.set(MenuState::Display),
            MenuButtonAction::Sound => menu_state.set(MenuState::Sound),
            MenuButtonAction::MainMenu => menu_state.set(MenuState::Main),
        }
    }

    click.propagate(false);
}

fn main_enter(mut commands: Commands, style: Res<Style>, asset_server: Res<AssetServer>) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = style.font(33.0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(MenuState::Main),
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    // Display the game name
                    builder.spawn((
                        ImageNode {
                            image: asset_server.load(TITLE_IMAGE_PATH),
                            ..default()
                        },
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                    // Display three buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            MenuButtonAction::NewGame,
                            children![
                                //(ImageNode::new(right_icon), button_icon_node.clone()),
                                (
                                    Text::new("New Game"),
                                    button_text_font.clone(),
                                    TextColor(style.text_color),
                                    Pickable::IGNORE
                                ),
                            ],
                        ))
                        .observe(menu_button_click);
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            MenuButtonAction::Settings,
                            children![
                                //(ImageNode::new(wrench_icon), button_icon_node.clone()),
                                (
                                    Text::new("Settings"),
                                    button_text_font.clone(),
                                    TextColor(style.text_color),
                                    Pickable::IGNORE
                                ),
                            ],
                        ))
                        .observe(menu_button_click);
                    builder
                        .spawn((
                            Button,
                            button_node,
                            BackgroundColor(style.button_color),
                            MenuButtonAction::Quit,
                            children![
                                //(ImageNode::new(exit_icon), button_icon_node),
                                (
                                    Text::new("Quit"),
                                    button_text_font,
                                    TextColor(style.text_color),
                                    Pickable::IGNORE
                                ),
                            ],
                        ))
                        .observe(menu_button_click);
                });
        });
}

fn settings_enter(mut commands: Commands, style: Res<Style>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (style.font(33.0), TextColor(style.text_color));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(MenuState::Settings),
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    [
                        (MenuButtonAction::Controls, "Controls"),
                        (MenuButtonAction::Display, "Display"),
                        (MenuButtonAction::Sound, "Sound"),
                        (MenuButtonAction::MainMenu, "Back"),
                    ]
                    .into_iter()
                    .for_each(|(action, text)| {
                        builder
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(style.button_color),
                                action,
                                children![(
                                    Text::new(text),
                                    button_text_style.clone(),
                                    Pickable::IGNORE
                                )],
                            ))
                            .observe(menu_button_click);
                    });
                });
        });
}

fn display_enter(mut commands: Commands, style: Res<Style>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (style.font(33.0), TextColor(style.text_color));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(MenuState::Display),
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            MenuButtonAction::Settings,
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(menu_button_click);
                });
        });
}

fn sound_enter(mut commands: Commands, style: Res<Style> /*volume: Res<Volume>*/) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        style.font(33.0),
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(TEXT_COLOR),
    );

    //let button_node_clone = button_node.clone();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSoundScreen,
        ))
        .with_children(|builder| {
            builder
                .spawn(
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                )
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Settings,
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(menu_button_click);
                });
        });
}

#[derive(Component, Clone, Debug)]
enum ControlsButtonAction {
    Prompt(Control, usize),
    PromptCancel,
    ResetBoth(Control),
    ResetAll,
    Save,
    Discard,
    Back,
}

fn controls_enter(mut commands: Commands, style: Res<Style>, controls: Res<Controls>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        style.font(33.0),
        TextColor(style.text_color),
    );

    //let button_node_clone = button_node.clone();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(MenuState::Sound),
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            MenuButtonAction::Settings,
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(menu_button_click);
                });
        });
}

const LINE_HEIGHT: f32 = 65.0;

pub fn update_scroll_position_event(
    mut trigger: Trigger<Pointer<Scroll>>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    let mut target = scrolled_node_query
        .get_mut(trigger.target)
        .expect("Cannot scroll a non-scrollable entity");

    let event = trigger.event();
    let dy = match event.unit {
        MouseScrollUnit::Line => event.y * LINE_HEIGHT,
        MouseScrollUnit::Pixel => event.y,
    };

    target.offset_y -= dy;

    trigger.propagate(false);
}
