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
        .add_systems(OnEnter(LoadGameState::Prompt), prompt_enter)
        .add_systems(
            OnEnter(LoadGameState::Main),
            remove_resource::<PromptTarget>,
        )
        .add_systems(Update, escape_out.run_if(in_state(MenuState::LoadGame)))
        .add_systems(
            OnEnter(LoadGameState::Loading),
            (prep_loading, crate::saving::load_game).chain(),
        );
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(MenuState = MenuState::LoadGame)]
#[states(scoped_entities)]
pub enum LoadGameState {
    #[default]
    Main,
    Prompt,
    Loading,
}

#[derive(Resource)]
pub struct SaveGames(pub Box<[SaveGameInfo]>);

#[derive(Resource)]
struct PromptTarget(pub GameID);

fn get_save_games(mut commands: Commands, db: NonSend<Database>) {
    let games = SaveGameInfo::get_all(&db).unwrap();

    commands.insert_resource(SaveGames(games));
}

#[derive(Component)]
pub struct LoadGameButton(pub GameID);

fn escape_out(
    controls_state: Res<State<LoadGameState>>,
    mut input_focus: ResMut<InputFocus>,
    mut next_load_game_state: ResMut<NextState<LoadGameState>>,
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
            L::Prompt | L::Loading => next_load_game_state.set(LoadGameState::Main),
        }
    }
}

fn prompt_on_click(
    mut click: Trigger<Pointer<Click>>,
    prompt: Query<&LoadGameButton>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<LoadGameState>>,
) {
    click.propagate(false);

    let Ok(LoadGameButton(game_id)) = prompt.get(click.target()) else {
        return;
    };

    match click.button {
        PointerButton::Primary => {
            commands.insert_resource(PromptTarget(*game_id));
            next_state.set(LoadGameState::Prompt);
        }
        PointerButton::Secondary | PointerButton::Middle => {}
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
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            MenuState::Main,
                        ));
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
                        min_height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::px(20.0, 20.0, 5.0, 5.0),
                        ..default()
                    },
                    Label,
                    AccessibilityNode(Accessible::new(Role::ListItem)),
                    BackgroundColor(style.button_color),
                    Button,
                    LoadGameButton(game.id),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                ))
                .observe(prompt_on_click)
                .with_children(|builder| {
                    builder
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                height: Val::Percent(100.0),
                                margin: UiRect::px(2.0, 2.0, 0.0, 0.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text::new(format!("game: {}", game.id.to_string())),
                                style.font(33.0),
                                Pickable::IGNORE,
                            ));

                            builder.spawn((
                                Text::new(format!(
                                    "created: {}",
                                    game.created.format("%Y/%m/%d %H:%M")
                                )),
                                style.font(24.0),
                                Pickable::IGNORE,
                            ));

                            builder.spawn((
                                Text::new(format!(
                                    "last saved: {}",
                                    game.last_saved.format("%Y/%m/%d %H:%M")
                                )),
                                style.font(24.0),
                                Pickable::IGNORE,
                            ));

                            builder.spawn((
                                Text::new(format!("seed: {:X}", game.world_seed)),
                                style.font(24.0),
                                Pickable::IGNORE,
                            ));
                        });
                });
        });
}

fn prompt_enter(mut commands: Commands, style: Res<Style>) {
    let button_text_style = (
        style.font(33.0),
        TextColor(style.text_color),
        TextLayout::new_with_justify(JustifyText::Center),
    );

    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            StateScoped(LoadGameState::Prompt),
            BackgroundColor(style.background_color),
            ZIndex(2),
        ))
        .with_children(|builder| {
            builder
                .spawn((Node {
                    display: Display::Flex,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|builder| {
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(65.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                align_self: AlignSelf::Center,
                                ..default()
                            },
                            BackgroundColor(style.button_color),
                            children![(Text::new("Load Game"), button_text_style.clone())],
                        ))
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            LoadGameState::Loading,
                        ));
                    builder
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(65.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                align_self: AlignSelf::Center,
                                ..default()
                            },
                            BackgroundColor(style.button_color),
                            children![(Text::new("Cancel"), button_text_style.clone())],
                        ))
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            LoadGameState::Main,
                        ));
                });
        });
}

fn prep_loading(mut commands: Commands, db: NonSend<Database>, target: Res<PromptTarget>) {
    commands.insert_resource(SaveGame::load(&db, target.0));
}
