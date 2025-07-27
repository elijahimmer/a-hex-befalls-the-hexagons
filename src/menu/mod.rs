//! TODO: Make the UI hexagon based.
//! TODO: Implement title screen and pausing separately.

pub mod controls;
#[cfg(feature = "sqlite")]
pub mod load_game;
pub mod new_game;

use crate::embed_asset;
use crate::prelude::*;
use bevy::input_focus::InputFocus;
use bevy::{input::mouse::MouseScrollUnit, prelude::*};
use controls::*;
#[cfg(feature = "sqlite")]
use load_game::*;
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
            .add_plugins(MenuNewGamePlugin);

        #[cfg(feature = "sqlite")]
        app.add_plugins(MenuLoadGamePlugin);

        app.add_systems(
            Update,
            (button_highlight, escape_out).run_if(in_state(AppState::Menu)),
        )
        .add_systems(OnEnter(MenuState::Main), main_enter)
        .add_systems(OnEnter(MenuState::Settings), settings_enter)
        .add_systems(OnEnter(MenuState::Display), display_enter)
        .add_systems(OnEnter(MenuState::Sound), sound_enter);
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::Menu)]
#[states(scoped_entities)]
pub enum MenuState {
    #[default]
    Main,
    Settings,
    Display,
    Sound,
    Controls,
    NewGame,
    #[cfg(feature = "sqlite")]
    LoadGame,
}

/// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

/// Whenever the player hits the pause button, it should
/// put them out as if they hit the back button.
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
                | M::Controls => {}
            #[cfg(feature = "sqlite")]
            M::LoadGame => {}

            M::Settings => next_state.set(MenuState::Main),
            M::Sound | M::Display => next_state.set(MenuState::Settings),
        }
    }
}

/// Highlight the buttons on hover to make them look better.
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

/// The action to preform when a button is clicked with a `MenuButtonAction`
fn quit_game_on_click(
    mut click: Trigger<Pointer<Click>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    click.propagate(false);

    if click.button == PointerButton::Primary {
        app_exit_events.write(AppExit::Success);
    }
}

fn main_enter(mut commands: Commands, style: Res<Style>, asset_server: Res<AssetServer>) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(15.0)),
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
                    [
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::NewGame),
                            "New Game",
                        ),
                        #[cfg(feature = "sqlite")]
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::LoadGame),
                            "Load Game",
                        ),
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::Settings),
                            "Settings",
                        ),
                    ]
                    .into_iter()
                    .for_each(|(action, text)| {
                        builder
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(style.button_color),
                                children![(
                                    Text::new(text),
                                    button_text_font.clone(),
                                    TextColor(style.text_color),
                                    Pickable::IGNORE
                                ),],
                            ))
                            .observe(action);
                    });

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            children![(
                                Text::new("Quit"),
                                button_text_font.clone(),
                                TextColor(style.text_color),
                                Pickable::IGNORE
                            ),],
                        ))
                        .observe(quit_game_on_click);
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
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::Controls),
                            "Controls",
                        ),
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::Display),
                            "Display",
                        ),
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::Sound),
                            "Sound",
                        ),
                        (
                            change_state_on_click(PointerButton::Primary, MenuState::Main),
                            "Back",
                        ),
                    ]
                    .into_iter()
                    .for_each(|(action, text)| {
                        builder
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(style.button_color),
                                children![(
                                    Text::new(text),
                                    button_text_style.clone(),
                                    Pickable::IGNORE
                                )],
                            ))
                            .observe(action);
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
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            MenuState::Settings,
                        ));
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
                            children![(Text::new("Back"), button_text_style.clone())],
                        ))
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            MenuState::Settings,
                        ));
                });
        });
}

const LINE_HEIGHT: f32 = 65.0;

/// Update the scroll position of the hovered node
/// when scrolled
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
