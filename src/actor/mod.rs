mod attack;
mod health;

pub use attack::*;
pub use health::*;

use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

//impl Actor {
//    #[cfg(feature = "sqlite")]
//    pub fn to_database(&self, db: &Database, game_id: GameID) -> Result<(), DatabaseError> {
//        let query = r#"
//            INSERT INTO Actor (
//                name,
//                game,
//                team,
//                health_max,
//                health_curr,
//                attack_damage_min,
//                attack_damage_min,
//                hit_chance
//            )
//            VALUES (
//                :name
//                :game,
//                :team,
//                :health_max,
//                :health_curr,
//                :attack_damage_min,
//                :attack_damage_max,
//                :hit_chance
//            );
//        "#;
//
//        let mut statement = db.connection.prepare(query)?;
//        statement.bind((":name", &*self.name))?;
//        statement.bind((":game", *game_id as i64))?;
//        let team = ron::to_string(&self.team).unwrap();
//        statement.bind((":team", &*team))?;
//        statement.bind((":health_max", self.health.health.max().get() as i64))?;
//        statement.bind((
//            ":health_curr",
//            self.health.health.current().map(|a| a.get()).unwrap_or(0) as i64,
//        ))?;
//        statement.bind((":attack_damage_min", self.attack.damage.start as i64))?;
//        statement.bind((":attack_damage_max", self.attack.damage.end as i64))?;
//        statement.bind((":hit_chance", self.attack.hit_chance as i64))?;
//
//        assert!(matches!(statement.next()?, sqlite::State::Done));
//
//        Ok(())
//    }
//
//    #[cfg(feature = "sqlite")]
//    pub fn from_database(
//        &self,
//        db: &Database,
//        game_id: GameID,
//    ) -> Result<Box<[Self]>, DatabaseError> {
//        let query = r#"
//            SELECT
//                name,
//                team,
//                health_curr,
//                health_max,
//                attack_damage_max,
//                attack_damage_min,
//                attack_speed,
//                hit_chance,
//            FROM Actor WHERE Actor.game = :game;
//        "#;
//
//        let mut statement = db.connection.prepare(query)?;
//        statement.bind((":game", *game_id as i64))?;
//
//        let res = Vec::<Self>::with_capacity(8);
//
//        while matches!(statement.next()?, sqlite::State::Row) {
//            let name = statement.read::<String, _>("name")?;
//            let name = Name::new(&name);
//            let team = statement.read::<String, _>("team")?;
//            let team = ron::from_str(&team).unwrap_or(Team::Enemy);
//            let health = HealthBundle::with_current(
//                statement.read::<i64, _>("health_curr")? as u32,
//                NonZero::new(statement.read::<i64, _>("health_max")? as u32)
//                    .unwrap_or(NonZero::new(1).unwrap()),
//            );
//            let attack = Attack::new(
//                statement.read::<i64, _>("attack_damage_min")? as u32
//                    ..statement.read::<i64, _>("attack_damage_max")? as u32,
//                statement.read::<i64, _>("attack_speed")? as u32,
//                statement.read::<f64, _>("hit_chance")? as f32,
//            );
//            // the actor will be placed after this
//            let transform = Transform::IDENTITY;
//
//            //res.push(
//            //);
//        }
//
//        Ok(res.into_boxed_slice())
//    }
//
//    #[cfg(not(feature = "sqlite"))]
//    pub fn from_database(
//        &self,
//        db: &Database,
//        party_id: PartyID,
//    ) -> Result<Box<[Self]>, DatabaseError> {
//        Ok(())
//    }
//}

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
