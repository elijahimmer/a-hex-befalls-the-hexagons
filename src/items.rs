use crate::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Resource, Deref, DerefMut, Default)]
pub struct Items(pub Vec<Item>);

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Display)]
pub enum Item {
    #[strum(to_string = "Healing Potion")]
    HealingPotion,
    #[strum(to_string = "Vision Potion")]
    VisionPotion,
}

impl Item {
    pub fn get_rand_item(rng: &mut impl Rng) -> Item {
        let item = rng.random_range(0..2);

        match item {
            0 => Item::HealingPotion,
            1 => Item::VisionPotion,
            _ => unreachable!(),
        }
    }
}

// #[cfg(feature = "sqlite")]
// impl Items {
//     pub fn to_database(&self, db: &Database, game_id: GameID) -> Result<(), DatabaseError> {
//         let query = "INSERT INTO Item(game_id, type) VALUES(:game_id, :type)";
//         let statement = db.connection.prepare(query)?;

//         db.connection.execute(query, (game_id.0, i_type))?;

//         Ok(())
//     }

//     pub fn from_database(db: &Database, game_id: GameID) -> Result<Box<[Self]>, DatabaseError> {
//         let query = "SELECT type FROM Item WHERE game_id = :game_id";

//         db.connection
//             .prepare(query)?
//             .query_map((game_id.0,), |row| {
//                 let i_type = row.get::<_, String>(0)?;
//                 let i_type = ron::from_str::<Item>(&i_type).unwrap();
//                 Ok(i_type)
//             })?
//             .collect()
//     }
// }

#[cfg(feature = "sqlite")]
pub fn save_items(
    items: Res<Items>,
    save_info: Res<SaveGame>,
    db: NonSend<Database>,
) -> Result<(), DatabaseError> {
    let game_id = save_info.game_id.0;
    db.connection
        .execute("DELETE FROM Item WHERE game_id = :game_id", (game_id,))?;

    let query = "INSERT INTO Item(game_id, type) VALUES(:game_id, :type)";
    let mut statement = db.connection.prepare(query)?;
    for item in items.0.iter() {
        let item = ron::to_string(&item).unwrap();
        statement.execute((game_id, item))?;
    }

    Ok(())
}

#[cfg(feature = "sqlite")]
pub fn load_items(
    mut commands: Commands,
    save_info: Res<SaveGame>,
    db: NonSend<Database>,
) -> Result<(), DatabaseError> {
    let game_id = save_info.game_id.0;

    let items = db
        .connection
        .prepare("SELECT type FROM Item WHERE game_id = :game_id")?
        .query_map((game_id,), |row| {
            let i_type = row.get::<_, String>(0)?;
            let i_type = ron::from_str(&i_type).unwrap();

            Ok(i_type)
        })?
        .collect::<Result<Vec<Item>, _>>()?;

    commands.insert_resource(Items(items));

    Ok(())
}
