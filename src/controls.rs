use crate::embed_asset;
use crate::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::{input::InputSystem, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::iter::IntoIterator;

const KEYBINDS_DB_TABLE: &str = "Keybinds";

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/buttons.png");

        app.add_systems(PreStartup, setup_controls)
            .init_resource::<ControlState>()
            .init_resource::<ButtonInput<Input>>()
            .add_systems(
                PreUpdate,
                (update_input_state, update_control_state)
                    .chain()
                    .after(InputSystem),
            )
            .add_systems(
                Update,
                controls_sync
                    .run_if(resource_changed::<Controls>.and(not(resource_added::<Controls>))),
            );
    }
}

fn setup_controls(mut commands: Commands, database: NonSend<Database>) {
    commands.insert_resource(Controls::from_database(&database));
}

#[derive(Clone, Default, Resource)]
pub struct ControlState {
    pressed: HashMap<Control, f32>,
    just_pressed: HashSet<Control>,
    just_released: HashSet<Control>,
}

/// Taken from [`bevy::input::ButtonInput`] so we could replace a hash set with a hash map.
impl ControlState {
    /// Registers a press for the given `input`.
    pub fn press(&mut self, input: Control, value: f32) {
        // Returns `true` if the `input` wasn't pressed before.
        if self.pressed.insert(input, value).is_none() {
            self.just_pressed.insert(input);
        }
    }

    /// Returns `true` if the `input` has been pressed.
    pub fn pressed(&self, input: Control) -> bool {
        self.pressed.contains_key(&input)
    }

    /// Returns `true` if any item in `inputs` has been pressed.
    pub fn any_pressed(&self, inputs: impl IntoIterator<Item = Control>) -> bool {
        inputs.into_iter().any(|it| self.pressed(it))
    }

    /// Returns `true` if all items in `inputs` have been pressed.
    pub fn all_pressed(&self, inputs: impl IntoIterator<Item = Control>) -> bool {
        inputs.into_iter().all(|it| self.pressed(it))
    }

    /// Registers a release for the given `input`.
    pub fn release(&mut self, input: Control) {
        // Returns `true` if the `input` was pressed.
        if self.pressed.remove(&input).is_some() {
            self.just_released.insert(input);
        }
    }

    /// Registers a release for all currently pressed inputs.
    pub fn release_all(&mut self) {
        // Move all items from pressed into just_released
        self.just_released
            .extend(self.pressed.drain().map(|(c, _)| c));
    }

    /// Returns `true` if the `input` has been pressed during the current frame.
    ///
    /// Note: This function does not imply information regarding the current state of [`ControlState::pressed`] or [`ControlState::just_released`].
    pub fn just_pressed(&self, input: Control) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has been pressed during the current frame.
    pub fn any_just_pressed(&self, inputs: impl IntoIterator<Item = Control>) -> bool {
        inputs.into_iter().any(|it| self.just_pressed(it))
    }

    /// Clears the `just_pressed` state of the `input` and returns `true` if the `input` has just been pressed.
    ///
    /// Future calls to [`ControlState::just_pressed`] for the given input will return false until a new press event occurs.
    pub fn clear_just_pressed(&mut self, input: Control) -> bool {
        self.just_pressed.remove(&input)
    }

