use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct TileLabel;

pub fn spawn_tile_labels<T: Component,U: Component>(
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
        With<T>,
    >,
    tile_q: Query<&mut TilePos, With<U>>,
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

            commands
                .spawn((
                    Text2d::new(format!("{},{}", tile_pos.x, tile_pos.y)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                    TextLayout::new_with_justify(JustifyText::Center),
                    transform,
                ));
            commands
                .entity(*tile_entity)
                .insert(TileLabel);
        }
    }
}
