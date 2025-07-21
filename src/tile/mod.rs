mod picking_backend;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::HexCoordSystem;
use bevy_ecs_tilemap::prelude::TilemapTileSize;
use bevy_ecs_tilemap::prelude::*;
use std::ops::Range;

#[cfg(feature = "debug")]
use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};

pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 48.0, y: 52.0 };
pub const TILE_SIZE_VEC: UVec2 = UVec2 { x: 48, y: 52 };
pub const TILE_ASSET_LOAD_PATH: &'static str = "embedded://assets/sprites/basic_sheet.png";
pub const TILE_ATLAS_SIZE: UVec2 = UVec2::new(15, 1);
pub const FLOOR_TILE_VARIENTS: Range<u32> = 0..6;
pub const SKY_TILE_VARIENTS: Range<u32> = 6..14;
pub const OUTLINE_TILE: u32 = 14;
pub const HEX_COORD_SYSTEM: HexCoordSystem = HexCoordSystem::Row;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugPickingPlugin).add_systems(
            PreUpdate,
            (|mut mode: ResMut<DebugPickingMode>| {
                *mode = match *mode {
                    DebugPickingMode::Disabled => DebugPickingMode::Normal,
                    _ => DebugPickingMode::Disabled,
                };
            })
            .run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F3,
            )),
        );
        app.add_plugins(picking_backend::TilemapBackend)
            .add_systems(PreStartup, setup_hex_tile_image);
    }
}

#[derive(Resource)]
pub struct HexTileImage {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct TileLabel;

fn setup_hex_tile_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image = asset_server.load(TILE_ASSET_LOAD_PATH);
    let layout = TextureAtlasLayout::from_grid(
        TILE_SIZE_VEC,
        TILE_ATLAS_SIZE.x,
        TILE_ATLAS_SIZE.y,
        None,
        None,
    );
    let layout = asset_server.add(layout);

    commands.insert_resource(HexTileImage { image, layout });
}

pub fn spawn_tile_labels<MapMarker: Component, TileMarker: Component>(
    mut commands: Commands,
    tilemap_q: Query<
        (
            &Transform,
            &TilemapType,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TileStorage,
            &TilemapAnchor,
        ),
        With<MapMarker>,
    >,
    tile_q: Query<&mut TilePos, With<TileMarker>>,
) {
    for (map_transform, map_type, map_size, grid_size, tile_size, tilemap_storage, anchor) in
        tilemap_q.iter()
    {
        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos
                .center_in_world(map_size, grid_size, tile_size, map_type, anchor)
                .extend(1.0);
            let transform = *map_transform * Transform::from_translation(tile_center);

            commands.spawn((
                Text2d::new(format!("{},{}", tile_pos.x, tile_pos.y)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                TextLayout::new_with_justify(JustifyText::Center),
                transform,
                TileLabel,
            ));
        }
    }
}
