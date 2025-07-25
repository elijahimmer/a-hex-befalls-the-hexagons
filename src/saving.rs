use crate::prelude::*;
use bevy::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Deref, DerefMut, Clone, Copy)]
pub struct GameID(pub u32);

#[derive(Resource)]
pub struct SaveGame {
    pub game_id: GameID,
    pub seed: u64,
    //items: Vec<Item>
}

impl SaveGame {
    pub fn new(db: &Database, seed: u64) -> Self {
        let query = "INSERT INTO SaveGame(last_saved,world_seed) VALUES(datetime('now'), ?1)";
        db.connection
            .execute(query, ((seed as i64).to_string(),))
            .unwrap();

        let query = "SELECT MAX(game_id) FROM SaveGame";
        let game_id = db
            .connection
            .query_one(query, (), |row| row.get(0))
            .unwrap();

        Self {
            game_id: GameID(game_id),
            seed,
        }
    }
}

#[derive(Clone)]
pub struct SaveGameInfo {
    pub id: GameID,
    pub created: chrono::DateTime<chrono::Local>,
    pub last_saved: chrono::DateTime<chrono::Local>,
    pub world_seed: u64,
}

impl SaveGameInfo {
    pub fn get_all(db: &Database) -> Result<Vec<Self>, DatabaseError> {
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