    /// Returns `true` if the `input` has been released during the current frame.
    ///
    /// Note: This function does not imply information regarding the current state of [`ControlState::pressed`] or [`ControlState::just_pressed`].
    pub fn just_released(&self, input: Control) -> bool {
        self.just_released.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has just been released.
    pub fn any_just_released(&self, inputs: impl IntoIterator<Item = Control>) -> bool {
        inputs.into_iter().any(|input| self.just_released(input))
    }

    /// Returns `true` if all items in `inputs` have just been released.
    pub fn all_just_released(&self, inputs: impl IntoIterator<Item = Control>) -> bool {
        inputs.into_iter().all(|input| self.just_released(input))
    }

    /// Returns `true` if all items in `inputs` have been just pressed.
    pub fn all_just_pressed(&self, inputs: impl IntoIterator<Item = Control>) -> bool {
        inputs.into_iter().all(|input| self.just_pressed(input))
    }

    /// Clears the `just_released` state of the `input` and returns `true` if the `input` has just been released.
    ///
    /// Future calls to [`ControlState::just_released`] for the given input will return false until a new release event occurs.
    pub fn clear_just_released(&mut self, input: Control) -> bool {
        self.just_released.remove(&input)
    }

    /// Clears the `pressed`, `just_pressed` and `just_released` data of the `input`.
    pub fn reset(&mut self, input: Control) {
        self.pressed.remove(&input);
        self.just_pressed.remove(&input);
        self.just_released.remove(&input);
    }

    /// Clears the `pressed`, `just_pressed`, and `just_released` data for every input.
    ///
    /// See also [`ControlState::clear`] for simulating elapsed time steps.
    pub fn reset_all(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Clears the `just pressed` and `just released` data for every input.
    ///
    /// See also [`ControlState::reset_all`] for a full reset.
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// An iterator visiting every pressed input in arbitrary order.
    pub fn get_pressed(&self) -> impl ExactSizeIterator<Item = (&Control, &f32)> {
        self.pressed.iter()
    }

    /// An iterator visiting every just pressed input in arbitrary order.
    ///
    /// Note: Returned elements do not imply information regarding the current state of [`ControlState::pressed`] or [`ControlState::just_released`].
    pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item = &Control> {
        self.just_pressed.iter()
    }

    /// An iterator visiting every just released input in arbitrary order.
    ///
    /// Note: Returned elements do not imply information regarding the current state of [`ControlState::pressed`] or [`ControlState::just_pressed`].
    pub fn get_just_released(&self) -> impl ExactSizeIterator<Item = &Control> {
        self.just_released.iter()
    }
}

/// This function isn't ideal, but I don't know if there
/// is a better way to do it with how we need.
fn update_input_state(
    mut input_state: ResMut<ButtonInput<Input>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepad: Query<&Gamepad>,
) {
    input_state.bypass_change_detection().clear();

    for pressed in keyboard.get_just_pressed() {
        input_state.press(Input::Keyboard(*pressed));
    }

    for released in keyboard.get_just_released() {
        input_state.release(Input::Keyboard(*released));
    }

    for pressed in mouse.get_just_pressed() {
        input_state.press(Input::Mouse(*pressed));
    }

    for released in mouse.get_just_released() {
        input_state.release(Input::Mouse(*released));
    }

    for gamepad in gamepad.iter() {
        for pressed in gamepad.digital().get_just_pressed() {
            input_state.press(Input::Gamepad(*pressed));
        }

        for released in gamepad.digital().get_just_released() {
            input_state.release(Input::Gamepad(*released));
        }
    }
}

fn update_control_state(
    mut control_state: ResMut<ControlState>,
    input_state: Res<ButtonInput<Input>>,
    controls: Res<Controls>,
) {
    // Avoid clearing if it's not empty to ensure change detection is not triggered.
    control_state.bypass_change_detection().clear();

    for Keybind(control, keybind) in controls.clone().into_iter() {
        let keybind = keybind.into_iter().filter_map(|k| k);

        let pressed = input_state.any_pressed(keybind.clone());
        let just_pressed = input_state.any_just_pressed(keybind.clone());
        let just_released = input_state.any_just_released(keybind);

        if just_pressed {
            control_state.press(control, 1.0);
        }

        if just_released && !pressed {
            control_state.release(control);
        }
    }
}

/// All of the information about an individual keybind
#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Keybind(pub Control, pub InputList);

impl Keybind {
    pub fn to_screen(&self, _style: &Style, _builder: &mut ChildSpawnerCommands) {
        todo!("display multiple");
    }
}

const TEXT_COLOR: Color = Color::srgb_u8(0xe0, 0xde, 0xf4);

pub fn input_to_screen(style: &Style, builder: &mut ChildSpawnerCommands, input: &Option<Input>) {
    match input {
        Some(input) => style.display_input(builder, input),
        None => {
            builder.spawn((
                Text::new("Not Bound"),
                TextFont {
                    font: style.font.clone(),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Label,
                Pickable::IGNORE,
            ));
        }
    }
}

/// The number of keybinds associated with a given control.
/// When changed, the update must be in the database
/// so that we sync all of them correctly.
const INPUT_LIST_LEN: usize = 2;
/// An individual set of inputs for a keybind
pub type InputList = [Option<Input>; INPUT_LIST_LEN];

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Input {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    MouseWheelAxis(MouseWheelAxis),
    Gamepad(GamepadButton),
    GamepadAxis(GamepadAxis),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MouseWheelAxis {
    X,
    Y,
}

// sometimes, you just have to do this...
impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use GamepadAxis as GA;
        use GamepadButton as G;
        use Input as I;
        use KeyCode as K;
        use MouseButton as M;
        use MouseWheelAxis as MA;

        match self {
            I::Keyboard(K::Unidentified(_)) => write!(f, "Unidentified"),
            I::Keyboard(K::Backquote) => write!(f, "`"),
            I::Keyboard(K::Backslash) => write!(f, "\\"),
            I::Keyboard(K::BracketLeft) => write!(f, "["),
            I::Keyboard(K::BracketRight) => write!(f, "]"),
            I::Keyboard(K::Comma) => write!(f, ","),
            I::Keyboard(K::Digit0) => write!(f, "0"),
            I::Keyboard(K::Digit1) => write!(f, "1"),
            I::Keyboard(K::Digit2) => write!(f, "2"),
            I::Keyboard(K::Digit3) => write!(f, "3"),
            I::Keyboard(K::Digit4) => write!(f, "4"),
            I::Keyboard(K::Digit5) => write!(f, "5"),
            I::Keyboard(K::Digit6) => write!(f, "6"),
            I::Keyboard(K::Digit7) => write!(f, "7"),
            I::Keyboard(K::Digit8) => write!(f, "8"),
            I::Keyboard(K::Digit9) => write!(f, "9"),
            I::Keyboard(K::Equal) => write!(f, "="),
            // should be show this as a backslash?
            I::Keyboard(K::IntlBackslash) => write!(f, "\\"),
            // should be show this as a backslash?
            I::Keyboard(K::IntlRo) => write!(f, "\\"),
            I::Keyboard(K::IntlYen) => write!(f, "¥"),
            I::Keyboard(K::KeyA) => write!(f, "A"),
            I::Keyboard(K::KeyB) => write!(f, "B"),
            I::Keyboard(K::KeyC) => write!(f, "C"),
            I::Keyboard(K::KeyD) => write!(f, "D"),
            I::Keyboard(K::KeyE) => write!(f, "E"),
            I::Keyboard(K::KeyF) => write!(f, "F"),
            I::Keyboard(K::KeyG) => write!(f, "G"),
            I::Keyboard(K::KeyH) => write!(f, "H"),
            I::Keyboard(K::KeyI) => write!(f, "I"),
            I::Keyboard(K::KeyJ) => write!(f, "J"),
            I::Keyboard(K::KeyK) => write!(f, "K"),
            I::Keyboard(K::KeyL) => write!(f, "L"),
            I::Keyboard(K::KeyM) => write!(f, "M"),
            I::Keyboard(K::KeyN) => write!(f, "N"),
            I::Keyboard(K::KeyO) => write!(f, "O"),
            I::Keyboard(K::KeyP) => write!(f, "P"),
            I::Keyboard(K::KeyQ) => write!(f, "Q"),
            I::Keyboard(K::KeyR) => write!(f, "R"),
            I::Keyboard(K::KeyS) => write!(f, "S"),
            I::Keyboard(K::KeyT) => write!(f, "T"),
            I::Keyboard(K::KeyU) => write!(f, "U"),
            I::Keyboard(K::KeyV) => write!(f, "V"),
            I::Keyboard(K::KeyW) => write!(f, "W"),
            I::Keyboard(K::KeyX) => write!(f, "X"),
            I::Keyboard(K::KeyY) => write!(f, "Y"),
            I::Keyboard(K::KeyZ) => write!(f, "Z"),
            I::Keyboard(K::Minus) => write!(f, "-"),
            I::Keyboard(K::Period) => write!(f, "."),
            I::Keyboard(K::Quote) => write!(f, "'"),
            I::Keyboard(K::Semicolon) => write!(f, ";"),
            I::Keyboard(K::Slash) => write!(f, "/"),
            I::Keyboard(K::AltLeft) => write!(f, "ALT"),
            I::Keyboard(K::AltRight) => write!(f, "RIGHT ALT"),
            I::Keyboard(K::Backspace) => write!(f, "BACKSPACE"),
            I::Keyboard(K::CapsLock) => write!(f, "CAPS"),
            I::Keyboard(K::ContextMenu) => write!(f, "CONTEXT MENU"),
            I::Keyboard(K::ControlLeft) => write!(f, "CTRL"),
            I::Keyboard(K::ControlRight) => write!(f, "RIGHT CTRL"),
            I::Keyboard(K::Enter) => write!(f, "ENTER"),
            I::Keyboard(K::SuperLeft) => write!(f, "OS"),
            I::Keyboard(K::SuperRight) => write!(f, "OS RIGHT"),
            I::Keyboard(K::ShiftLeft) => write!(f, "SHIFT"),
            I::Keyboard(K::ShiftRight) => write!(f, "RIGHT SHIFT"),
            I::Keyboard(K::Space) => write!(f, "SPACE"),
            I::Keyboard(K::Tab) => write!(f, "TAB"),
            I::Keyboard(K::Convert) => write!(f, "CONVERT"),
            I::Keyboard(K::KanaMode) => write!(f, "KANA MODE"),
            I::Keyboard(K::Lang1) => write!(f, "LANG 1"),
            I::Keyboard(K::Lang2) => write!(f, "LANG 2"),
            I::Keyboard(K::Lang3) => write!(f, "LANG 3"),
            I::Keyboard(K::Lang4) => write!(f, "LANG 4"),
            I::Keyboard(K::Lang5) => write!(f, "LANG 5"),
            I::Keyboard(K::NonConvert) => write!(f, "NON-CONVERT"),
            I::Keyboard(K::Delete) => write!(f, "DELETE"),
            I::Keyboard(K::End) => write!(f, "END"),
            I::Keyboard(K::Help) => write!(f, "HELP"),
            I::Keyboard(K::Home) => write!(f, "HOME"),
            I::Keyboard(K::Insert) => write!(f, "INSERT"),
            I::Keyboard(K::PageDown) => write!(f, "PAGE DOWN"),
            I::Keyboard(K::PageUp) => write!(f, "PAGE UP"),
            I::Keyboard(K::ArrowDown) => write!(f, "DOWN ARROW"),
            I::Keyboard(K::ArrowLeft) => write!(f, "LEFT ARROW"),
            I::Keyboard(K::ArrowRight) => write!(f, "RIGHT ARROW"),
            I::Keyboard(K::ArrowUp) => write!(f, "UP ARROW"),
            I::Keyboard(K::NumLock) => write!(f, "NUM LOCK"),
            I::Keyboard(K::Numpad0) => write!(f, "NUMPAD 0"),
            I::Keyboard(K::Numpad1) => write!(f, "NUMPAD 1"),
            I::Keyboard(K::Numpad2) => write!(f, "NUMPAD 2"),
            I::Keyboard(K::Numpad3) => write!(f, "NUMPAD 3"),
            I::Keyboard(K::Numpad4) => write!(f, "NUMPAD 4"),
            I::Keyboard(K::Numpad5) => write!(f, "NUMPAD 5"),
            I::Keyboard(K::Numpad6) => write!(f, "NUMPAD 6"),
            I::Keyboard(K::Numpad7) => write!(f, "NUMPAD 7"),
            I::Keyboard(K::Numpad8) => write!(f, "NUMPAD 8"),
            I::Keyboard(K::Numpad9) => write!(f, "NUMPAD 9"),
            I::Keyboard(K::NumpadAdd) => write!(f, "NUMPAD +"),
            I::Keyboard(K::NumpadBackspace) => write!(f, "NUMPAD BACKSPACE"),
            I::Keyboard(K::NumpadClear) => write!(f, "NUMPAD CLEAR"),
            I::Keyboard(K::NumpadClearEntry) => write!(f, "NUMPAD CLEAR ENTRY"),
            I::Keyboard(K::NumpadComma) => write!(f, "NUMPAD ,"),
            I::Keyboard(K::NumpadDecimal) => write!(f, "NUMPAD ."),
            I::Keyboard(K::NumpadDivide) => write!(f, "NUMPAD /"),
            I::Keyboard(K::NumpadEnter) => write!(f, "NUMPAD ENTER"),
            I::Keyboard(K::NumpadEqual) => write!(f, "NUMPAD ="),
            I::Keyboard(K::NumpadHash) => write!(f, "NUMPAD #"),
            I::Keyboard(K::NumpadMemoryAdd) => write!(f, "NUMPAD MEMORY ADD"),
            I::Keyboard(K::NumpadMemoryClear) => write!(f, "NUMPAD MEMORY CLEAR"),
            I::Keyboard(K::NumpadMemoryRecall) => write!(f, "NUMPAD MEMORY RECALL"),
            I::Keyboard(K::NumpadMemoryStore) => write!(f, "NUMPAD MEMORY STORE"),
            I::Keyboard(K::NumpadMemorySubtract) => write!(f, "NUMPAD MEMORY SUBTRACT"),
            I::Keyboard(K::NumpadMultiply) => write!(f, "NUMPAD MULTIPLY"),
            I::Keyboard(K::NumpadParenLeft) => write!(f, "NUMPAD ("),
            I::Keyboard(K::NumpadParenRight) => write!(f, "NUMPAD )"),
            I::Keyboard(K::NumpadStar) => write!(f, "NUMPAD STAR"),
            I::Keyboard(K::NumpadSubtract) => write!(f, "NUMPAD -"),
            I::Keyboard(K::Escape) => write!(f, "ESC"),
            I::Keyboard(K::Fn) => write!(f, "FN"),
            I::Keyboard(K::FnLock) => write!(f, "FN LOCK"),
            I::Keyboard(K::PrintScreen) => write!(f, "PRINT SCREEN"),
            I::Keyboard(K::ScrollLock) => write!(f, "SCROLL LOCK"),
            I::Keyboard(K::Pause) => write!(f, "PAUSE"),
            I::Keyboard(K::BrowserBack) => write!(f, "BROWSER BACK"),
            I::Keyboard(K::BrowserFavorites) => write!(f, "BROWSER FAVORITES"),
            I::Keyboard(K::BrowserForward) => write!(f, "BROWSER FORWARD"),
            I::Keyboard(K::BrowserHome) => write!(f, "BROWSER HOME"),
            I::Keyboard(K::BrowserRefresh) => write!(f, "BROWSER REFRESH"),
            I::Keyboard(K::BrowserSearch) => write!(f, "BROWSER SEARCH"),
            I::Keyboard(K::BrowserStop) => write!(f, "BROWSER STOP"),
            I::Keyboard(K::Eject) => write!(f, "EJECT"),
            I::Keyboard(K::LaunchApp1) => write!(f, "LAUNCH APP 1"),
            I::Keyboard(K::LaunchApp2) => write!(f, "LAUNCH APP 2"),
            I::Keyboard(K::LaunchMail) => write!(f, "LAUNCH APP 3"),
            I::Keyboard(K::MediaPlayPause) => write!(f, "MEDIA PAUSE"),
            I::Keyboard(K::MediaSelect) => write!(f, "MEDIA SELECT"),
            I::Keyboard(K::MediaStop) => write!(f, "MEDIA STOP"),
            I::Keyboard(K::MediaTrackNext) => write!(f, "MEDIA TRACK NEXT"),
            I::Keyboard(K::MediaTrackPrevious) => write!(f, "MEDIA TRACK PREVIOUS"),
            I::Keyboard(K::Power) => write!(f, "POWER"),
            I::Keyboard(K::Sleep) => write!(f, "SLEEP"),
            I::Keyboard(K::AudioVolumeDown) => write!(f, "AUDIO VOLUME DOWN"),
            I::Keyboard(K::AudioVolumeMute) => write!(f, "AUDIO VOLUME MUTE"),
            I::Keyboard(K::AudioVolumeUp) => write!(f, "AUDIO VOLUME UP"),
            I::Keyboard(K::WakeUp) => write!(f, "WAKE UP"),
            I::Keyboard(K::Meta) => write!(f, "META"),
            I::Keyboard(K::Hyper) => write!(f, "HYPR"),
            I::Keyboard(K::Turbo) => write!(f, "TURBO"),
            I::Keyboard(K::Abort) => write!(f, "ABORT"),
            I::Keyboard(K::Resume) => write!(f, "RESUME"),
            I::Keyboard(K::Suspend) => write!(f, "SUSPEND"),
            I::Keyboard(K::Again) => write!(f, "AGAIN"),
            I::Keyboard(K::Copy) => write!(f, "COPY"),
            I::Keyboard(K::Cut) => write!(f, "CUT"),
            I::Keyboard(K::Find) => write!(f, "FIND"),
            I::Keyboard(K::Open) => write!(f, "OPEN"),
            I::Keyboard(K::Paste) => write!(f, "PASTE"),
            I::Keyboard(K::Props) => write!(f, "PROPS"),
            I::Keyboard(K::Select) => write!(f, "SELECT"),
            I::Keyboard(K::Undo) => write!(f, "UNDO"),
            I::Keyboard(K::Hiragana) => write!(f, "HIRAGANA"),
            I::Keyboard(K::Katakana) => write!(f, "KATAKANA"),
            I::Keyboard(K::F1) => write!(f, "F1"),
            I::Keyboard(K::F2) => write!(f, "F2"),
            I::Keyboard(K::F3) => write!(f, "F3"),
            I::Keyboard(K::F4) => write!(f, "F4"),
            I::Keyboard(K::F5) => write!(f, "F5"),
            I::Keyboard(K::F6) => write!(f, "F6"),
            I::Keyboard(K::F7) => write!(f, "F7"),
            I::Keyboard(K::F8) => write!(f, "F8"),
            I::Keyboard(K::F9) => write!(f, "F9"),
            I::Keyboard(K::F10) => write!(f, "F10"),
            I::Keyboard(K::F11) => write!(f, "F11"),
            I::Keyboard(K::F12) => write!(f, "F12"),
            I::Keyboard(K::F13) => write!(f, "F13"),
            I::Keyboard(K::F14) => write!(f, "F14"),
            I::Keyboard(K::F15) => write!(f, "F15"),
            I::Keyboard(K::F16) => write!(f, "F16"),
            I::Keyboard(K::F17) => write!(f, "F17"),
            I::Keyboard(K::F18) => write!(f, "F18"),
            I::Keyboard(K::F19) => write!(f, "F19"),
            I::Keyboard(K::F20) => write!(f, "F20"),
            I::Keyboard(K::F21) => write!(f, "F21"),
            I::Keyboard(K::F22) => write!(f, "F22"),
            I::Keyboard(K::F23) => write!(f, "F23"),
            I::Keyboard(K::F24) => write!(f, "F24"),
            I::Keyboard(K::F25) => write!(f, "F25"),
            I::Keyboard(K::F26) => write!(f, "F26"),
            I::Keyboard(K::F27) => write!(f, "F27"),
            I::Keyboard(K::F28) => write!(f, "F28"),
            I::Keyboard(K::F29) => write!(f, "F29"),
            I::Keyboard(K::F30) => write!(f, "F30"),
            I::Keyboard(K::F31) => write!(f, "F31"),
            I::Keyboard(K::F32) => write!(f, "F32"),
            I::Keyboard(K::F33) => write!(f, "F33"),
            I::Keyboard(K::F34) => write!(f, "F34"),
            I::Keyboard(K::F35) => write!(f, "F35"),
            I::Mouse(M::Left) => write!(f, "LEFT CLICK"),
            I::Mouse(M::Right) => write!(f, "RIGHT CLICK"),
            I::Mouse(M::Middle) => write!(f, "MIDDLE CLICK"),
            I::Mouse(M::Back) => write!(f, "MOUSE BACK"),
            I::Mouse(M::Forward) => write!(f, "MOUSE FORWARD"),
            I::Mouse(M::Other(other)) => write!(f, "MOUSE BUTTON {}", other),
            I::MouseWheelAxis(MA::X) => write!(f, "MOUSE WHEEL X AXIS"),
            I::MouseWheelAxis(MA::Y) => write!(f, "MOUSE WHEEL Y AXIS"),
            I::Gamepad(G::South) => write!(f, "GAMEPAD SOUTH"),
            I::Gamepad(G::East) => write!(f, "GAMEPAD EAST"),
            I::Gamepad(G::North) => write!(f, "GAMEPAD NORTH"),
            I::Gamepad(G::West) => write!(f, "GAMEPAD WEST"),
            I::Gamepad(G::C) => write!(f, "GAMEPAD C"),
            I::Gamepad(G::Z) => write!(f, "GAMEPAD Z"),
            I::Gamepad(G::LeftTrigger) => write!(f, "LEFT TRIGGER"),
            I::Gamepad(G::LeftTrigger2) => write!(f, "LEFT TRIGGER 2"),
            I::Gamepad(G::RightTrigger) => write!(f, "RIGHT TRIGGER"),
            I::Gamepad(G::RightTrigger2) => write!(f, "RIGHT TRIGGER 2"),
            I::Gamepad(G::Select) => write!(f, "SELECT"),
            I::Gamepad(G::Start) => write!(f, "START"),
            I::Gamepad(G::Mode) => write!(f, "MODE"),
            I::Gamepad(G::LeftThumb) => write!(f, "LEFT THUMB"),
            I::Gamepad(G::RightThumb) => write!(f, "RIGHT THUMB"),
            I::Gamepad(G::DPadUp) => write!(f, "DPAD UP"),
            I::Gamepad(G::DPadDown) => write!(f, "DPAD DOWN"),
            I::Gamepad(G::DPadLeft) => write!(f, "DPAD LEFT"),
            I::Gamepad(G::DPadRight) => write!(f, "DPAD RIGHT"),
            I::Gamepad(G::Other(other)) => write!(f, "GAMEPAD BUTTON {other}"),
            I::GamepadAxis(GA::LeftStickX) => write!(f, "GAMEPAD LEFT STICK X"),
            I::GamepadAxis(GA::LeftStickY) => write!(f, "GAMEPAD LEFT STICK Y"),
            I::GamepadAxis(GA::LeftZ) => write!(f, "GAMPAD LEFT STICK Z"),
            I::GamepadAxis(GA::RightStickX) => write!(f, "GAMEPAD RIGHT STICK X"),
            I::GamepadAxis(GA::RightStickY) => write!(f, "GAMEPAD RIGHT STICK Y"),
            I::GamepadAxis(GA::RightZ) => write!(f, "GAMEPAD RIGHT STICK Z"),
            I::GamepadAxis(GA::Other(other)) => write!(f, "GAMEPAD AXIS {other}"),
        }
    }
}

/// The list of controls for each input
#[derive(Resource, Clone, Eq, PartialEq, Debug)]
pub struct Controls {
    pub move_up: InputList,
    pub move_down: InputList,
    pub move_left: InputList,
    pub move_right: InputList,
    pub zoom_in: InputList,
    pub zoom_out: InputList,
    pub pause: InputList,
    pub select: InputList,
}

impl Controls {
    pub fn get_control_mut(&mut self, control: Control) -> &mut InputList {
        match control {
            Control::MoveUp => &mut self.move_up,
            Control::MoveDown => &mut self.move_down,
            Control::MoveLeft => &mut self.move_left,
            Control::MoveRight => &mut self.move_right,
            Control::ZoomIn => &mut self.zoom_in,
            Control::ZoomOut => &mut self.zoom_out,
            Control::Pause => &mut self.pause,
            Control::Select => &mut self.select,
        }
    }

    pub fn get_control(&self, control: Control) -> InputList {
        match control {
            Control::MoveUp => self.move_up,
            Control::MoveDown => self.move_down,
            Control::MoveLeft => self.move_left,
            Control::MoveRight => self.move_right,
            Control::ZoomIn => self.zoom_in,
            Control::ZoomOut => self.zoom_out,
            Control::Pause => self.pause,
            Control::Select => self.select,
        }
    }

    pub fn get_control_part(&self, control: Control, entry: usize) -> Option<Input> {
        assert!(entry < INPUT_LIST_LEN);

        (self.get_control(control))[entry]
    }

    pub fn set_control(&mut self, control: Control, entry: usize, bind: Option<Input>) {
        assert!(entry < INPUT_LIST_LEN);

        self.get_control_mut(control)[entry] = bind;
    }

    pub fn reset_control(&mut self, control: Control) {
        *self.get_control_mut(control) = match control {
            Control::MoveUp => DEFAULT_UP_CONTROLS,
            Control::MoveDown => DEFAULT_DOWN_CONTROLS,
            Control::MoveLeft => DEFAULT_LEFT_CONTROLS,
            Control::MoveRight => DEFAULT_RIGHT_CONTROLS,
            Control::ZoomIn => DEFAULT_ZOOM_IN_CONTROLS,
            Control::ZoomOut => DEFAULT_ZOOM_OUT_CONTROLS,
            Control::Pause => DEFAULT_PAUSE_CONTROLS,
            Control::Select => DEFAULT_SELECT_CONTROLS,
        }
    }

    pub fn reset_control_part(&mut self, control: Control, i: usize) {
        assert!(i < INPUT_LIST_LEN);

        self.get_control_mut(control)[i] = match control {
            Control::MoveUp => DEFAULT_UP_CONTROLS,
            Control::MoveDown => DEFAULT_DOWN_CONTROLS,
            Control::MoveLeft => DEFAULT_LEFT_CONTROLS,
            Control::MoveRight => DEFAULT_RIGHT_CONTROLS,
            Control::ZoomIn => DEFAULT_ZOOM_IN_CONTROLS,
            Control::ZoomOut => DEFAULT_ZOOM_OUT_CONTROLS,
            Control::Pause => DEFAULT_PAUSE_CONTROLS,
            Control::Select => DEFAULT_SELECT_CONTROLS,
        }[i];
    }

    pub fn reset_controls(&mut self) {
        *self = default();
    }

    // TODO: Do this in a single transaction maybe? (don't know if it matters)
    fn from_database(db: &Database) -> Self {
        Self {
            move_up: db.get_kv(KEYBINDS_DB_TABLE, "move_up", DEFAULT_UP_CONTROLS),
            move_down: db.get_kv(KEYBINDS_DB_TABLE, "move_down", DEFAULT_DOWN_CONTROLS),
            move_left: db.get_kv(KEYBINDS_DB_TABLE, "move_left", DEFAULT_LEFT_CONTROLS),
            move_right: db.get_kv(KEYBINDS_DB_TABLE, "move_right", DEFAULT_RIGHT_CONTROLS),
            zoom_in: db.get_kv(KEYBINDS_DB_TABLE, "zoom_in", DEFAULT_ZOOM_IN_CONTROLS),
            zoom_out: db.get_kv(KEYBINDS_DB_TABLE, "zoom_out", DEFAULT_ZOOM_OUT_CONTROLS),
            pause: db.get_kv(KEYBINDS_DB_TABLE, "pause", DEFAULT_PAUSE_CONTROLS),
            select: db.get_kv(KEYBINDS_DB_TABLE, "select", DEFAULT_SELECT_CONTROLS),
        }
    }

    //// TODO: Do this in a single transaction maybe? (don't know if it matters)
    fn to_database(&self, db: &Database) -> Result<(), crate::database::SetKvError> {
        db.set_kv(KEYBINDS_DB_TABLE, "move_up", self.move_up)?;
        db.set_kv(KEYBINDS_DB_TABLE, "move_down", self.move_down)?;
        db.set_kv(KEYBINDS_DB_TABLE, "move_left", self.move_left)?;
        db.set_kv(KEYBINDS_DB_TABLE, "move_right", self.move_right)?;
        db.set_kv(KEYBINDS_DB_TABLE, "zoom_in", self.zoom_in)?;
        db.set_kv(KEYBINDS_DB_TABLE, "zoom_out", self.zoom_out)?;
        db.set_kv(KEYBINDS_DB_TABLE, "pause", self.pause)?;
        db.set_kv(KEYBINDS_DB_TABLE, "select", self.select)?;

        Ok(())
    }
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            move_up: DEFAULT_UP_CONTROLS,
            move_down: DEFAULT_DOWN_CONTROLS,
            move_left: DEFAULT_LEFT_CONTROLS,
            move_right: DEFAULT_RIGHT_CONTROLS,
            zoom_in: DEFAULT_ZOOM_IN_CONTROLS,
            zoom_out: DEFAULT_ZOOM_OUT_CONTROLS,
            pause: DEFAULT_PAUSE_CONTROLS,
            select: DEFAULT_SELECT_CONTROLS,
        }
    }
}

