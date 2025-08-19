use crate::generate_map::*;
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[cfg(feature = "sqlite")]
pub fn save_map(
    tile_storage: Single<&TileStorage, With<MapTilemap>>,
    info_q: Query<(&TilePos, &RoomInfo), With<MapTile>>,
    save_info: Res<SaveGame>,
    db: NonSend<Database>,
) -> Result<(), DatabaseError> {
    let game_id = save_info.game_id.0;

    let query = r#"
            INSERT OR REPLACE INTO RoomInfo(
                game_id,
                position_x,
                position_y,
                cleared,
                r_type,
                rng_seed
            )
            VALUES(
                :game_id,
                :position_x,
                :position_y,
                :cleared,
                :r_type,
                :rng_seed
            );
        "#;

    let mut query = db.connection.prepare(query)?;

    for (
        TilePos { x: pos_x, y: pos_y },
        RoomInfo {
            cleared,
            r_type,
            rng_seed,
        },
    ) in tile_storage
        .iter()
        .filter_map(|entity| *entity)
        .filter_map(|entity| info_q.get(entity).ok())
    {
        let r_type = ron::to_string(&r_type).unwrap();

        query.execute((game_id, pos_x, pos_y, cleared, r_type, *rng_seed as i64))?;
    }

    Ok(())
}

#[cfg(feature = "sqlite")]
pub fn load_map(
    mut commands: Commands,
    db: NonSend<Database>,
    save_game: Res<SaveGame>,
    asset_server: Res<AssetServer>,
) -> Result<(), DatabaseError> {
    let tile_sprite = asset_server.load(MAP_TILE_ASSET_LOAD_PATH);

    let game_id = save_game.game_id;
    let query = "
            SELECT
                position_x,
                position_y,
                cleared,
                r_type,
                rng_seed
            FROM RoomInfo WHERE RoomInfo.game_id = :game;
        ";

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(MAP_SIZE);

    db.connection
        .prepare(query)?
        .query_map((game_id.0,), |row| {
            let x = row.get("position_x")?;
            let y = row.get("position_y")?;
            let cleared = row.get("cleared")?;
            let r_type = row.get::<_, String>("r_type")?;
            let r_type = ron::from_str(&r_type).unwrap_or(RoomType::EmptyRoom);
            // cast as sqlite can only store i64s
            let rng_seed = row.get::<_, i64>("rng_seed")? as u64;

            Ok((
                TilePos { x, y },
                RoomInfo {
                    cleared,
                    r_type,
                    rng_seed,
                },
            ))
        })?
        .map(|c| c.unwrap())
        .for_each(|(tile_pos, room_info)| {
            let id = commands
                .spawn((
                    room_info,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(FLOOR_TILE_VARIENTS.start),
                        ..Default::default()
                    },
                    MapTile,
                ))
                .id();
            commands.entity(tilemap_entity).add_child(id);
            tile_storage.set(&tile_pos, id);
        });

    commands.entity(tilemap_entity).insert((
        MapTilemap,
        TilemapBundle {
            grid_size: TILE_SIZE.into(),
            map_type: TilemapType::Hexagon(HexCoordSystem::Column),
            size: MAP_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_sprite.clone()),
            tile_size: TILE_SIZE,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_translation(WORLD_MAP_ORIGIN),
            ..Default::default()
        },
    ));

    Ok(())
}
