use super::MenuState;
use crate::prelude::*;

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy_ui_text_input::{TextInputFilter, TextInputMode, TextInputNode};

pub struct MenuNewGamePlugin;
impl Plugin for MenuNewGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<NewGameState>();
        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<NewGameState>);
        app.add_systems(OnEnter(NewGameState::Main), new_game_enter)
            .add_systems(
                OnEnter(NewGameState::GeneratingWorld),
                (generating_world_enter, set_camera_scale),
            )
            .add_systems(OnExit(NewGameState::GeneratingWorld), reset_camera_scale)
            .add_systems(Update, escape_out.run_if(in_state(MenuState::NewGame)));
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(MenuState = MenuState::NewGame)]
#[states(scoped_entities)]
pub enum NewGameState {
    #[default]
    Main,
    GeneratingWorld,
}

#[derive(Component, Clone, Debug)]
pub enum NewGameButtonAction {
    ToMenu,
    GenerateWorld,
    CancelGeneration,
}

#[derive(Component)]
pub struct WorldNameTextBox;

#[derive(Component)]
pub struct WorldSeedTextBox;

fn set_camera_scale(
    camera_id: Res<MainCamera>,
    mut projection: Query<&mut Projection, With<MainCameraMarker>>,
) {
    let projection = &mut projection
        .get_mut(camera_id.0)
        .expect("There should be a camera!");

    let Projection::Orthographic(ref mut projection2d) = **projection else {
        unreachable!("Only Orthographic Projection is supported!");
    };

    projection2d.scale = CAMERA_DEFAULT_SCALE * 4.0;
}

fn reset_camera_scale(
    camera_id: Res<MainCamera>,
    mut projection: Query<&mut Projection, With<MainCameraMarker>>,
) {
    let projection = &mut projection
        .get_mut(camera_id.0)
        .expect("There should be a camera!");

    let Projection::Orthographic(ref mut projection2d) = **projection else {
        unreachable!("Only Orthographic Projection is supported!");
    };

    projection2d.scale = CAMERA_DEFAULT_SCALE;
}

fn escape_out(
    new_game_state: Res<State<NewGameState>>,
    mut input_focus: ResMut<InputFocus>,
    mut next_new_game_state: ResMut<NextState<NewGameState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
        if let Some(_) = input_focus.0 {
            input_focus.clear();
            return;
        }

        use NewGameState as S;
        match *new_game_state.get() {
            S::Main => {
                next_menu_state.set(MenuState::Main);
            }
            S::GeneratingWorld => {
                next_new_game_state.set(NewGameState::Main);
            }
        }
    }
}

fn new_game_menu_click(
    mut click: Trigger<Pointer<Click>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut next_new_game_state: ResMut<NextState<NewGameState>>,
    target_query: Query<&NewGameButtonAction>,
) {
    if let Ok(action) = target_query.get(click.target()) {
        use NewGameButtonAction as A;
        use PointerButton as P;
        match (click.button, action) {
            (P::Primary, A::ToMenu) => {
                next_menu_state.set(MenuState::Main);
            }
            (_, A::ToMenu) => {}

            (P::Primary, A::GenerateWorld) => {
                next_new_game_state.set(NewGameState::GeneratingWorld);
            }
            (_, A::GenerateWorld) => {}

            (P::Primary, A::CancelGeneration) => {
                next_new_game_state.set(NewGameState::Main);
            }
            (_, A::CancelGeneration) => {}
        }
    }

    click.propagate(false);
}

fn new_game_enter(mut commands: Commands, style: Res<Style>) {
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
        TextLayout::new_with_justify(JustifyText::Center),
    );

    //let button_node_clone = button_node.clone();
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(NewGameState::Main),
        ))
        .observe(clear_focus_on_click)
        .with_children(|builder| {
            builder
                .spawn(Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|builder| {
                    builder.spawn((button_text_style.clone(), Text::new("Save Name:")));

                    builder
                        .spawn((
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(60.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                padding: UiRect::all(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(style.background_color.with_alpha(1.0)),
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                WorldNameTextBox,
                                TextInputNode {
                                    clear_on_submit: false,
                                    mode: TextInputMode::SingleLine,
                                    focus_on_pointer_down: true,
                                    unfocus_on_submit: true,
                                    max_chars: Some(64),
                                    ..default()
                                },
                                button_text_style.clone(),
                            ));
                        })
                        .observe(stop_event_propagate::<Pointer<Click>>);

                    builder.spawn((button_text_style.clone(), Text::new("Seed:")));

                    builder
                        .spawn((
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(60.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                margin: UiRect::all(Val::Px(10.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(style.background_color.with_alpha(1.0)),
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                WorldSeedTextBox,
                                TextInputNode {
                                    clear_on_submit: false,
                                    mode: TextInputMode::SingleLine,
                                    focus_on_pointer_down: true,
                                    unfocus_on_submit: true,
                                    max_chars: Some(16),
                                    filter: Some(TextInputFilter::Hex),
                                    ..default()
                                },
                                button_text_style.clone(),
                            ));
                        })
                        .observe(stop_event_propagate::<Pointer<Click>>);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            NewGameButtonAction::GenerateWorld,
                            children![(
                                Text::new("Generate World"),
                                button_text_style.clone(),
                                Pickable::IGNORE
                            )],
                        ))
                        .observe(new_game_menu_click);
                });

            builder
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        align_self: AlignSelf::End,
                        ..default()
                    },
                    BackgroundColor(style.background_color),
                ))
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            NewGameButtonAction::ToMenu,
                            children![(
                                Text::new("Back"),
                                button_text_style.clone(),
                                Pickable::IGNORE
                            )],
                        ))
                        .observe(new_game_menu_click);
                });
        });
}

fn generating_world_enter(mut commands: Commands, style: Res<Style>) {
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
        TextLayout::new_with_justify(JustifyText::Center),
    );

    //let button_node_clone = button_node.clone();
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(NewGameState::GeneratingWorld),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        align_self: AlignSelf::End,
                        ..default()
                    },
                    BackgroundColor(style.background_color),
                ))
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            NewGameButtonAction::CancelGeneration,
                            children![(
                                Text::new("Cancel"),
                                button_text_style.clone(),
                                Pickable::IGNORE
                            )],
                        ))
                        .observe(new_game_menu_click);
                });
        });
}