impl IntoIterator for Controls {
    type Item = Keybind;
    type IntoIter = ControlsIter;

    fn into_iter(self) -> ControlsIter {
        ControlsIter {
            controls: self,
            current: Some(default()),
        }
    }
}

#[derive(Default)]
pub struct ControlsIter {
    controls: Controls,
    current: Option<Control>,
}

impl Iterator for ControlsIter {
    type Item = Keybind;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.and_then(|control| {
            let res = match control {
                Control::MoveUp => Keybind(Control::MoveUp, self.controls.move_up),
                Control::MoveDown => Keybind(Control::MoveDown, self.controls.move_down),
                Control::MoveLeft => Keybind(Control::MoveLeft, self.controls.move_left),
                Control::MoveRight => Keybind(Control::MoveRight, self.controls.move_right),
                Control::ZoomIn => Keybind(Control::ZoomIn, self.controls.zoom_in),
                Control::ZoomOut => Keybind(Control::ZoomOut, self.controls.zoom_out),
                Control::Pause => Keybind(Control::Pause, self.controls.pause),
                Control::Select => Keybind(Control::Select, self.controls.select),
            };

            self.current = control.next();

            Some(res)
        })
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Control {
    #[default]
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ZoomIn,
    ZoomOut,
    Pause,
    Select,
}

impl Control {
    pub fn next(self) -> Option<Self> {
        match self {
            Control::MoveUp => Some(Control::MoveDown),
            Control::MoveDown => Some(Control::MoveLeft),
            Control::MoveLeft => Some(Control::MoveRight),
            Control::MoveRight => Some(Control::ZoomIn),
            Control::ZoomIn => Some(Control::ZoomOut),
            Control::ZoomOut => Some(Control::Pause),
            Control::Pause => Some(Control::Select),
            Control::Select => None,
        }
    }

