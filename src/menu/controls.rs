use super::{MenuState, update_scroll_position_event};
use crate::prelude::*;

use accesskit::{Node as Accessible, Role};

use bevy::input_focus::InputFocus;
use bevy::{
    a11y::AccessibilityNode,
    ecs::hierarchy::ChildSpawnerCommands,
    input::{
        ButtonState, gamepad::GamepadButtonChangedEvent, keyboard::KeyboardInput,
        mouse::MouseButtonInput,
    },
    picking::hover::HoverMap,
    prelude::*,
};

use crate::controls::Control;
use crate::controls::{Input, Keybind, input_to_screen};

pub struct MenuControlsPlugin;

impl Plugin for MenuControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ControlsState>();
        #[cfg(feature = "debug")]
        app.add_systems(Update, log_transitions::<ControlsState>);
        app.add_systems(
            OnEnter(MenuState::Controls),
            (controls_enter, init_resource::<ControlsWIP>),
        )
        .add_systems(OnExit(MenuState::Controls), remove_resource::<ControlsWIP>)
        .add_systems(
            Update,
            (
                controls_changed.run_if(resource_exists_and_changed::<ControlsWIP>),
                escape_out,
            )
                .run_if(in_state(MenuState::Controls)),
        )
        .add_systems(OnEnter(ControlsState::Prompt), control_prompt_enter)
        .add_systems(
            OnExit(ControlsState::Prompt),
            remove_resource::<PromptTarget>,
        )
        .add_systems(
            Update,
            assign_key_input.run_if(in_state(ControlsState::Prompt)),
        )
        .add_systems(
            OnEnter(ControlsState::SaveWarning),
            control_save_warning_enter,
        );
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(MenuState = MenuState::Controls)]
#[states(scoped_entities)]
pub enum ControlsState {
    #[default]
    Main,
    Prompt,
    SaveWarning,
}

#[derive(Resource)]
struct PromptTarget(Control, usize);

/// Must be set when entering this menu.
/// Must be unset when leaving.
/// This is used to store the shown controls,
/// and is synced to the real controls on save,
/// or ignored on discard
#[derive(Resource)]
pub struct ControlsWIP(pub Controls);

impl FromWorld for ControlsWIP {
    fn from_world(world: &mut World) -> Self {
        Self(
            world
                .get_resource::<Controls>()
                .expect("There should be controls by now!")
                .clone(),
        )
    }
}

#[derive(Component)]
pub struct CancelPromptButton;

#[derive(Component)]
pub struct PromptButton(pub Control, pub usize);

fn prompt_on_click(
    mut click: Trigger<Pointer<Click>>,
    prompt: Query<&PromptButton>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ControlsState>>,
    mut controls_wip: ResMut<ControlsWIP>,
) {
    click.propagate(false);

    let Ok(PromptButton(control, entry)) = prompt.get(click.target()) else {
        return;
    };

    match click.button {
        PointerButton::Primary => {
            commands.insert_resource(PromptTarget(*control, *entry));
            next_state.set(ControlsState::Prompt);
        }
        PointerButton::Secondary => {
            controls_wip.0.set_control(*control, *entry, None);
        }
        PointerButton::Middle => {
            controls_wip.0.reset_control_part(*control, *entry);
        }
    }
}

fn reset_control_on_click(
    control: Control,
) -> impl Fn(Trigger<Pointer<Click>>, ResMut<ControlsWIP>) {
    move |mut click, mut controls_wip| {
        click.propagate(false);
        match click.button {
            PointerButton::Primary => controls_wip.0.reset_control(control),
            _ => {}
        }
    }
}

fn reset_controls_on_click(
    mut click: Trigger<Pointer<Click>>,
    mut controls_wip: ResMut<ControlsWIP>,
) {
    click.propagate(false);
    match click.button {
        PointerButton::Primary => controls_wip.0.reset_controls(),
        _ => {}
    }
}

