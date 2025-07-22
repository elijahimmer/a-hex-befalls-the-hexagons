use crate::actor::*;
use crate::animation::{AnimationConfig, AnimationFrameTimer, execute_animations};
use crate::embed_asset;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::prelude::*;
use std::num::NonZero;

const PLAYER_LAYER: f32 = 1.0;
const SQUARE_SIZE: f32 = 20.0;

const THEIF_IMAGE_PATH: &str = "embedded://assets/sprites/theif.png";
const THEIF_FRAME_SIZE: UVec2 = UVec2::new(16, 30);
const THEIF_TEXTURE_COLUMNS: u32 = 2;
const THEIF_TEXTURE_ROWS: u32 = 1;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/theif.png");
        //app.add_plugins(controls::GameControlsPlugin);
        app.init_resource::<AnimationFrameTimer>()
            .add_systems(OnEnter(GameState::Game), (fixup_room, spawn_theif))
            .add_systems(Update, execute_animations);
    }
}

fn fixup_room(mut commands: Commands, tilemap: Single<(Entity, &TileStorage), With<RoomTilemap>>) {
    let (entity, tile_storage) = *tilemap;

    commands.entity(entity).insert(Pickable::default());

    for tile in tile_storage.iter().filter_map(|t| *t) {
        commands
            .entity(tile)
            .insert((Pickable::default(), Visibility::Visible));
        //.observe(tile_hover_indicator)
        //.observe(tile_hover_indicator_remove);
    }
}

fn spawn_theif(
    mut commands: Commands,
    tilemap: Single<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TilemapType,
            &TilemapAnchor,
        ),
        With<RoomTilemap>,
    >,
    asset_server: Res<AssetServer>,
) {
    let (map_size, grid_size, tile_size, map_type, map_anchor) = *tilemap;
    let theif_image = asset_server.load(THEIF_IMAGE_PATH);
    let atlas_layout = TextureAtlasLayout::from_grid(
        THEIF_FRAME_SIZE,
        THEIF_TEXTURE_COLUMNS,
        THEIF_TEXTURE_ROWS,
        None,
        None,
    );
    let atlas_layout = asset_server.add(atlas_layout);

    let theif_atlas = TextureAtlas {
        layout: atlas_layout,
        index: 0,
    };

    let center_tile_pos = TilePos {
        x: map_size.x / 2 - 1,
        y: map_size.y / 2,
    };

    let world_pos =
        center_tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);

    commands.spawn((
        Actor {
            team: Team::Player,
            animation: AnimationConfig::new(0, 1, 1),
            attack_damage: AttackDamage(0..10),
            attack_speed: AttackSpeed(NonZero::new(10).unwrap()),
            health: HealthBundle::new(NonZero::new(10).unwrap()),
            hit_chance: HitChance(1.0),
            name: Name::new("Theif"),
            sprite: Sprite {
                image: theif_image,
                texture_atlas: Some(theif_atlas),
                anchor: Anchor::Center,
                custom_size: Some(Vec2::new(32., 60.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(world_pos.x, world_pos.y, PLAYER_LAYER),
        },
        Pickable::default(),
        Visibility::Visible,
    ));
    //.observe(select_player::<Pointer<Click>>);
}
