use crate::prelude::*;
use bevy::prelude::*;
#[cfg(feature = "sqlite")]
use chrono::{DateTime, Utc};

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
        db.connection
            .execute(query, ((seed as i64).to_string(),))
            .unwrap();

        let game_id = db.connection.last_insert_rowid();

        Self {
            game_id: GameID(game_id),
            seed,
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

    pub fn save(&self, _: &Database) -> DatabaseResult<()> {
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