fn save_changes_on_click(
    mut click: Trigger<Pointer<Click>>,
    mut controls_master: ResMut<Controls>,
    controls_wip: Res<ControlsWIP>,
) {
    click.propagate(false);
    match click.button {
        PointerButton::Primary => *controls_master = controls_wip.0.clone(),
        _ => {}
    }
}

fn discard_changes_on_click(
    mut click: Trigger<Pointer<Click>>,
    controls_master: Res<Controls>,
    mut controls_wip: ResMut<ControlsWIP>,
) {
    click.propagate(false);
    match click.button {
        PointerButton::Primary => controls_wip.0 = controls_master.clone(),
        _ => {}
    }
}

fn back_button_click(
    mut click: Trigger<Pointer<Click>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut controls_state: ResMut<NextState<ControlsState>>,
    controls_master: Res<Controls>,
    controls_wip: Res<ControlsWIP>,
) {
    click.propagate(false);
    match click.button {
        PointerButton::Primary => {
            if controls_wip.0 == *controls_master {
                menu_state.set(MenuState::Settings);
            } else {
                controls_state.set(ControlsState::SaveWarning);
            }
        }
        _ => {}
    }
}

fn escape_out(
    controls_state: Res<State<ControlsState>>,
    mut input_focus: ResMut<InputFocus>,
    mut next_controls_state: ResMut<NextState<ControlsState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    controls_master: Res<Controls>,
    controls_wip: Res<ControlsWIP>,
    key: Res<ControlState>,
) {
    if key.just_pressed(Control::Pause) {
        if let Some(_) = input_focus.0 {
            input_focus.clear();
            return;
        }

        use ControlsState as C;
        match *controls_state.get() {
            C::Prompt => {
                // ignore, the prompt handles the input.
            }
            C::SaveWarning => {
                next_menu_state.set(MenuState::Settings);
            }
            C::Main => {
                if controls_wip.0 == *controls_master {
                    next_menu_state.set(MenuState::Settings);
                } else {
                    next_controls_state.set(ControlsState::SaveWarning);
                }
            }
        }
    }
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
            StateScoped(MenuState::Controls),
        ))
        .with_children(|builder| {
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
                    controls
                        .clone()
                        .into_iter()
                        .for_each(|keybind| controls_row(builder, &style, keybind))
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
                    builder.spawn((
                        Button,
                        button_node.clone(),
                        BackgroundColor(style.button_color),
                        children![(Text::new("Back"), button_text_style.clone(), Pickable::IGNORE)],
                    ))
                        .observe(back_button_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            children![(Text::new("Save"), button_text_style.clone())],
                        ))
                        .observe(save_changes_on_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            children![(Text::new("Discard"), button_text_style.clone())],
                        ))
                        .observe(discard_changes_on_click);

                    builder
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(style.button_color),
                            children![(Text::new("Reset All"), button_text_style.clone())],
                        ))
                        .observe(reset_controls_on_click);

                    builder.spawn((
                        Text::new(
                            "Note: The keys show are based on the physical key and may not reflect the keyboard input in a text box.",
                        ),
                        (
                            style.font(18.0),
                            TextLayout::new_with_justify(JustifyText::Center),
                        ),
                        Pickable::IGNORE,
                    ));
                });
        });
}

fn controls_row(builder: &mut ChildSpawnerCommands<'_>, style: &Style, keybind: Keybind) {
    let Keybind(control, keys) = keybind;
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
                        Text::new(control.to_string()),
                        TextColor(style.title_color),
                        style.font(33.0),
                        Pickable::IGNORE,
                    ));
                });

            for (i, key) in keys.into_iter().enumerate() {
                builder
                    .spawn((
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
                        PromptButton(control, i),
                        Pickable {
                            should_block_lower: false,
                            is_hoverable: true,
                        },
                    ))
                    .observe(prompt_on_click)
                    .with_children(|builder| input_to_screen(style, builder, &key));
            }

            builder
                .spawn((
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
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                    children![(
                        Text("Reset Both".into()),
                        style.font(33.0),
                        TextColor(style.text_color)
                    )],
                ))
                .observe(reset_control_on_click(control));
        });
}

