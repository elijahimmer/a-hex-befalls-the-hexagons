use crate::prelude::*;

const GET_NEWEST_SAVE_GAME: &str = r#"
    SELECT seq FROM sqlite_sequence WHERE name='SaveGame';
"#;

#[derive(Deref, DerefMut, Clone, Reflect, Serialize, Deserialize)]
pub struct GameID(pub u32);

impl GameID {
    pub fn create_new_save_game(db: &Database) -> Self {
        let query = r#"
            INSERT INTO SaveGame()
        "#;
        let statement = db.connection.prepare(query);
    }
}

#[derive(Resource)]
pub struct SaveGame {
    game_id: GameID,
    //items: Vec<Item>
}
