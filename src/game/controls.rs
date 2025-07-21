use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::fmt::Debug;

const HOVER_INDICATOR_LAYER: f32 = ROOM_TILE_LAYER + 0.1;

pub struct GameControlsPlugin;

impl Plugin for GameControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), spawn_hover_indicator.chain())
            .add_systems(Update, move_to_target.run_if(in_state(GameState::Game)));
    }
}

/// The entity that displays the hover indicator
#[derive(Resource)]
pub struct HoverIndicator(pub Entity);

/// Stored in axial coordinate
#[derive(Component)]
pub struct MoveToTile(TilePos);

#[derive(Component, Reflect)]
pub struct Player {
    pub player_speed: f32,
}

#[derive(Component, Reflect)]
pub struct IsSelected;

fn spawn_hover_indicator(mut commands: Commands, tile_image: Res<HexTileImage>) {
    let id = commands
        .spawn((
            Sprite {
                image: tile_image.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    index: OUTLINE_TILE as usize,
                    layout: tile_image.layout.clone(),
                }),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, HOVER_INDICATOR_LAYER),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .id();

    commands.insert_resource(HoverIndicator(id));
}

pub fn select_player<E: Debug + Clone>(
    ev: Trigger<E>,
    mut commands: Commands,
    query_player: Query<Option<&IsSelected>, With<Player>>,
) {
    if let Ok(is_selected) = query_player.get(ev.target()) {
        match is_selected {
            Some(_) => {
                commands.entity(ev.target()).remove::<IsSelected>();
            }
            None => {
                commands.entity(ev.target()).insert(IsSelected);
            }
        }
    }
}

fn move_to_target(
    mut commands: Commands,
    mut query_player: Query<(Entity, &mut Transform, &Player, &MoveToTile)>,
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
    time: Res<Time>,
) {
    let (map_size, grid_size, tile_size, map_type, map_anchor) = *tilemap;

    for (entity, mut transform, player, MoveToTile(target_tile)) in query_player.iter_mut() {
        let target_tile = TilePos::center_in_world(
            target_tile,
            map_size,
            grid_size,
            tile_size,
            map_type,
            map_anchor,
        );

        let direction = target_tile - transform.translation.xy();
        let distance = direction.length();

        let move_player = direction.normalize_or_zero()
            * (player.player_speed * time.delta_secs()).clamp(0.0, distance);

        transform.translation += move_player.extend(0.0);

        if transform.translation.xy() == target_tile {
            commands.get_entity(entity).unwrap().remove::<MoveToTile>();
        }
    }
}

pub fn tile_hover_indicator(
    mut trigger: Trigger<Pointer<Over>>,
    hover_indicator: Res<HoverIndicator>,
    mut hover_indicator_q: Query<(&mut Transform, &mut Visibility)>,
    tile_q: Query<(&ChildOf, &TilePos)>,
    tilemap_q: Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapTileSize,
            &TilemapType,
            &TilemapAnchor,
        ),
        With<RoomTilemap>,
    >,
) {
    trigger.propagate(false);

    let Ok((parent, tile_pos)) = tile_q.get(trigger.target()) else {
        // this should be observing a tile, and that tile should have a position.
        return;
    };

    let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.get(parent.parent())
    else {
        return;
    };

    let Ok((mut transform, mut visibility)) = hover_indicator_q.get_mut(hover_indicator.0) else {
        return;
    };

    transform.translation = tile_pos
        .center_in_world(map_size, grid_size, tile_size, map_type, anchor)
        .extend(transform.translation.z);

    *visibility = Visibility::Visible;
}

pub fn tile_hover_indicator_remove(
    mut trigger: Trigger<Pointer<Out>>,
    hover_indicator: Res<HoverIndicator>,
    mut hover_indicator_q: Query<&mut Visibility>,
) {
    trigger.propagate(false);

    let Ok(mut visibility) = hover_indicator_q.get_mut(hover_indicator.0) else {
        return;
    };

    *visibility = Visibility::Hidden;
}