fn controls_changed(
    mut commands: Commands,
    style: Res<Style>,
    controls: Res<ControlsWIP>,
    button: Query<(Entity, &PromptButton, &Children)>,
) {
    for (entity, PromptButton(control, entry), children) in button.iter() {
        let key = controls.0.get_control_part(*control, *entry);
        for child in children {
            commands.entity(*child).despawn();
        }
        commands
            .entity(entity)
            .remove_children(children)
            .with_children(|builder| input_to_screen(&style, builder, &key));
    }
}

fn control_prompt_enter(mut commands: Commands, style: Res<Style>) {
    let button_text_style = (
        style.font(33.0),
        TextColor(style.text_color),
        TextLayout::new_with_justify(JustifyText::Center),
    );

    commands.spawn((
        Node {
            display: Display::Flex,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        StateScoped(ControlsState::Prompt),
        BackgroundColor(style.background_color.with_alpha(1.0)),
        ZIndex(2),
        children![(
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    Text::new("Press any key to bind,"),
                    style.font(33.0),
                    TextColor(style.text_color),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),
                (
                    Text::new("or click 'Cancel'"),
                    style.font(33.0),
                    TextColor(style.text_color),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),
                (
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
                    CancelPromptButton,
                    children![(
                        Text::new("Cancel"),
                        button_text_style.clone(),
                        CancelPromptButton
                    )],
                )
            ],
        )],
    ));
}

fn assign_key_input(
    mut commands: Commands,
    mut keyboard: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut gamepad: EventReader<GamepadButtonChangedEvent>,
    mut controls: ResMut<ControlsWIP>,
    cancel_button_query: Query<Has<CancelPromptButton>>,
    target: Res<PromptTarget>,
    hover_map: Res<HoverMap>,
) {
    for ev in keyboard.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Keyboard(ev.key_code)));
                commands.set_state(ControlsState::Main);
                return;
            }
            ButtonState::Released => {}
        }
    }

    for ev in mouse.read() {
        match ev.state {
            ButtonState::Pressed => {
                if ev.button == MouseButton::Left {
                    for (_pointer, pointer_map) in hover_map.iter() {
                        for (entity, _hit) in pointer_map.iter() {
                            if let Ok(true) = cancel_button_query.get(*entity) {
                                commands.set_state(ControlsState::Main);
                                return;
                            }
                        }
                    }
                }

                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Mouse(ev.button)));
                commands.set_state(ControlsState::Main);
                return;
            }
            ButtonState::Released => {}
        }
    }

    for ev in gamepad.read() {
        match ev.state {
            ButtonState::Pressed => {
                controls
                    .0
                    .set_control(target.0, target.1, Some(Input::Gamepad(ev.button)));
                commands.set_state(ControlsState::Main);
                return;
            }
            ButtonState::Released => {}
        }
    }
}

fn control_save_warning_enter(mut commands: Commands, style: Res<Style>) {
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
            StateScoped(ControlsState::SaveWarning),
            BackgroundColor(style.background_color.with_alpha(1.0)),
            ZIndex(2),
        ))
        .with_children(|builder| {
            builder
                .spawn(Node {
                    display: Display::Flex,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
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
                            children![(Text::new("Save Changes"), button_text_style.clone(),)],
                        ))
                        .observe(save_changes_on_click)
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            MenuState::Settings,
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
                            children![(Text::new("Discard Changes"), button_text_style.clone(),)],
                        ))
                        .observe(discard_changes_on_click)
                        .observe(change_state_on_click(
                            PointerButton::Primary,
                            MenuState::Settings,
                        ));
                });
        });
}
