use crate::prelude::*;
use bevy::prelude::*;
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
        }
    }

    pub fn load(db: &Database, game_id: GameID) -> Self {
        let query = "SELECT world_seed FROM SaveGame WHERE SaveGame.game_id = :game_id";

        let world_seed: i64 = db
            .connection
            .query_one(query, (game_id.0,), |row| row.get(0))
            .unwrap();

        Self {
            game_id,
            seed: world_seed as u64,
        }
    }

    pub fn save(&self, db: &Database) -> Result<(), DatabaseError> {
        let query = "UPDATE SaveGame SET last_saved = datetime('now') WHERE game_id = ?1;";
        db.connection.execute(query, (self.game_id.0,))?;
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

    {
        let db = world.get_non_send_resource::<Database>().unwrap();
        db.connection.execute_batch("COMMIT;").unwrap();
    }

    info!("Game Save Successful");
}

fn save_game_inner(db: NonSend<Database>, save: Res<SaveGame>) {
    save.save(&db).unwrap();
}

// TODO: Have it spawn the world rest of the game
pub fn load_game(world: &mut World) {
    info!("Loading Game");
    let actors = world
        .run_system_cached(crate::actor::load_actors)
        .unwrap()
        .unwrap();

    world.spawn_batch(actors.into_iter());

    world
        .get_resource_mut::<NextState<AppState>>()
        .unwrap()
        .set(AppState::Game);
    info!("Game Load Successful")
}
