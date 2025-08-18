use super::MenuState;
use crate::generate_map::GenerationSettings;
use crate::prelude::*;
use crate::room::CurrentRoom;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy_ui_text_input::{TextInputContents, TextInputFilter, TextInputMode, TextInputNode};

pub struct MenuNewGamePlugin;
impl Plugin for MenuNewGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<NewGameState>();
        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<NewGameState>);
        app.add_systems(OnEnter(NewGameState::Main), new_game_enter)
            .add_systems(
                OnEnter(NewGameState::GeneratingWorld),
                (generating_world_enter, setup_party),
            )
            .add_systems(Update, escape_out.run_if(in_state(MenuState::NewGame)))
            .add_systems(
                Update,
                progress_check.run_if(in_state(NewGameState::GeneratingWorld)),
            );
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

#[derive(Resource, Default)]
pub struct GenerationProgress {
    pub world_done: bool,
    pub characters_done: bool,
}

impl GenerationProgress {
    pub fn done(&self) -> bool {
        self.world_done & self.characters_done
    }
}

#[derive(Component)]
pub struct WorldNameTextBox;

#[derive(Component)]
pub struct WorldSeedTextBox;

fn progress_check(
    mut commands: Commands,
    progress: Res<GenerationProgress>,
    mut next_state: ResMut<NextState<AppState>>,
    info_q: Query<(Entity, &RoomInfo)>,
) {
    if progress.done() {
        let current_room = info_q
            .iter()
            .find_map(
                |(
                    entity,
                    RoomInfo {
                        r_type: room_type, ..
                    },
                )| (matches!(room_type, RoomType::Entrance)).then_some(entity),
            )
            .unwrap();

        commands.entity(current_room).insert(CurrentRoom);

        #[cfg(feature = "sqlite")]
        commands.run_system_cached(crate::saving::save_game);
        next_state.set(AppState::Game);
    }
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

fn cancel_generation(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    room_map_q: Query<Entity, With<RoomTilemap>>,
    party: Query<Entity, With<Actor>>,
    mut next_new_game_state: ResMut<NextState<NewGameState>>,
) {
    let PointerButton::Primary = click.button else {
        return;
    };

    next_new_game_state.set(NewGameState::Main);

    commands.remove_resource::<GenerationProgress>();
    #[cfg(feature = "sqlite")]
    commands.remove_resource::<SaveGame>();
    commands.remove_resource::<GenerationSettings>();

    for entity in room_map_q.iter() {
        commands.entity(entity).despawn();
    }

    for entity in party.iter() {
        commands.entity(entity).despawn();
    }

    click.propagate(false);
}

fn generate_world_click(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut next_new_game_state: ResMut<NextState<NewGameState>>,
    db: NonSend<Database>,
    contents_query: Query<&TextInputContents, With<WorldSeedTextBox>>,
) {
    let PointerButton::Primary = click.button else {
        return;
    };

    let seed = contents_query
        .single()
        .inspect_err(|e| warn!("Failed to get seed from textbox with {e}"))
        .ok()
        .and_then(|seed| {
            u64::from_str_radix(seed.get(), 16)
                .inspect_err(|e| warn!("Failed to parse seed from textbox with {e}"))
                .ok()
        })
        .unwrap_or_else(|| getrandom::u64().unwrap_or(0x5eed_f0e_feee));

    commands.insert_resource(GenerationProgress::default());
    #[cfg(feature = "sqlite")]
    commands.insert_resource(SaveGame::new(&db, seed));
    commands.insert_resource(GenerationSettings { seed: seed });

    next_new_game_state.set(NewGameState::GeneratingWorld);

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
                                TextInputContents::default(),
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
                            children![(
                                Text::new("Generate World"),
                                button_text_style.clone(),
                                Pickable::IGNORE
                            )],
                        ))
                        .observe(generate_world_click);
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
                            children![(
                                Text::new("Cancel"),
                                button_text_style.clone(),
                                Pickable::IGNORE
                            )],
                        ))
                        .observe(cancel_generation);
                });
        });
}

fn setup_party(
    mut commands: Commands,
    mut progress: ResMut<GenerationProgress>,
    asset_server: Res<AssetServer>,
) {
    use ActorName as A;
    for name in [A::Warrior, A::Priestess, A::Theif] {
        commands.spawn((
            ActorBundle::from_name(&asset_server, name, Team::Player, Transform::IDENTITY, true),
            Visibility::Hidden,
        ));
    }

    commands.init_resource::<Items>();
    progress.characters_done = true;
}
