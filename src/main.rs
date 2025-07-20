mod camera;
mod consts;
mod controls;
mod database;
mod menu;
mod generate_map;
mod sky;
mod style;
mod tiles;
mod util;

pub mod prelude {
    use bevy::prelude::States;

    #[cfg(feature = "debug")]
    pub use bevy::dev_tools::states::log_transitions;
    pub type RandomSource = wyrand::WyRand;

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    pub enum GameState {
        InitialLoading,
        Menu,
        // TODO: Change back to `InitialLoading` when merging
        #[default]
        Game,
    }

    pub use crate::consts::*;

    pub use crate::camera::{
        CAMERA_DEFAULT_MOVE_AREA, CAMERA_DEFAULT_MOVE_SPEED, CAMERA_DEFAULT_SCALE,
        CAMERA_DEFAULT_ZOOM_LIMIT, CAMERA_DEFAULT_ZOOM_SPEED, CameraMovementSettings, MainCamera,
        MainCameraMarker,
    };
    pub use crate::controls::{Control, ControlState, Controls, Keybind};
    pub use crate::database::{Database, DatabaseError, FromDatabase, ToDatabase};
    pub use crate::style::{Icons, Style};
    pub use crate::util::*;
}

use camera::CameraPlugin;
use controls::ControlsPlugin;
use database::DatabasePlugin;
use menu::MenuPlugin;
use generate_map::GenerateMapPlugin;
use prelude::*;
use sky::SkyPlugin;
use style::StylePlugin;

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

    // Embed the sprite assets.
    embed_asset!(app, "assets/sprites/basic_sheet.png");

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
    // State
    #[cfg(feature = "debug")]
    app.add_systems(Update, log_transitions::<GameState>);
    app.init_state::<GameState>();
    // Local Plugins
    app.add_plugins(DatabasePlugin);

    app.add_plugins(StylePlugin)
        .add_plugins(ControlsPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(SkyPlugin)
        //.add_plugins(SavePlugin)
        .add_plugins(CameraPlugin)
        //.insert_resource::<GlobalRandom>(GlobalRandom(rand))
        //.add_systems(Startup, spawn_floors)
        .add_plugins(GenerateMapPlugin);
    app.add_systems(
        Update,
        check_textures.run_if(in_state(GameState::InitialLoading)),
    )
    .run();
}

// Wait for everything to load
// TODO: Find a better way to wait, because this just transitions immediately.
fn check_textures(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu);
}

//const AXIAL_DIRECTIONS: [AxialPos; 7] = [
//    AxialPos::new(0, 0),
//    AxialPos::new(1, 0),
//    AxialPos::new(0, 1),
//    AxialPos::new(0, -1),
//    AxialPos::new(-1, 0),
//    AxialPos::new(1, -1),
//    AxialPos::new(-1, 1),
//];
//
//const BASIC_TILE_SHEET_PATH: &'static str = "embedded://assets/sprites/basic_sheet.png";
//fn spawn_floors(mut commands: Commands, asset_server: Res<AssetServer>) {
//    let floor_texture = asset_server.load(BASIC_TILE_SHEET_PATH);
//
//    AXIAL_DIRECTIONS
//        .iter()
//        .map(|p| AxialPos::new(p.q * 4, p.r * 4))
//        .for_each(|trans| {
//            spawn_section(&mut commands, trans, floor_texture.clone());
//        });
//}
//
//fn spawn_section(commands: &mut Commands, origin: AxialPos, texture: Handle<Image>) {
//    let map_size = TilemapSize { x: 1, y: 2 };
//
//    let center =
//        (origin - AxialPos { q: 1, r: 0 }).center_in_world_row(&FLOOR_TILE_SIZE.as_vec2().into());
//
//    let mut tile_storage = TileStorage::empty(map_size);
//    let tilemap_entity = commands.spawn_empty().id();
//    let tilemap_id = TilemapId(tilemap_entity);
//    let hex_coord_system = HexCoordSystem::Row;
//    let map_type = TilemapType::Hexagon(hex_coord_system);
//
//    let tile_positions = generate_hexagon(
//        AxialPos::from_tile_pos_given_coord_system(&TilePos { x: 1, y: 1 }, hex_coord_system),
//        1,
//    )
//    .into_iter()
//    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(hex_coord_system));
//
//    commands.entity(tilemap_id.0).with_children(|parent| {
//        for tile_pos in tile_positions {
//            let tile_entity = parent
//                .spawn(TileBundle {
//                    position: tile_pos,
//                    tilemap_id,
//                    texture_index: TileTextureIndex(0),
//                    ..Default::default()
//                })
//                .id();
//            tile_storage.checked_set(&tile_pos, tile_entity)
//        }
//    });
//
//    commands.entity(tilemap_entity).insert(TilemapBundle {
//        grid_size: FLOOR_TILE_SIZE.as_vec2().into(),
//        size: map_size,
//        storage: tile_storage,
//        texture: TilemapTexture::Single(texture),
//        tile_size: FLOOR_TILE_SIZE.as_vec2().into(),
//        map_type,
//        anchor: TilemapAnchor::Center,
//        transform: Transform::from_translation(center.extend(0.)),
//        frustum_culling: FrustumCulling(true),
//        render_settings: TilemapRenderSettings {
//            y_sort: true,
//            ..Default::default()
//        },
//        ..Default::default()
//    });
//}
