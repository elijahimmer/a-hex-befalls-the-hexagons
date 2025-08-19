use crate::generate_map::MapTilemap;
use crate::prelude::*;
use crate::room::CurrentRoom;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[cfg(feature = "sqlite")]
use chrono::{DateTime, Utc};

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SaveState>()
            .add_systems(OnEnter(SaveState::Save), save_game)
            .add_systems(OnEnter(SaveState::Load), load_game);
    }
}

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum SaveState {
    #[default]
    None,
    Save,
    Load,
}

/// The rowid of the save game table.
#[derive(Deref, DerefMut, Clone, Copy)]
pub struct GameID(pub i64);

/// The global resource for the currently loaded save game.
#[derive(Resource)]
pub struct SaveGame {
    pub game_id: GameID,
    /// The seed used to generate the world
    pub seed: u64,
    pub pillar_count: u64,
}

#[cfg(feature = "sqlite")]
impl SaveGame {
    pub fn new(db: &Database, seed: u64) -> Self {
        let query = "INSERT INTO SaveGame(last_saved,world_seed) VALUES(datetime('now'), ?1)";
        db.connection.execute(query, (seed as i64,)).unwrap();

        let game_id = db.connection.last_insert_rowid();

        Self {
            game_id: GameID(game_id),
            seed,
            pillar_count: 0,
        }
    }

    pub fn load(db: &Database, game_id: GameID) -> Self {
        let query =
            "SELECT world_seed,pillar_count FROM SaveGame WHERE SaveGame.game_id = :game_id";

        let world_seed = db
            .connection
            .query_one(query, (game_id.0,), |row| {
                Ok((row.get::<_, i64>(0)?, row.get(1)?))
            })
            .unwrap();

        Self {
            game_id,
            seed: world_seed.0 as u64,
            pillar_count: world_seed.1,
        }
    }

    /// Updates the [`SaveGame`] database entry with the new save time and current room
    pub fn save(&self, db: &Database, current_room: &TilePos) -> Result<(), DatabaseError> {
        let query = "
        UPDATE SaveGame
            SET last_saved = datetime('now'),
                current_room_x = :current_room_x,
                current_room_y = :current_room_y,
                pillar_count = :pillar_count
            WHERE game_id = :game_id";
        db.connection.execute(
            query,
            (
                current_room.x,
                current_room.y,
                self.pillar_count,
                self.game_id.0,
            ),
        )?;
        Ok(())
    }
}

#[cfg(not(feature = "sqlite"))]
impl SaveGame {
    pub fn new(_: &Database, seed: u64) -> Self {
        Self {
            game_id: GameID(0),
            seed,
        }
    }

    pub fn save(&self, _: &Database) -> Result<(), DatabaseError> {
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
#[derive(Clone)]
pub struct SaveGameInfo {
    pub id: GameID,
    pub created: chrono::DateTime<chrono::Local>,
    pub last_saved: chrono::DateTime<chrono::Local>,
    pub world_seed: u64,
}

#[cfg(feature = "sqlite")]
impl SaveGameInfo {
    pub fn get_all(db: &Database) -> Result<Box<[Self]>, DatabaseError> {
        db.connection
            .prepare(
                "SELECT game_id,created,last_saved,world_seed FROM SaveGame ORDER BY game_id DESC",
            )?
            .query_map((), |row| {
                let created: DateTime<Utc> = row.get(1)?;
                let last_saved: DateTime<Utc> = row.get(2)?;
                Ok(Self {
                    id: GameID(row.get(0)?),
                    created: created.into(),
                    last_saved: last_saved.into(),
                    world_seed: row.get::<_, i64>(3)? as u64,
                })
            })?
            .collect()
    }
}

/// Takes the World as this should be the only thing running at the time.
pub fn save_game(world: &mut World) {
    info!("Saving Game");
    {
        let db = world.get_non_send_resource::<Database>().unwrap();
        db.connection.execute_batch("BEGIN TRANSACTION;").unwrap();
    }

    world.run_system_cached(save_game_inner).unwrap();

    world
        .run_system_cached(crate::actor::save_actors)
        .unwrap()
        .unwrap();

    world
        .run_system_cached(crate::spawn_map::save_map)
        .unwrap()
        .unwrap();

    world
        .run_system_cached(crate::items::save_items)
        .unwrap()
        .unwrap();

    {
        let db = world.get_non_send_resource::<Database>().unwrap();
        db.connection.execute_batch("COMMIT;").unwrap();
    }

    info!("Game Save Successful");
}

fn save_game_inner(
    db: NonSend<Database>,
    save: Res<SaveGame>,
    pos: Single<&TilePos, With<CurrentRoom>>,
) {
    save.save(&db, *pos).unwrap();
}

pub fn load_game(world: &mut World) {
    info!("Loading Game");

    world
        .run_system_cached(crate::actor::load_actors)
        .unwrap()
        .unwrap();

    world
        .run_system_cached(crate::spawn_map::load_map)
        .unwrap()
        .unwrap();

    world
        .run_system_cached(crate::items::load_items)
        .unwrap()
        .unwrap();

    world.run_system_cached(load_game_inner).unwrap();

    world
        .get_resource_mut::<NextState<AppState>>()
        .unwrap()
        .set(AppState::Game);

    info!("Game Load Successful")
}

fn load_game_inner(
    mut commands: Commands,
    db: NonSend<Database>,
    save: Res<SaveGame>,
    storage: Single<&TileStorage, With<MapTilemap>>,
) {
    let query =
        "SELECT current_room_x,current_room_y FROM SaveGame WHERE SaveGame.game_id = :game_id";

    let pos = db
        .connection
        .query_one(query, (save.game_id.0,), |row| {
            Ok(TilePos {
                x: row.get(0)?,
                y: row.get(1)?,
            })
        })
        .unwrap();

    let entity = storage.get(&pos).unwrap();

    commands.get_entity(entity).unwrap().insert(CurrentRoom);
}
