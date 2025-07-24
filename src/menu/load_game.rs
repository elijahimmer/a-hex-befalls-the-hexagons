use super::{MenuState, update_scroll_position_event};
use crate::prelude::*;

use accesskit::{Node as Accessible, Role};

use bevy::input_focus::InputFocus;
use bevy::{a11y::AccessibilityNode, ecs::hierarchy::ChildSpawnerCommands, prelude::*};

pub struct MenuLoadGamePlugin;

impl Plugin for MenuLoadGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<LoadGameState>();
        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<LoadGameState>);
        app.add_systems(
            OnEnter(MenuState::LoadGame),
            (get_save_games, load_game_enter).chain(),
        )
        .add_systems(OnExit(MenuState::LoadGame), remove_resource::<SaveGames>)
        .add_systems(Update, escape_out.run_if(in_state(MenuState::LoadGame)));
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(MenuState = MenuState::LoadGame)]
#[states(scoped_entities)]
pub enum LoadGameState {
    #[default]
    Main,
}

#[derive(Resource)]
pub struct SaveGames(pub Vec<SaveGameInfo>);

fn get_save_games(mut commands: Commands, db: NonSend<Database>) {
    let games = SaveGameInfo::get_all(&db).unwrap();

    commands.insert_resource(SaveGames(games));
}

#[derive(Component)]
pub struct LoadGameButton(pub GameID);

fn escape_out(
    controls_state: Res<State<LoadGameState>>,
    mut input_focus: ResMut<InputFocus>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
        if let Some(_) = input_focus.0 {
            input_focus.clear();
            return;
        }

        use LoadGameState as L;
        match *controls_state.get() {
            L::Main => next_menu_state.set(MenuState::Main),
        }
    }
}

fn back_button_click(
    mut click: Trigger<Pointer<Click>>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    click.propagate(false);
    match click.button {
        PointerButton::Primary => {
            menu_state.set(MenuState::Main);
        }
        _ => {}
    }
}

fn load_game_enter(mut commands: Commands, style: Res<Style>, saves: Res<SaveGames>) {
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
            StateScoped(MenuState::LoadGame),
        ))
        .with_children(|builder| {
            if saves.0.len() == 0 {
                builder.spawn((
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),

                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        justify_self: JustifySelf::Center,

                        ..default()
                    },
                    children![(Text::new("No Save Games"), TextColor(style.title_color),)],
                ));
            } else {
                builder
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(85.0),
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),

                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        row_gap: Val::Px(10.0),

                        overflow: Overflow::scroll_y(),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .observe(update_scroll_position_event)
                    .with_children(|builder| {
                        saves
                            .0
                            .iter()
                            .cloned()
                            .for_each(|game| game_entry(builder, &style, game))
                    });
            }

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
                            children![(
                                Text::new("Back"),
                                button_text_style.clone(),
                                Pickable::IGNORE
                            )],
                        ))
                        .observe(back_button_click);
                });
        });
}

fn game_entry(builder: &mut ChildSpawnerCommands<'_>, style: &Style, game: SaveGameInfo) {
    builder
        .spawn((Node::default(), Pickable::IGNORE))
        .with_children(|builder| {
            builder
                .spawn((
                    Node {
                        width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Label,
                    AccessibilityNode(Accessible::new(Role::ListItem)),
                    Pickable::IGNORE,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new(game.id.to_string()),
                        TextColor(style.title_color),
                        style.font(33.0),
                        Pickable::IGNORE,
                    ));

                    builder.spawn((
                        Button,
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Px(150.0),
                            margin: UiRect::px(2.0, 2.0, 0.0, 0.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(style.button_color),
                        AccessibilityNode(Accessible::new(Role::ListItem)),
                        LoadGameButton(game.id),
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: true,
                        },
                        children![(Text::new("Load Game"), Pickable::IGNORE)],
                    ));
                    //.observe(prompt_on_click);
                });
        });
}
