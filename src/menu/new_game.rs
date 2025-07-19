use super::MenuState;
use crate::prelude::*;

use bevy::{prelude::*, ui::FocusPolicy};

pub struct MenuNewGamePlugin;

impl Plugin for MenuNewGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<NewGameState>()
            .add_systems(Update, log_transitions::<NewGameState>)
            .add_systems(OnEnter(NewGameState::Main), new_game_enter)
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
    Back,
    GenerateWorld,
}

fn escape_out(
    new_game_state: Res<State<NewGameState>>,
    mut next_new_game_state: ResMut<NextState<NewGameState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
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
    mut commands: Commands,
    target_query: Query<&NewGameButtonAction>,
) {
    if let Ok(action) = target_query.get(click.target()) {
        use NewGameButtonAction as A;
        use PointerButton as P;
        match (click.button, action) {
            (P::Primary, A::Back) => {
                commands.set_state(MenuState::Main);
            }
            (_, A::Back) => {}

            (P::Primary, A::GenerateWorld) => {
                commands.set_state(NewGameState::GeneratingWorld);
            }
            (_, A::GenerateWorld) => {}
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
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Center,
                ..default()
            },
            StateScoped(MenuState::NewGame),
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
                    FocusPolicy::Block,
                    BackgroundColor(style.background_color),
                ))
                .with_children(|builder| {
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

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            NewGameButtonAction::Back,
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
