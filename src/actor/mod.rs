mod health;

pub use health::*;

use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::Range;

#[derive(Component, Deref, DerefMut, Clone, Reflect, Serialize, Deserialize)]
pub struct PartyID(pub u32);

#[derive(Resource)]
pub struct Party {
    party_id: i32,
    members: Vec<Entity>,
    //items: Vec<Item>
}

impl Party {
    pub fn from_database(
        &self,
        db: &Database,
        mut commands: Commands,
    ) -> Result<Self, DatabaseError> {
        todo!()
    }
}

/// The typical components for any given actor.
#[derive(Bundle)]
pub struct Actor {
    pub name: Name,
    pub team: Team,
    pub health: HealthBundle,
    pub attack_damage: AttackDamage,
    pub attack_speed: AttackSpeed,
    pub hit_chance: HitChance,
    pub transform: Transform,
    pub sprite: Sprite,
    pub animation: AnimationConfig,
}

impl Actor {
    #[cfg(feature = "sqlite")]
    pub fn to_database(&self, db: &Database, game_id: GameID) -> Result<(), DatabaseError> {
        let query = r#"
            INSERT INTO Actor (
                name,
                game,
                health_max,
                health_curr,
                attack_damage_min,
                attack_damage_min,
                hit_chance
            )
            VALUES (
                :name,
                :game,
                :health_max,
                :health_curr,
                :attack_damage_min,
                :attack_damage_max,
                :hit_chance
            );
        "#;

        let mut statement = db.connection.prepare(query)?;
        statement.bind((":name", &*self.name))?;
        statement.bind((":game", *game_id as i64))?;
        statement.bind((":health_max", self.health.health.max().get() as i64))?;
        statement.bind((
            ":health_curr",
            self.health.health.current().map(|a| a.get()).unwrap_or(0) as i64,
        ))?;
        statement.bind((":attack_damage_min", self.attack_damage.start as i64))?;
        statement.bind((":attack_damage_max", self.attack_damage.end as i64))?;
        statement.bind((":hit_chance", *self.hit_chance as i64))?;

        assert!(matches!(statement.next()?, sqlite::State::Done));

        Ok(())
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn to_database(&self, db: &Database, game_id: GameID) -> Result<(), DatabaseError> {
        Ok(())
    }

    #[cfg(feature = "sqlite")]
    pub fn from_database(&self, db: &Database, game_id: GameID) -> Result<Box<[Self]>, DatabaseError> {
        let query = r#"
            SELECT
                name,
                game,
                health_max,
                health_curr,
                attack_damage_min,
                attack_damage_min,
                hit_chance
            FROM Actor WHERE Actor.game = :game;
        "#;

        let mut statement = db.connection.prepare(query)?;
        statement.bind((":game", *game_id as i64))?;

        let res = Vec::<Self>::with_capacity(8);


        while matches!(statement.next()?, sqlite::State::Row) {
            res.append(Self {
                name: statement.get::<u32>(0),
            });
        }

        Ok(res.to_boxed_slice())
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn from_database(&self, db: &Database, party_id: PartyID) -> Result<Box<[Self]>, DatabaseError> {
        Ok(())
    }
}

/// The team the actor is in for combat.
#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Team {
    /// The player controls this actor and
    /// decides their moves.
    Player,
    /// The game decides what these actors do
    /// on a given turn.
    Enemy,
}

/// The range of damage they can do.
#[derive(Component, Deref, DerefMut, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct AttackDamage(pub Range<u32>);

/// Determines the order of turns in combat.
/// Higher numbers means they will go sooner.
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct AttackSpeed(pub NonZero<u32>);

/// The chance the actor has to hit when they attack.
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct HitChance(pub f32);

/// The chance the actor has to block in combat.
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BlockChance(pub f32);
