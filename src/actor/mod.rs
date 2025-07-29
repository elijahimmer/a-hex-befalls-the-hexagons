mod attack;
mod health;

pub use attack::*;
pub use health::*;

use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(feature = "sqlite")]
use std::num::NonZero;
use strum::{Display, EnumIter};

/// The typical components for any given actor.
#[derive(Bundle)]
pub struct Actor {
    pub name: ActorName,
    pub team: Team,
    pub health: HealthBundle,
    pub attack: Attack,
    pub speed: AttackSpeed,
    pub transform: Transform,
    pub animation: AnimationBundle,
}

impl Actor {
    pub fn from_name(
        asset_server: &AssetServer,
        name: ActorName,
        team: Team,
        transform: Transform,
    ) -> Self {
        Self {
            name,
            team,
            health: HealthBundle::from_name(name),
            attack: Attack::from_name(name),
            speed: AttackSpeed::from_name(name),
            transform,
            animation: AnimationBundle::from_name(asset_server, name),
        }
    }
}

#[cfg(feature = "sqlite")]
pub fn save_actors(
    components: Query<(&ActorName, &Team, &Health, &Attack, &AttackSpeed)>,
    save_info: Res<SaveGame>,
    db: NonSend<Database>,
) -> Result<(), DatabaseError> {
    let game_id = save_info.game_id;
    for (name, team, health, attack, speed) in components {
        let query = r#"
            INSERT INTO Actor(
                name,
                game_id,
                team,
                health_max,
                health_curr,
                attack_damage_min,
                attack_damage_max,
                hit_chance
                attack_speed,
            )
            VALUES(
                :name,
                :game,
                :team,
                :health_max,
                :health_curr,
                :attack_damage_min,
                :attack_damage_max,
                :hit_chance
                :attack_speed,
            );
        "#;

        db.connection.execute(
            query,
            (
                name.to_string(),
                *game_id,
                team.to_string(),
                health.max(),
                health.current(),
                attack.damage.start,
                attack.damage.end,
                attack.hit_chance,
                speed.0,
            ),
        )?;
    }

    Ok(())
}

#[cfg(feature = "sqlite")]
pub fn load_actors(
    db: NonSend<Database>,
    save_game: Res<SaveGame>,
    asset_server: Res<AssetServer>,
) -> Result<Box<[Actor]>, DatabaseError> {
    let game_id = save_game.game_id;
    let query = r#"
            SELECT
                name,
                team,
                health_curr,
                health_max,
                attack_damage_max,
                attack_damage_min,
                attack_speed,
                hit_chance
            FROM Actor WHERE Actor.game_id = :game;
        "#;

    db.connection
        .prepare(query)?
        .query_map((game_id.0,), |row| {
            let name = row.get::<_, String>("name")?;
            let name = ron::from_str(&name).unwrap_or(ActorName::UnknownJim);

            let team = row.get::<_, String>("team")?;
            let team = ron::from_str(&team).unwrap_or(Team::Enemy);

            let health = HealthBundle::with_current(
                row.get("health_curr")?,
                NonZero::new(row.get("health_max")?).unwrap_or(NonZero::new(1).unwrap()),
            );
            let attack = Attack::new(
                row.get("attack_damage_min")?..row.get("attack_damage_max")?,
                row.get("hit_chance")?,
            );
            let speed = AttackSpeed::new(row.get("attack_speed")?);
            // the actor will be placed after this
            let transform = Transform::IDENTITY;
            let animation = AnimationBundle::from_name(&asset_server, name);

            Ok(Actor {
                name,
                team,
                health,
                attack,
                speed,
                transform,
                animation,
            })
        })?
        .collect()
}

/// The team the actor is in for combat.
#[derive(
    Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumIter, Display,
)]
pub enum Team {
    /// The player controls this actor and
    /// decides their moves.
    Player,
    /// The game decides what these actors do
    /// on a given turn.
    Enemy,
}

/// The team the actor, both in combat and for the sprite image.
#[derive(
    Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumIter, Display,
)]
pub enum ActorName {
    Warrior,
    Priestess,
    Theif,
    Goblin,
    Ogre,
    Skeleton,
    #[strum(to_string = "Unknown Jim")]
    UnknownJim,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Display)]
pub enum SpecialAction {
    #[strum(to_string = "Heal")]
    HealTarget,
    #[strum(to_string = "Crushing Blow")]
    CrushingBlow,
    #[strum(to_string = "Surprise Attack")]
    SurpriseAttack,
}
