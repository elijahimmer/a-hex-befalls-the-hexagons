use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

const PLAYER_LAYER: f32 = 1.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameState>()
            // TODO: Game re-loading and setup should be different
            .add_systems(OnEnter(AppState::Game), (fixup_room, spawn_theif));
    }
}

#[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[source(AppState = AppState::Game)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    Intermission,
    PlayerTurn,
    EnemyTurn,
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

    let center_tile_pos = TilePos {
        x: map_size.x / 2 - 1,
        y: map_size.y / 2,
    };

    let world_pos =
        center_tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);

    let transform = Transform::from_xyz(world_pos.x, world_pos.y, PLAYER_LAYER);

    commands.spawn((
        Actor::from_name(&asset_server, ActorName::Theif, Team::Player, transform),
        Pickable::default(),
        Visibility::Visible,
    ));

    //.observe(select_player::<Pointer<Click>>);
}
