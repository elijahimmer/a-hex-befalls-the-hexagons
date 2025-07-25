mod actor;
mod animation;
mod camera;
mod controls;
mod database;
mod game;
mod generate_map;
mod menu;
mod saving;
mod sky;
mod style;
mod tile;
mod util;

pub mod prelude {
    use bevy::prelude::*;

    #[cfg(feature = "debug")]
    pub use bevy::dev_tools::states::log_transitions;

    pub type RandomSource = wyrand::WyRand;

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    pub enum AppState {
        #[default]
        InitialLoading,
        Menu,
        Game,
    }

    pub use crate::actor::*;
    pub use crate::animation::AnimationConfig;
    pub use crate::camera::{MainCamera, MainCameraMarker};
    pub use crate::controls::{Control, ControlState, Controls, Keybind};
    pub use crate::database::{Database, DatabaseError, FromDatabase, ToDatabase};
    pub use crate::generate_map::{ROOM_TILE_LAYER, RoomTile, RoomTilemap};
    pub use crate::saving::{GameID, SaveGame, SaveGameInfo};
    pub use crate::style::{Icons, Style};
    pub use crate::tile::*;
    pub use crate::util::*;
}

use animation::AnimationPlugin;
use camera::CameraPlugin;
use controls::ControlsPlugin;
use database::DatabasePlugin;
use game::GamePlugin;
use generate_map::GenerateMapPlugin;
use menu::MenuPlugin;
use prelude::*;
use sky::SkyPlugin;
use style::StylePlugin;
use tile::TilePlugin;

#[cfg(feature = "debug")]
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    text::FontSmoothing,
};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ui_text_input::TextInputPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RaMmYen Game".into(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
    ); // fallback to nearest sampling

    #[cfg(feature = "debug")]
    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextFont {
                font_size: 18.0,
                font: default(),
                font_smoothing: FontSmoothing::default(),
                ..default()
            },
            text_color: FPS_COUNTER_COLOR,
            refresh_interval: core::time::Duration::from_millis(100),
            enabled: true,
        },
    });

    // third party plugins
    app.add_plugins(TilemapPlugin).add_plugins(TextInputPlugin);

    // Debug state transitions
    #[cfg(feature = "debug")]
    app.add_systems(Update, log_transitions::<AppState>);

    app.init_state::<AppState>();
    // Local Plugins
    app.add_plugins(DatabasePlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(TilePlugin)
        .add_plugins(GamePlugin)
        .add_plugins(StylePlugin)
        .add_plugins(ControlsPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(SkyPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(GenerateMapPlugin);

    app.add_systems(
        Update,
        check_textures.run_if(in_state(AppState::InitialLoading)),
    )
    .run();
}

/// Wait for all of the `StartUp` commands to run for first iteration
/// before the `OnEnter` triggers of the Main menu.
fn check_textures(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Menu);
}
