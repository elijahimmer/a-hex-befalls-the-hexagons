mod attack;
mod health;

pub use attack::*;
pub use health::*;

use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::num::NonZero;

/// The typical components for any given actor.
#[derive(Bundle)]
pub struct Actor {
    pub name: Name,
    pub team: Team,
    pub health: HealthBundle,
    pub attack: Attack,
    pub transform: Transform,
    pub sprite: Sprite,
    pub animation: AnimationConfig,
}

#[cfg(feature = "sqlite")]
use rusqlite::types::*;

#[cfg(feature = "sqlite")]
impl Actor {
    fn to_database(&self, db: &Database, game_id: GameID) -> Result<(), DatabaseError> {
        let query = r#"
            INSERT INTO Actor (
                name,
                game,
                team,
                health_max,
                health_curr,
                attack_damage_min,
                attack_damage_min,
                hit_chance
            )
            VALUES (
                :name
                :game,
                :team,
                :health_max,
                :health_curr,
                :attack_damage_min,
                :attack_damage_max,
                :hit_chance
            );
        "#;

        db.connection.execute(
            query,
            (
                &*self.name,
                *game_id,
                ron::to_string(&self.team).unwrap(),
                self.health.health.max(),
                self.health.health.current(),
                self.attack.damage.start,
                self.attack.damage.end,
                self.attack.hit_chance,
            ),
        )?;

        Ok(())
    }

    //pub fn from_database(
    //    &self,
    //    db: &Database,
    //    game_id: GameID,
    //) -> Result<Box<[Self]>, DatabaseError> {
    //    let query = r#"
    //        SELECT
    //            name,
    //            team,
    //            health_curr,
    //            health_max,
    //            attack_damage_max,
    //            attack_damage_min,
    //            attack_speed,
    //            hit_chance,
    //        FROM Actor WHERE Actor.game = :game;
    //    "#;

    //    db.connection
    //        .prepare(query)?
    //        .query_map((), |row| {
    //            let name = row.get(0)?;
    //            let name = Name::new(&name);

    //            let team = row.get::<_, String>(1)?;
    //            let team = ron::from_str(&team).unwrap_or(Team::Enemy);

    //            let health = HealthBundle::with_current(
    //                row.get("health_curr")?,
    //                NonZero::new(row.get("health_max")?).unwrap_or(NonZero::new(1).unwrap()),
    //            );
    //            let attack = Attack::new(
    //                row.get("attack_damage_min")?..row.get("attack_damage_max")?,
    //                row.get("attack_speed")?,
    //                row.get("hit_chance")?,
    //            );
    //            // the actor will be placed after this
    //            let transform = Transform::IDENTITY;

    //            Self {
    //                name,
    //                team,
    //                health,
    //                atat
    //            }
    //        })
    //        .collect();
    //}
}

#[cfg(not(feature = "sqlite"))]
impl Actor {
    pub fn to_database(&self, db: &Database, game_id: GameID) -> Result<(), DatabaseError> {
        Ok(())
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn from_database(
        &self,
        db: &Database,
        party_id: PartyID,
    ) -> Result<Box<[Self]>, DatabaseError> {
        Ok(())
    }
}

/// The team the actor is in for combat.
#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Team {
    /// The player controls this actor and
    /// decides their moves.
    Player,
    /// The game decides what these actors do
    /// on a given turn.
    Enemy,
}

///// The team the actor is in for combat.
//#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
//pub enum ActorType {
//    Theif,
//    Ogre,
//    Goblin,
//}
//
//impl ActorType {
//    pub fn get_sprite(self, asset_server: &AssetServer) -> Sprite {
//        let asset = asset_server.load(self.get_sprite_path());
//        let atlas_layout = asset_server.add(atlas_layout);
//
//        let theif_atlas = TextureAtlas {
//            layout: atlas_layout,
//            index: 0,
//        };
//    }
//
//    pub fn get_sprite_path(self) -> &'static str {
//        use ActorType as A;
//        match self {
//            A::Theif => "embedded://assets/sprites/Theif.png",
//            A::Ogre => "embedded://assets/sprites/Ogre.png",
//            A::Goblin => "embedded://assets/sprites/Goblin.png",
//        }
//    }
//
//    pub fn get_sprite_size(self) -> UVec2 {
//        use ActorType as A;
//        match self {
//            A::Theif => UVec2::new(16, 20),
//            A::Ogre => UVec2::new(16, 20),
//            A::Goblin => UVec2::new(16, 20),
//        }
//    }
//}