    pub fn as_string(self) -> &'static str {
        match self {
            Control::MoveUp => "Move Up",
            Control::MoveDown => "Move Down",
            Control::MoveLeft => "Move Left",
            Control::MoveRight => "Move Right",
            Control::ZoomIn => "Zoom In",
            Control::ZoomOut => "Zoom Out",
            Control::Pause => "Pause",
            Control::Select => "Select",
        }
    }
}

use std::fmt::{Display, Formatter};
impl Display for Control {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_string())
    }
}

const DEFAULT_UP_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowUp)),
    Some(Input::Keyboard(KeyCode::KeyW)),
];
const DEFAULT_DOWN_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowDown)),
    Some(Input::Keyboard(KeyCode::KeyS)),
];
const DEFAULT_LEFT_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowLeft)),
    Some(Input::Keyboard(KeyCode::KeyA)),
];
const DEFAULT_RIGHT_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::ArrowRight)),
    Some(Input::Keyboard(KeyCode::KeyD)),
];
const DEFAULT_ZOOM_IN_CONTROLS: InputList = [Some(Input::Keyboard(KeyCode::Comma)), None];
const DEFAULT_ZOOM_OUT_CONTROLS: InputList = [Some(Input::Keyboard(KeyCode::Period)), None];
const DEFAULT_PAUSE_CONTROLS: InputList = [
    Some(Input::Keyboard(KeyCode::Escape)),
    Some(Input::Keyboard(KeyCode::CapsLock)),
];
const DEFAULT_SELECT_CONTROLS: InputList = [
    Some(Input::Mouse(MouseButton::Left)),
    Some(Input::Keyboard(KeyCode::KeyE)),
];

fn controls_sync(database: NonSend<Database>, controls: Res<Controls>) {
    match controls.to_database(&database) {
        Ok(()) => {}
        Err(err) => {
            warn!("Failed to sync controls to database with: {err}");
        }
    };
}
